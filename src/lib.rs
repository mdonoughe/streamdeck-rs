extern crate failure;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_dns;
extern crate tokio_tcp;
extern crate tokio_tungstenite;
extern crate tungstenite;

#[cfg(feature = "logging")]
pub mod logging;
pub mod registration;
pub mod socket;

pub use crate::registration::RegistrationInfo;
pub use crate::socket::StreamDeckSocket;

use serde::{de, ser};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

/// A message received from the Stream Deck software.
///
/// - `G` represents the global settings that are persisted within the Stream Deck software.
/// - `S` represents the settings that are persisted within the Stream Deck software.
/// - `M` represents the messages that are received from the property inspector.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/)
#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum Message<G, S, M> {
    /// A key has been pressed.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#keydown)
    #[serde(rename_all = "camelCase")]
    KeyDown {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device where the key was pressed.
        device: String,
        /// Additional information about the key press.
        payload: KeyPayload<S>,
    },
    /// A key has been released.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#keyup)
    #[serde(rename_all = "camelCase")]
    KeyUp {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device where the key was pressed.
        device: String,
        /// Additional information about the key press.
        payload: KeyPayload<S>,
    },
    /// An instance of the action has been added to the display.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#willappear)
    #[serde(rename_all = "camelCase")]
    WillAppear {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device where the action will appear, or None if it does not appear on a device.
        device: Option<String>,
        /// Additional information about the action's appearance.
        payload: VisibilityPayload<S>,
    },
    /// An instance of the action has been removed from the display.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#willdisappear)
    #[serde(rename_all = "camelCase")]
    WillDisappear {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device where the action was visible, or None if it was not on a device.
        device: Option<String>,
        /// Additional information about the action's appearance.
        payload: VisibilityPayload<S>,
    },
    /// The title has changed for an instance of an action.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#titleparametersdidchange)
    #[serde(rename_all = "camelCase")]
    TitleParametersDidChange {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device where the action is visible, or None if it is not on a device.
        device: Option<String>,
        /// Additional information about the new title.
        payload: TitleParametersPayload<S>,
    },
    /// A device has connected.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#devicedidconnect)
    #[serde(rename_all = "camelCase")]
    DeviceDidConnect {
        /// The ID of the device that has connected.
        device: String,
        /// Information about the device.
        device_info: DeviceInfo,
    },
    /// A device has disconnected.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#devicediddisconnect)
    #[serde(rename_all = "camelCase")]
    DeviceDidDisconnect {
        /// The ID of the device that has disconnected.
        device: String,
    },
    /// An application monitored by the manifest file has launched.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#applicationdidlaunch)
    #[serde(rename_all = "camelCase")]
    ApplicationDidLaunch {
        /// Information about the launched application.
        payload: ApplicationPayload,
    },
    /// An application monitored by the manifest file has terminated.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#applicationdidterminate)
    #[serde(rename_all = "camelCase")]
    ApplicationDidTerminate {
        /// Information about the terminated application.
        payload: ApplicationPayload,
    },
    /// The property inspector has sent data.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#sendtoplugin)
    #[serde(rename_all = "camelCase")]
    SendToPlugin {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// Information sent from the property inspector.
        payload: M,
    },
    /// The application has sent settings for an action.
    ///
    /// This message is sent in response to GetSettings, but also after the
    /// property inspector changes the settings.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#didreceivesettings)
    #[serde(rename_all = "camelCase")]
    DidReceiveSettings {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device where the action exists.
        device: String,
        /// The current settings for the action.
        payload: KeyPayload<S>,
    },
    /// The property inspector for an action has become visible.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#propertyinspectordidappear)
    #[serde(rename_all = "camelCase")]
    PropertyInspectorDidAppear {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device where the action exists.
        device: String,
    },
    /// The property inspector for an action is no longer visible.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#propertyinspectordiddisappear)
    #[serde(rename_all = "camelCase")]
    PropertyInspectorDidDisappear {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device where the action exists.
        device: String,
    },
    /// The application has sent settings for an action.
    ///
    /// This message is sent in response to GetGlobalSettings, but also after
    /// the property inspector changes the settings.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#didreceiveglobalsettings)
    #[serde(rename_all = "camelCase")]
    DidReceiveGlobalSettings {
        /// The current settings for the action.
        payload: GlobalSettingsPayload<G>,
    },
    /// An event from an unsupported version of the Stream Deck software.
    ///
    /// This occurs when the Stream Deck software sends an event that is not
    /// understood. Usually this will be because the Stream Deck software is
    /// newer than the plugin, and it should be safe to ignore these.
    #[serde(other)]
    Unknown,
}

