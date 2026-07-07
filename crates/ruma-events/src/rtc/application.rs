//! The application object shared by MatrixRTC events ([MSC4143]).
//!
//! The `application` object identifies the MatrixRTC application type (e.g. `m.call`) and may
//! carry application-specific constraints. It is used by both the `m.rtc.slot` and `m.rtc.member`
//! events.
//!
//! [MSC4143]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143

use ruma_common::serde::JsonObject;
use serde::{Deserialize, Deserializer, Serialize};

/// The MatrixRTC application an [`m.rtc.slot`] or [`m.rtc.member`] event applies to.
///
/// [`m.rtc.slot`]: super::slot
/// [`m.rtc.member`]: super::member
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "type")]
pub enum RtcApplication {
    /// A voice or video call application.
    #[serde(rename = "m.call")]
    Call(CallApplication),

    /// A custom application.
    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomApplication),
}

impl<'de> Deserialize<'de> for RtcApplication {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let mut obj = JsonObject::deserialize(deserializer)?;
        let application_type = match obj.remove("type") {
            Some(serde_json::Value::String(s)) => s,
            Some(_) => return Err(D::Error::custom("`type` must be a string")),
            None => return Err(D::Error::missing_field("type")),
        };

        Ok(match application_type.as_str() {
            "m.call" => Self::Call(
                serde_json::from_value(serde_json::Value::Object(obj)).map_err(D::Error::custom)?,
            ),
            _ => Self::_Custom(CustomApplication { application_type, data: obj }),
        })
    }
}

impl RtcApplication {
    /// A constructor to create a custom application.
    ///
    /// Prefer to use the public variants of `RtcApplication` where possible; this constructor is
    /// meant to be used for unsupported application types only and does not allow setting arbitrary
    /// data for supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `application_type` is known and serialization of `data` to the
    /// corresponding `RtcApplication` variant fails.
    pub fn new(application_type: &str, data: JsonObject) -> serde_json::Result<Self> {
        Ok(match application_type {
            "m.call" => Self::Call(serde_json::from_value(serde_json::Value::Object(data))?),
            _ => Self::_Custom(CustomApplication {
                application_type: application_type.to_owned(),
                data,
            }),
        })
    }

    /// Returns a reference to the application type.
    pub fn application_type(&self) -> &str {
        match self {
            Self::Call(_) => "m.call",
            Self::_Custom(custom) => &custom.application_type,
        }
    }
}

/// A voice or video call application (`m.call`).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct CallApplication {
    /// The identifier of the call.
    #[serde(rename = "m.call.id")]
    pub call_id: String,

    /// Whether this call is voice-only.
    #[serde(
        rename = "m.call.voice_only",
        default,
        skip_serializing_if = "ruma_common::serde::is_default"
    )]
    pub voice_only: bool,
}

impl CallApplication {
    /// Creates a new `CallApplication` with the given call ID.
    pub fn new(call_id: String) -> Self {
        Self { call_id, voice_only: false }
    }
}

impl From<CallApplication> for RtcApplication {
    fn from(value: CallApplication) -> Self {
        Self::Call(value)
    }
}

/// A custom MatrixRTC application.
///
/// This type does not implement `Deserialize` to prevent users from
/// constructing the `_Custom` variant of [`RtcApplication`] for a known `type`.
/// Deserialize through [`RtcApplication`] instead.
#[doc(hidden)]
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CustomApplication {
    /// The type of application.
    #[serde(rename = "type")]
    application_type: String,

    /// Remaining application data.
    #[serde(flatten)]
    data: JsonObject,
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{CallApplication, RtcApplication};

    #[test]
    fn serialize_roundtrip_call_application() {
        let application = RtcApplication::from(CallApplication::new("UUID".to_owned()));
        let json = json!({
            "type": "m.call",
            "m.call.id": "UUID",
        });

        assert_eq!(application.application_type(), "m.call");
        assert_eq!(to_json_value(&application).unwrap(), json);
        assert_eq!(from_json_value::<RtcApplication>(json).unwrap(), application);
    }

    #[test]
    fn serialize_roundtrip_call_application_voice_only() {
        let mut call = CallApplication::new("UUID".to_owned());
        call.voice_only = true;
        let application = RtcApplication::from(call);
        let json = json!({
            "type": "m.call",
            "m.call.id": "UUID",
            "m.call.voice_only": true,
        });

        assert_eq!(to_json_value(&application).unwrap(), json);
        assert_eq!(from_json_value::<RtcApplication>(json).unwrap(), application);
    }

    #[test]
    fn serialize_roundtrip_custom_application() {
        let json = json!({
            "type": "org.example.app",
            "foo": "bar",
        });
        let application = from_json_value::<RtcApplication>(json.clone()).unwrap();

        assert_eq!(application.application_type(), "org.example.app");
        assert_eq!(to_json_value(&application).unwrap(), json);
    }
}
