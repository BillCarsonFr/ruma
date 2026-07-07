//! Type for the MatrixRTC encryption key event ([MSC4143]).
//!
//! Stable: `m.rtc.encryption_key`
//! Unstable: `org.matrix.msc4143.rtc.encryption_key`
//!
//! An `m.rtc.encryption_key` event distributes a per-participant media encryption key. It is sent
//! as an encrypted to-device message to the other members of a MatrixRTC session.
//!
//! [MSC4143]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143

use ruma_common::OwnedRoomId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.rtc.encryption_key` to-device event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    type = "org.matrix.msc4143.rtc.encryption_key",
    alias = "m.rtc.encryption_key",
    kind = ToDevice
)]
pub struct ToDeviceRtcEncryptionKeyEventContent {
    /// The room the MatrixRTC session belongs to.
    pub room_id: OwnedRoomId,

    /// The `member.id` of the participant this key belongs to.
    pub member_id: String,

    /// The media encryption key material.
    pub media_key: MediaKey,

    /// The version of the key format.
    pub version: String,
}

impl ToDeviceRtcEncryptionKeyEventContent {
    /// Creates a new `ToDeviceRtcEncryptionKeyEventContent`.
    pub fn new(
        room_id: OwnedRoomId,
        member_id: String,
        media_key: MediaKey,
        version: String,
    ) -> Self {
        Self { room_id, member_id, media_key, version }
    }
}

/// Media encryption key material.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct MediaKey {
    /// The index of the key, used to distinguish it from others during rotation.
    ///
    /// A value in the range 0 to 255.
    pub index: u8,

    /// The key material, encoded using unpadded base64.
    pub key: String,
}

impl MediaKey {
    /// Creates a new `MediaKey`.
    pub fn new(index: u8, key: String) -> Self {
        Self { index, key }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::owned_room_id;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{MediaKey, ToDeviceRtcEncryptionKeyEventContent};
    use crate::{AnyToDeviceEvent, ToDeviceEvent};

    #[test]
    fn encryption_key_event_serialization() {
        let content = ToDeviceRtcEncryptionKeyEventContent::new(
            owned_room_id!("!roomid:matrix.domain"),
            "xyzABCDEF0123".to_owned(),
            MediaKey::new(10, "base64encodedkey".to_owned()),
            "0".to_owned(),
        );

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "room_id": "!roomid:matrix.domain",
                "member_id": "xyzABCDEF0123",
                "media_key": {
                    "index": 10,
                    "key": "base64encodedkey",
                },
                "version": "0",
            })
        );
    }

    #[test]
    fn encryption_key_event_deserialization() {
        let json_data = json!({
            "content": {
                "room_id": "!roomid:matrix.domain",
                "member_id": "xyzABCDEF0123",
                "media_key": {
                    "index": 10,
                    "key": "base64encodedkey",
                },
                "version": "0",
            },
            "sender": "@user:matrix.domain",
            "type": "m.rtc.encryption_key"
        });

        let event = from_json_value::<AnyToDeviceEvent>(json_data).unwrap();
        assert_matches!(event, AnyToDeviceEvent::RtcEncryptionKey(ToDeviceEvent { content, .. }));
        assert_eq!(content.member_id, "xyzABCDEF0123");
        assert_eq!(content.media_key.index, 10);
    }
}
