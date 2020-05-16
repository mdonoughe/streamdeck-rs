use super::{Message, MessageOut};
use failure::Fail;
use futures::prelude::*;
use serde::{de, ser};
use serde_derive::Serialize;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::{self, WebSocketStream};
use url::Url;

/// Provides encoding and decoding for messages sent to/from the Stream Deck software.
///
/// - `S` represents settings persisted within the Stream Deck software.
/// - `MI` represents messages received from the property inspector.
/// - `MO` represents messages sent to the property inspector.
pub struct StreamDeckSocket<G, S, MI, MO> {
    inner: WebSocketStream<TcpStream>,
    _g: PhantomData<G>,
    _s: PhantomData<S>,
    _mi: PhantomData<MI>,
    _mo: PhantomData<MO>,
}

impl<G, S, MI, MO> StreamDeckSocket<G, S, MI, MO> {
    /// Begins connecting to the Stream Deck software.
    ///
    /// `address` may be specified either as a port number or as a `Url`.
    ///
    /// # Examples
    ///
    /// ```
    /// let params = RegistrationParams::from_args(env::args()).unwrap();
    /// let connect = StreamDeckSocket::<GlobalSettings, ActionSettings, PiMessage, PiMessageOut>::connect(params.port, params.event, params.uuid);
    /// tokio::run(connect
    ///     .map_err(|e| println!("connection failed: {:?}", e))
    ///     .and_then(|socket| socket.for_each(|message| println!("received: {:?}", message))
    ///         .map_err(|e| println!("read error: {:?}", e))));
    /// ```
    pub async fn connect<A: Into<Address>>(
        address: A,
        event: String,
        uuid: String,
    ) -> Result<Self, ConnectError> {
        let address = address.into();

        let (mut stream, _) = tokio_tungstenite::connect_async(address.url)
            .await
            .map_err(ConnectError::ConnectionError)?;

        let message = serde_json::to_string(&Registration {
            event: &event,
            uuid: &uuid,
        })
        .unwrap();
        stream
            .send(tungstenite::Message::Text(message))
            .await
            .map_err(ConnectError::SendError)?;

        Ok(StreamDeckSocket {
            inner: stream,
            _g: PhantomData,
            _s: PhantomData,
            _mi: PhantomData,
            _mo: PhantomData,
        })
    }

    fn pin_get_inner(self: Pin<&mut Self>) -> Pin<&mut WebSocketStream<TcpStream>> {
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }
    }
}

/// Represents an error that occurred reading or writing the web socket.
#[derive(Debug, Fail)]
pub enum StreamDeckSocketError {
    /// The web socket reported an error.
    #[fail(display = "WebSocket error")]
    WebSocketError(#[fail(cause)] tungstenite::error::Error),
    /// The message could not be encoded/decoded.
    #[fail(display = "Bad message")]
    BadMessage(#[fail(cause)] serde_json::Error),
}

impl<G, S, MI, MO> Stream for StreamDeckSocket<G, S, MI, MO>
where
    G: de::DeserializeOwned,
    S: de::DeserializeOwned,
    MI: de::DeserializeOwned,
{
    type Item = Result<Message<G, S, MI>, StreamDeckSocketError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut inner = self.pin_get_inner();
        loop {
            match inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(tungstenite::Message::Text(message)))) => {
                    break match serde_json::from_str(&message) {
                        Ok(message) => Poll::Ready(Some(Ok(message))),
                        Err(error) => {
                            Poll::Ready(Some(Err(StreamDeckSocketError::BadMessage(error))))
                        }
                    };
                }
                Poll::Ready(Some(Ok(_))) => {}
                Poll::Ready(Some(Err(error))) => {
                    break Poll::Ready(Some(Err(StreamDeckSocketError::WebSocketError(error))))
                }
                Poll::Ready(None) => break Poll::Ready(None),
                Poll::Pending => break Poll::Pending,
            }
        }
    }
}

impl<G, S, MI, MO> Sink<MessageOut<G, S, MO>> for StreamDeckSocket<G, S, MI, MO>
where
    G: ser::Serialize,
    S: ser::Serialize,
    MO: ser::Serialize,
{
    type Error = StreamDeckSocketError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.pin_get_inner()
            .poll_ready(cx)
            .map_err(StreamDeckSocketError::WebSocketError)
    }

    fn start_send(self: Pin<&mut Self>, item: MessageOut<G, S, MO>) -> Result<(), Self::Error> {
        let message = serde_json::to_string(&item).map_err(StreamDeckSocketError::BadMessage)?;
        self.pin_get_inner()
            .start_send(tungstenite::Message::Text(message))
            .map_err(StreamDeckSocketError::WebSocketError)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.pin_get_inner()
            .poll_flush(cx)
            .map_err(StreamDeckSocketError::WebSocketError)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.pin_get_inner()
            .poll_close(cx)
            .map_err(StreamDeckSocketError::WebSocketError)
    }
}

/// Represents an address to connect to.
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

/// Represents an error that occurred while connecting to and registering with the Stream Deck software.
#[derive(Debug, Fail)]
pub enum ConnectError {
    /// The web socket connection could not be established.
    #[fail(display = "Websocket connection error")]
    ConnectionError(#[fail(cause)] tungstenite::error::Error),
    /// The registration information could not be sent.
    #[fail(display = "Send error")]
    SendError(#[fail(cause)] tungstenite::error::Error),
}

#[derive(Serialize)]
struct Registration<'a> {
    event: &'a str,
    uuid: &'a str,
}