/// A message to be sent to the Stream Deck software.
///
/// - `G` represents the global settings that are persisted within the Stream Deck software.
/// - `S` represents the action settings that are persisted within the Stream Deck software.
/// - `M` represents the messages that are sent to the property inspector.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/)
#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum MessageOut<G, S, M> {
    /// Set the title of an action instance.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#settitle)
    #[serde(rename_all = "camelCase")]
    SetTitle {
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The title to set.
        payload: TitlePayload,
    },
    /// Set the image of an action instance.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#setimage)
    #[serde(rename_all = "camelCase")]
    SetImage {
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The image to set.
        payload: ImagePayload,
    },
    /// Temporarily overlay the key image with an alert icon.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#showalert)
    #[serde(rename_all = "camelCase")]
    ShowAlert {
        /// The instance of the action (key or part of a multiaction).
        context: String,
    },
    /// Temporarily overlay the key image with a checkmark.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#showok)
    #[serde(rename_all = "camelCase")]
    ShowOk {
        /// The instance of the action (key or part of a multiaction).
        context: String,
    },
    /// Retrieve settings for an instance of an action via DidReceiveSettings.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#getsettings)
    #[serde(rename_all = "camelCase")]
    GetSettings {
        /// The instance of the action (key or part of a multiaction).
        context: String,
    },
    /// Store settings for an instance of an action.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#setsettings)
    #[serde(rename_all = "camelCase")]
    SetSettings {
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The settings to save.
        payload: S,
    },
    /// Set the state of an action.
    ///
    /// Normally, Stream Deck changes the state of an action automatically when the key is pressed.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#setstate)
    #[serde(rename_all = "camelCase")]
    SetState {
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The desired state.
        payload: StatePayload,
    },
    /// Send data to the property inspector.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#sendtopropertyinspector)
    #[serde(rename_all = "camelCase")]
    SendToPropertyInspector {
        /// The uuid of the action.
        action: String,
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The message to send.
        payload: M,
    },
    /// Select a new profile.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#switchtoprofile)
    #[serde(rename_all = "camelCase")]
    SwitchToProfile {
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The device to change the profile of.
        device: String,
        /// The profile to activate.
        payload: ProfilePayload,
    },
    /// Open a URL in the default browser.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#openurl)
    #[serde(rename_all = "camelCase")]
    OpenUrl {
        /// The url to open.
        payload: UrlPayload,
    },
    /// Retrieve plugin settings for via DidReceiveGlobalSettings.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#getglobalsettings)
    #[serde(rename_all = "camelCase")]
    GetGlobalSettings {
        /// The instance of the action (key or part of a multiaction).
        context: String,
    },
    /// Store plugin settings.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#setglobalsettings)
    #[serde(rename_all = "camelCase")]
    SetGlobalSettings {
        /// The instance of the action (key or part of a multiaction).
        context: String,
        /// The settings to save.
        payload: G,
    },
    /// Write to the log.
    ///
    /// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#logmessage)
    #[serde(rename_all = "camelCase")]
    LogMessage {
        /// The message to log.
        payload: LogMessagePayload,
    },
}

/// The target of a command.
#[derive(Debug)]
pub enum Target {
    /// Both the device and a the display within the Stream Deck software.
    Both, // 0
    /// Only the device.
    Hardware, // 1
    /// Only the display within the Stream Deck software.
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

/// The title to set as part of a [SetTitle](enum.MessageOut.html#variant.SetTitle) message.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#settitle)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitlePayload {
    /// The new title.
    pub title: Option<String>,
    /// The target displays.
    pub target: Target,
}

/// The image to set as part of a [SetImage](enum.MessageOut.html#variant.SetImage) message.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#setimage)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePayload {
    /// An image in the form of a data URI.
    pub image: Option<String>,
    /// The target displays.
    pub target: Target,
}

