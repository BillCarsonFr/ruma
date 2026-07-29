//! The encryption object of the MatrixRTC slot event ([MSC4143]).
//!
//! The `encryption` object identifies the mechanism used to agree on the key material that
//! encrypts the real-time data exchanged in a MatrixRTC slot. It may carry mechanism-specific
//! settings.
//!
//! [MSC4143]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143

use ruma_common::serde::JsonObject;
use serde::{Deserialize, Deserializer, Serialize};

/// The encryption mechanism used in an [`m.rtc.slot`].
///
/// If no encryption mechanism is set on a slot, encryption is disabled. Note that encryption is
/// required in encrypted rooms and MUST NOT be used in unencrypted rooms.
///
/// [`m.rtc.slot`]: super::slot
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "type")]
pub enum RtcEncryption {
    /// Every member maintains a unique sender key which is shared with the other members of the
    /// slot via Olm-encrypted to-device messages.
    ///
    /// The keys are distributed with [`m.rtc.encryption_key`] to-device messages.
    ///
    /// [`m.rtc.encryption_key`]: super::encryption_key
    #[serde(rename = "org.matrix.msc4143.per_member")]
    PerMember(PerMemberEncryption),

    /// A custom encryption mechanism.
    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomEncryption),
}

impl<'de> Deserialize<'de> for RtcEncryption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let mut obj = JsonObject::deserialize(deserializer)?;
        let encryption_type = match obj.remove("type") {
            Some(serde_json::Value::String(s)) => s,
            Some(_) => return Err(D::Error::custom("`type` must be a string")),
            None => return Err(D::Error::missing_field("type")),
        };

        Ok(match encryption_type.as_str() {
            "org.matrix.msc4143.per_member" | "m.per_member" => Self::PerMember(
                serde_json::from_value(serde_json::Value::Object(obj)).map_err(D::Error::custom)?,
            ),
            _ => Self::_Custom(CustomEncryption { encryption_type, data: obj }),
        })
    }
}

impl RtcEncryption {
    /// A constructor to create a custom encryption mechanism.
    ///
    /// Prefer to use the public variants of `RtcEncryption` where possible; this constructor is
    /// meant to be used for unsupported encryption types only and does not allow setting arbitrary
    /// data for supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `encryption_type` is known and serialization of `data` to the
    /// corresponding `RtcEncryption` variant fails.
    pub fn new(encryption_type: &str, data: JsonObject) -> serde_json::Result<Self> {
        Ok(match encryption_type {
            "org.matrix.msc4143.per_member" | "m.per_member" => {
                Self::PerMember(serde_json::from_value(serde_json::Value::Object(data))?)
            }
            _ => Self::_Custom(CustomEncryption {
                encryption_type: encryption_type.to_owned(),
                data,
            }),
        })
    }

    /// Creates a new `RtcEncryption::PerMember`.
    pub fn per_member() -> Self {
        Self::PerMember(PerMemberEncryption::new())
    }

    /// Returns a reference to the encryption type.
    pub fn encryption_type(&self) -> &str {
        match self {
            Self::PerMember(_) => "org.matrix.msc4143.per_member",
            Self::_Custom(custom) => &custom.encryption_type,
        }
    }
}

/// The per-member encryption mechanism (`m.per_member`).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PerMemberEncryption {}

impl PerMemberEncryption {
    /// Creates a new `PerMemberEncryption`.
    pub fn new() -> Self {
        Self {}
    }
}

impl From<PerMemberEncryption> for RtcEncryption {
    fn from(value: PerMemberEncryption) -> Self {
        Self::PerMember(value)
    }
}

/// A custom encryption mechanism.
///
/// This type does not implement `Deserialize` to prevent users from
/// constructing the `_Custom` variant of [`RtcEncryption`] for a known `type`.
/// Deserialize through [`RtcEncryption`] instead.
#[doc(hidden)]
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CustomEncryption {
    /// The type of encryption mechanism.
    #[serde(rename = "type")]
    encryption_type: String,

    /// Remaining encryption data.
    #[serde(flatten)]
    data: JsonObject,
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::RtcEncryption;

    #[test]
    fn serialize_roundtrip_per_member_encryption() {
        let encryption = RtcEncryption::per_member();
        let json = json!({ "type": "org.matrix.msc4143.per_member" });

        assert_eq!(encryption.encryption_type(), "org.matrix.msc4143.per_member");
        assert_eq!(to_json_value(&encryption).unwrap(), json);
        assert_eq!(from_json_value::<RtcEncryption>(json).unwrap(), encryption);
    }

    #[test]
    fn deserialize_stable_per_member_encryption() {
        let json = json!({ "type": "m.per_member" });
        assert_eq!(from_json_value::<RtcEncryption>(json).unwrap(), RtcEncryption::per_member());
    }

    #[test]
    fn serialize_roundtrip_custom_encryption() {
        let json = json!({
            "type": "org.example.encryption",
            "foo": "bar",
        });
        let encryption = from_json_value::<RtcEncryption>(json.clone()).unwrap();

        assert_eq!(encryption.encryption_type(), "org.example.encryption");
        assert_eq!(to_json_value(&encryption).unwrap(), json);
    }
}
