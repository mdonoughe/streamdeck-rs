use super::{Color, DeviceSize, DeviceType};
use failure::Fail;
use serde::{de, ser};
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Information about a connected device.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/registration-procedure/#info-parameter)
#[derive(Debug, Deserialize, Serialize)]
pub struct RegistrationInfoDevice {
    /// The ID of the specific device.
    pub id: String,
    /// The user-specified name of the device.
    ///
    /// Added in Stream Deck software version 4.3.
    pub name: Option<String>,
    /// The size of the device.
    pub size: DeviceSize,
    /// The type of the device.
    #[serde(rename = "type")]
    pub _type: Option<DeviceType>,
}

/// The language the Stream Deck software is running in.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/registration-procedure/#Info-parameter)
#[derive(Debug)]
pub enum Language {
    English,
    French,
    German,
    Spanish,
    Japanese,
    /// Unlike the other lanuages which are not specifically localized to a country, Chinese is specifically zh-CN.
    ChineseChina,
    /// A language that was not documented in the 4.0.0 SDK.
    Unknown(String),
}

impl<'de> de::Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Language;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Language, E>
            where
                E: de::Error,
            {
                Ok(match value {
                    "en" => Language::English,
                    "fr" => Language::French,
                    "de" => Language::German,
                    "es" => Language::Spanish,
                    "ja" => Language::Japanese,
                    "zh_cn" => Language::ChineseChina,
                    value => Language::Unknown(value.to_string()),
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl ser::Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let lang = match self {
            Language::English => "en",
            Language::French => "fr",
            Language::German => "de",
            Language::Spanish => "es",
            Language::Japanese => "ja",
            Language::ChineseChina => "zh_cn",
            Language::Unknown(value) => value,
        };

        serializer.serialize_str(lang)
    }
}

/// The platform on which the Stream Deck software is running.
#[derive(Debug)]
pub enum Platform {
    /// Mac OS X
    Mac,
    /// Windows
    Windows,
    /// A platform not documented in the 4.0.0 SDK.
    Unknown(String),
}

impl ser::Serialize for Platform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let platform = match self {
            Platform::Mac => "mac",
            Platform::Windows => "windows",
            Platform::Unknown(s) => s,
        };

        serializer.serialize_str(platform)
    }
}

impl<'de: 'a, 'a> de::Deserialize<'de> for Platform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Platform;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Platform, E>
            where
                E: de::Error,
            {
                Ok(match value {
                    "mac" => Platform::Mac,
                    "windows" => Platform::Windows,
                    value => Platform::Unknown(value.to_string()),
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

/// Information about the Stream Deck software.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/registration-procedure/#info-parameter)
#[derive(Debug, Deserialize, Serialize)]
pub struct RegistrationInfoApplication {
    pub language: Language,
    pub platform: Platform,
    pub version: String,
}

/// Information about the plugin
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/registration-procedure/#info-parameter)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationInfoPlugin {
    /// Version of the plugin as per the manifest
    pub version: String,
    /// Unique identifier of the plugin
    pub uuid: String,
}

/// The user's preferred colors
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserColors {
    pub button_pressed_background_color: Option<Color>,
    pub button_pressed_border_color: Option<Color>,
    pub button_pressed_text_color: Option<Color>,
    pub disabled_color: Option<Color>,
    pub highlight_color: Option<Color>,
    pub mouse_down_color: Option<Color>,
}

/// Information about the environment the plugin is being loaded into.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/registration-procedure/#info-parameter)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationInfo {
    pub application: RegistrationInfoApplication,
    pub plugin: RegistrationInfoPlugin,
    pub device_pixel_ratio: u8,
    pub devices: Vec<RegistrationInfoDevice>,
    pub colors: UserColors,
}

/// Registration parameters provided to the plugin on startup.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/registration-procedure/#compiled-plugin-registration)
#[derive(Deserialize)]
pub struct RegistrationParams {
    /// The web socket port listening for the plugin.
    pub port: u16,
    /// The uuid of the plugin.
    pub uuid: String,
    /// The event the plugin should send to register with the Stream Deck software.
    pub event: String,
    /// Information about the environment the plugin is being loaded into.
    pub info: RegistrationInfo,
}

/// An error that occurred while collecting the registration parameters.
#[derive(Debug, Fail)]
pub enum RegistrationParamsError {
    /// The port number was not found.
    #[fail(display = "port not provided")]
    NoPort,
    /// The port number was found but could not be parsed.
    #[fail(display = "port could not be parsed")]
    BadPort(#[fail(cause)] std::num::ParseIntError),
    /// The uuid was not found.
    #[fail(display = "uuid not provided")]
    NoUuid,
    /// The registration event to send was not found.
    #[fail(display = "event not provided")]
    NoEvent,
    /// The registration environment info was not found.
    #[fail(display = "info not provided")]
    NoInfo,
    /// The registration environment info could not be parsed.
    #[fail(display = "info could not be parsed")]
    BadInfo(#[fail(cause)] serde_json::Error),
}

impl RegistrationParams {
    /// Pull the registration parameters out of a command line.
    ///
    /// # Examples
    ///
    /// ```
    /// RegistrationParams::from_args(env::args())
    /// ```
    pub fn from_args<I: IntoIterator<Item = String>>(
        args: I,
    ) -> Result<RegistrationParams, RegistrationParamsError> {
        let mut iter = args.into_iter();
        let mut port = None;
        let mut uuid = None;
        let mut event = None;
        let mut info = None;

        loop {
            match iter.next().as_deref() {
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
