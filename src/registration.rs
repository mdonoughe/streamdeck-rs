use super::{DeviceSize, DeviceType, Language, Platform};
use failure::Fail;
use serde_derive::Deserialize;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct RegistrationInfoDevice {
    pub id: String,
    pub size: DeviceSize,
    #[serde(rename = "type")]
    pub _type: Option<DeviceType>,
}

#[derive(Deserialize)]
pub struct RegistrationInfoApplication {
    pub language: Language,
    pub platform: Platform,
    pub version: String,
}

#[derive(Deserialize)]
pub struct RegistrationInfo {
    pub application: RegistrationInfoApplication,
    pub devices: Vec<RegistrationInfoDevice>,
}

#[derive(Deserialize)]
pub struct RegistrationParams {
    pub port: u16,
    pub uuid: String,
    pub event: String,
    pub info: RegistrationInfo,
}

#[derive(Debug, Fail)]
pub enum RegistrationParamsError {
    #[fail(display = "port not provided")]
    NoPort,
    #[fail(display = "port could not be parsed")]
    BadPort(#[fail(cause)] std::num::ParseIntError),
    #[fail(display = "uuid not provided")]
    NoUuid,
    #[fail(display = "event not provided")]
    NoEvent,
    #[fail(display = "info not provided")]
    NoInfo,
    #[fail(display = "info could not be parsed")]
    BadInfo(#[fail(cause)] serde_json::Error),
}

impl RegistrationParams {
    pub fn from_args<I: IntoIterator<Item = String>>(
        args: I,
    ) -> Result<RegistrationParams, RegistrationParamsError> {
        let mut iter = args.into_iter();
        let mut port = None;
        let mut uuid = None;
        let mut event = None;
        let mut info = None;

        loop {
            match iter.next().as_ref().map(|a| a.as_str()) {
                Some("-port") => port = iter.next().map(|a| u16::from_str(&a)),
                Some("-pluginUUID") => uuid = iter.next(),
                Some("-registerEvent") => event = iter.next(),
                Some("-info") => info = iter.next().map(|a| serde_json::from_str(&a)),
                Some(_) => {}
                None => break,
            }
        }
        let port = port
            .ok_or(RegistrationParamsError::NoPort)?
            .map_err(RegistrationParamsError::BadPort)?;
        let uuid = uuid.ok_or(RegistrationParamsError::NoUuid)?;
        let event = event.ok_or(RegistrationParamsError::NoEvent)?;
        let info = info
            .ok_or(RegistrationParamsError::NoInfo)?
            .map_err(RegistrationParamsError::BadInfo)?;

        Ok(RegistrationParams {
            port,
            uuid,
            event,
            info,
        })
    }
}
