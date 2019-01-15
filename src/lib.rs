extern crate failure;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_dns;
extern crate tokio_tcp;
extern crate tokio_tungstenite;
extern crate tungstenite;

pub mod registration;
pub mod socket;

pub use crate::registration::RegistrationInfo;
pub use crate::socket::StreamDeckSocket;

use serde::{de, ser};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum Message<S, M> {
    #[serde(rename_all = "camelCase")]
    KeyDown {
        action: String,
        context: String,
        device: String,
        payload: KeyPayload<S>,
    },
    #[serde(rename_all = "camelCase")]
    KeyUp {
        action: String,
        context: String,
        device: String,
        payload: KeyPayload<S>,
    },
    #[serde(rename_all = "camelCase")]
    WillAppear {
        action: String,
        context: String,
        device: Option<String>,
        payload: VisibilityPayload<S>,
    },
    #[serde(rename_all = "camelCase")]
    WillDisappear {
        action: String,
        context: String,
        device: Option<String>,
        payload: VisibilityPayload<S>,
    },
    #[serde(rename_all = "camelCase")]
    TitleParametersDidChange {
        action: String,
        context: String,
        device: Option<String>,
        payload: TitleParametersPayload<S>,
    },
    #[serde(rename_all = "camelCase")]
    DeviceDidConnect {
        device: String,
        device_info: DeviceInfo,
    },
    #[serde(rename_all = "camelCase")]
    DeviceDidDisconnect { device: String },
    #[serde(rename_all = "camelCase")]
    ApplicationDidLaunch { payload: ApplicationPayload },
    #[serde(rename_all = "camelCase")]
    ApplicationDidTerminate { payload: ApplicationPayload },
    #[serde(rename_all = "camelCase")]
    SendToPlugin {
        action: String,
        context: String,
        payload: M,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum MessageOut<S, M> {
    SetTitle {
        context: String,
        payload: TitlePayload,
    },
    SetImage {
        context: String,
        payload: ImagePayload,
    },
    ShowAlert {
        context: String,
    },
    ShowOk {
        context: String,
    },
    SetSettings {
        context: String,
        payload: S,
    },
    SetState {
        context: String,
        payload: StatePayload,
    },
    SendToPropertyInspector {
        action: String,
        context: String,
        payload: M,
    },
    SwitchToProfile {
        context: String,
        device: String,
        payload: ProfilePayload,
    },
    OpenUrl {
        payload: UrlPayload,
    },
}

#[derive(Debug)]
pub enum Target {
    Both,     // 0
    Hardware, // 1
    Software, // 2
}

impl ser::Serialize for Target {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_u8(match self {
            Target::Both => 0,
            Target::Hardware => 1,
            Target::Software => 2,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitlePayload {
    pub title: Option<String>,
    pub target: Target,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePayload {
    pub image: Option<String>,
    pub target: Target,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatePayload {
    pub state: u8,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePayload {
    pub profile: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlPayload {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyPayload<S> {
    pub settings: S,
    pub coordinates: Option<Coordinates>,
    pub state: u8,
    pub user_desired_state: Option<u8>,
    //TODO: is_in_multi_action ignored. replace coordinates with enum Location { Coordinates, MultiAction }.
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityPayload<S> {
    pub settings: S,
    pub coordinates: Option<Coordinates>,
    pub state: u8,
    //TODO: is_in_multi_action ignored. replace coordinates with enum Location { Coordinates, MultiAction }.
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleParametersPayload<S> {
    pub settings: S,
    pub coordinates: Coordinates,
    pub state: Option<u8>,
    pub title: String,
    pub title_parameters: TitleParameters,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub size: DeviceSize,
    #[serde(rename = "type")]
    pub _type: Option<DeviceType>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationPayload {
    pub application: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coordinates {
    pub column: u8,
    pub row: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Alignment {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleParameters {
    pub font_family: String,
    pub font_size: u8,
    pub font_style: String,
    pub font_underline: bool,
    pub show_title: bool,
    pub title_alignment: Alignment,
    pub title_color: String,
}

pub enum Language {
    English,
    French,
    German,
    Spanish,
    Japanese,
    ChineseChina,
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

pub enum Platform {
    Mac,
    Windows,
    Unknown(String),
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSize {
    pub columns: u8,
    pub rows: u8,
}

#[derive(Debug)]
pub enum DeviceType {
    StreamDeck,
    StreamDeckMini,
    Unknown(u64),
}

impl<'de> de::Deserialize<'de> for DeviceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = DeviceType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<DeviceType, E>
            where
                E: de::Error,
            {
                Ok(match value {
                    0 => DeviceType::StreamDeck,
                    1 => DeviceType::StreamDeckMini,
                    value => DeviceType::Unknown(value),
                })
            }
        }

        deserializer.deserialize_u64(Visitor)
    }
}