/// The state to set as part of a [SetState](enum.MessageOut.html#variant.SetState) message.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#setstate)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatePayload {
    /// The new state.
    pub state: u8,
}

/// The profile to activate as part of a [SwitchToProfile](enum.MessageOut.html#variant.SwitchToProfile) message.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#SwitchToProfile)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePayload {
    /// The name of the profile to activate.
    pub profile: String,
}

/// The URL to launch as part of a [OpenUrl](enum.MessageOut.html#variant.OpenUrl) message.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-sent/#openurl)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlPayload {
    /// The URL to launch.
    pub url: String,
}

/// Additional information about the key pressed.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyPayload<S> {
    /// The stored settings for the action instance.
    pub settings: S,
    /// The location of the key that was pressed, or None if this action instance is part of a multi action.
    pub coordinates: Option<Coordinates>,
    /// The current state of the action instance.
    pub state: u8,
    /// The desired state of the action instance (if this instance is part of a multi action).
    pub user_desired_state: Option<u8>,
    //TODO: is_in_multi_action ignored. replace coordinates with enum Location { Coordinates, MultiAction }.
}

/// Additional information about a key's appearance.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityPayload<S> {
    /// The stored settings for the action instance.
    pub settings: S,
    /// The location of the key, or None if this action instance is part of a multi action.
    pub coordinates: Option<Coordinates>,
    /// The state of the action instance.
    pub state: u8,
    //TODO: is_in_multi_action ignored. replace coordinates with enum Location { Coordinates, MultiAction }.
}

/// The new title of a key.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleParametersPayload<S> {
    /// The stored settings for the action instance.
    pub settings: S,
    /// The location of the key, or None if this action instance is part of a multi action.
    pub coordinates: Coordinates,
    /// The state of the action instance.
    pub state: Option<u8>,
    /// The new title.
    pub title: String,
    /// Additional parameters for the display of the title.
    pub title_parameters: TitleParameters,
}

/// The new global settings.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSettingsPayload<G> {
    /// The stored settings for the plugin.
    pub settings: G,
}

/// A log message.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogMessagePayload {
    /// The log message text.
    pub message: String,
}

/// Information about a hardware device.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/registration-procedure/#Info-parameter)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// The size of the device.
    pub size: DeviceSize,
    /// The type of the device, or None if the Stream Deck software is running with no device attached.
    #[serde(rename = "type")]
    pub _type: Option<DeviceType>,
}

/// Information about a monitored application that has launched or terminated.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationPayload {
    /// The name of the application.
    pub application: String,
}

/// The location of a key on a device.
///
/// Locations are specified using zero-indexed values starting from the top left corner of the device.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coordinates {
    /// The x coordinate of the key.
    pub column: u8,
    /// The y-coordinate of the key.
    pub row: u8,
}

/// The vertical alignment of a title.
///
/// Titles are always centered horizontally.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Alignment {
    /// The title should appear at the top of the key.
    Top,
    /// The title should appear in the middle of the key.
    Middle,
    /// The title should appear at the bottom of the key.
    Bottom,
}

/// Style information for a title.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/events-received/#titleparametersdidchange)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleParameters {
    /// The name of the font family.
    pub font_family: String,
    /// The font size.
    pub font_size: u8,
    /// Whether the font is bold and/or italic.
    pub font_style: String,
    /// Whether the font is underlined.
    pub font_underline: bool,
    /// Whether the title is displayed.
    pub show_title: bool,
    /// The vertical alignment of the title.
    pub title_alignment: Alignment,
    /// The color of the title.
    pub title_color: String,
}

/// The size of a device in keys.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSize {
    /// The number of key columns on the device.
    pub columns: u8,
    /// The number of key rows on the device.
    pub rows: u8,
}

/// The type of connected hardware device.
///
/// [Official Documentation](https://developer.elgato.com/documentation/stream-deck/sdk/overview/#stream-deck-hardware)
#[derive(Debug)]
pub enum DeviceType {
    /// The [Stream Deck](https://www.elgato.com/en/gaming/stream-deck).
    StreamDeck,
    /// The [Stream Deck Mini](https://www.elgato.com/en/gaming/stream-deck-mini).
    StreamDeckMini,
    /// A device not documented in the 4.0.0 SDK.
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
