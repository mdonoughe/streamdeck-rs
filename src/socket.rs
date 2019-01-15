use super::{Message, MessageOut};
use failure::Fail;
use futures::prelude::*;
use futures::sink::Send;
use serde::{de, ser};
use serde_derive::Serialize;
use std::marker::PhantomData;
use tokio_dns::IoFuture;
use tokio_tcp::TcpStream;
use tokio_tungstenite::{ConnectAsync, WebSocketStream};
use url::{Host, Url};

pub struct StreamDeckSocket<S, MI, MO> {
    inner: WebSocketStream<TcpStream>,
    _s: PhantomData<S>,
    _mi: PhantomData<MI>,
    _mo: PhantomData<MO>,
}

impl<S, MI, MO> StreamDeckSocket<S, MI, MO> {
    pub fn connect<A: Into<Address>>(
        address: A,
        event: String,
        uuid: String,
    ) -> Connect<S, MI, MO> {
        let address: Address = address.into();

        Connect {
            state: Some(match address.url.scheme() {
                "ws" => {
                    let end = address.url.with_default_port(|_| Err(())).unwrap();
                    let future = match end.host {
                        Host::Domain(host) => tokio_dns::TcpStream::connect((host, end.port)),
                        Host::Ipv4(host) => tokio_dns::TcpStream::connect((host, end.port)),
                        Host::Ipv6(host) => tokio_dns::TcpStream::connect((host, end.port)),
                    };
                    ConnectState::Connecting(future, address.url, event, uuid)
                }
                scheme => ConnectState::UnsupportedScheme(scheme.to_string()),
            }),
            _s: PhantomData,
            _mi: PhantomData,
            _mo: PhantomData,
        }
    }
}

#[derive(Debug, Fail)]
pub enum StreamDeckSocketError {
    #[fail(display = "WebSocket error")]
    WebSocketError(#[fail(cause)] tungstenite::error::Error),
    #[fail(display = "Bad message")]
    BadMessage(#[fail(cause)] serde_json::Error),
}

impl<S, MI, MO> Stream for StreamDeckSocket<S, MI, MO>
where
    S: de::DeserializeOwned,
    MI: de::DeserializeOwned,
{
    type Item = Message<S, MI>;
    type Error = StreamDeckSocketError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.inner.poll() {
                Ok(Async::Ready(Some(tungstenite::Message::Text(message)))) => {
                    break match serde_json::from_str(&message) {
                        Ok(message) => Ok(Async::Ready(Some(message))),
                        Err(error) => Err(StreamDeckSocketError::BadMessage(error)),
                    }
                }
                Ok(Async::Ready(Some(_))) => {}
                Ok(Async::Ready(None)) => break Ok(Async::Ready(None)),
                Ok(Async::NotReady) => break Ok(Async::NotReady),
                Err(error) => break Err(StreamDeckSocketError::WebSocketError(error)),
            }
        }
    }
}

impl<S, MI, MO> Sink for StreamDeckSocket<S, MI, MO>
where
    S: ser::Serialize,
    MO: ser::Serialize,
{
    type SinkItem = MessageOut<S, MO>;
    type SinkError = StreamDeckSocketError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        let message = serde_json::to_string(&item).map_err(StreamDeckSocketError::BadMessage)?;
        match self.inner.start_send(tungstenite::Message::Text(message)) {
            Ok(AsyncSink::Ready) => Ok(AsyncSink::Ready),
            Ok(AsyncSink::NotReady(_)) => Ok(AsyncSink::NotReady(item)),
            Err(error) => Err(StreamDeckSocketError::WebSocketError(error)),
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.inner
            .poll_complete()
            .map_err(StreamDeckSocketError::WebSocketError)
    }
}

pub struct Address {
    pub url: Url,
}

impl From<Url> for Address {
    fn from(value: Url) -> Self {
        Address { url: value }
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        let mut url = Url::parse("ws://localhost").unwrap();
        url.set_port(Some(value)).unwrap();
        Address { url }
    }
}

#[derive(Debug, Fail)]
pub enum ConnectError {
    #[fail(display = "Unsupported scheme \"{}\"", _0)]
    UnsupportedScheme(String),
    #[fail(display = "Connection error")]
    ConnectionError(#[fail(cause)] std::io::Error),
    #[fail(display = "Websocket protocol error")]
    ProtocolError(#[fail(cause)] tungstenite::error::Error),
    #[fail(display = "Send error")]
    SendError(#[fail(cause)] tungstenite::error::Error),
}

enum ConnectState {
    UnsupportedScheme(String),
    Connecting(IoFuture<TcpStream>, Url, String, String),
    Negotiating(ConnectAsync<TcpStream>, String, String),
    Registering(Send<WebSocketStream<TcpStream>>),
}

pub struct Connect<S, MI, MO> {
    state: Option<ConnectState>,
    _s: PhantomData<S>,
    _mi: PhantomData<MI>,
    _mo: PhantomData<MO>,
}

#[derive(Serialize)]
pub struct Registration<'a> {
    event: &'a str,
    uuid: &'a str,
}

impl<S, MI, MO> Future for Connect<S, MI, MO> {
    type Item = StreamDeckSocket<S, MI, MO>;
    type Error = ConnectError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.state = Some(loop {
            self.state = Some(match self.state.take() {
                Some(ConnectState::UnsupportedScheme(scheme)) => {
                    return Err(ConnectError::UnsupportedScheme(scheme.to_string()))
                }
                Some(ConnectState::Connecting(mut future, url, event, uuid)) => {
                    match future.poll() {
                        Ok(Async::Ready(stream)) => {
                            let _ = stream.set_nodelay(true);
                            ConnectState::Negotiating(
                                tokio_tungstenite::client_async(url, stream),
                                event,
                                uuid,
                            )
                        }
                        Ok(Async::NotReady) => {
                            break ConnectState::Connecting(future, url, event, uuid)
                        }
                        Err(err) => return Err(ConnectError::ConnectionError(err)),
                    }
                }
                Some(ConnectState::Negotiating(mut future, event, uuid)) => match future.poll() {
                    Ok(Async::Ready((stream, _))) => {
                        let message = serde_json::to_string(&Registration {
                            event: &event,
                            uuid: &uuid,
                        })
                        .unwrap();
                        ConnectState::Registering(stream.send(tungstenite::Message::Text(message)))
                    }
                    Ok(Async::NotReady) => break ConnectState::Negotiating(future, event, uuid),
                    Err(err) => return Err(ConnectError::ProtocolError(err)),
                },
                Some(ConnectState::Registering(mut future)) => match future.poll() {
                    Ok(Async::Ready(stream)) => {
                        return Ok(Async::Ready(StreamDeckSocket {
                            inner: stream,
                            _s: PhantomData,
                            _mi: PhantomData,
                            _mo: PhantomData,
                        }))
                    }
                    Ok(Async::NotReady) => break ConnectState::Registering(future),
                    Err(err) => return Err(ConnectError::SendError(err)),
                },
                None => panic!("tried to poll consumed future"),
            })
        });
        Ok(Async::NotReady)
    }
}
