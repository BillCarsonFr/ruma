//! Type for the MatrixRTC slot event ([MSC4143]).
//!
//! Stable: `m.rtc.slot`
//! Unstable: `org.matrix.msc4143.rtc.slot`
//!
//! An `m.rtc.slot` state event defines a MatrixRTC slot: a virtual location for a MatrixRTC
//! application to run in, and the container for the members and sessions of that application. The
//! `state_key` is the `slot_id`, formatted as `{application.type}#{application_slot_id}` (e.g.
//! `m.call#ROOM`).
//!
//! A slot is open when its `status` is [`RtcSlotStatus::Open`] and it carries a valid
//! [`application`][RtcSlotEventContent::application]. Any slot that does not fulfill these
//! requirements is closed.
//!
//! [MSC4143]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143

use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use super::{application::RtcApplication, encryption::RtcEncryption};
use crate::PrivOwnedStr;

/// The content of an `m.rtc.slot` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    type = "org.matrix.msc4143.rtc.slot",
    alias = "m.rtc.slot",
    kind = State,
    state_key_type = String
)]
pub struct RtcSlotEventContent {
    /// The current status of the slot.
    pub status: RtcSlotStatus,

    /// The application that can run in this slot.
    ///
    /// Required if the `status` is [`RtcSlotStatus::Open`]. The application's type must align with
    /// the `state_key` of the event.
    ///
    /// This may be kept around on a closed slot to simplify re-opening it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<RtcApplication>,

    /// The encryption mechanism to use in this slot.
    ///
    /// If absent, encryption is disabled. Encryption is required in encrypted rooms, and MUST NOT
    /// be used in unencrypted rooms.
    ///
    /// This may be kept around on a closed slot to simplify re-opening it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption: Option<RtcEncryption>,
}

impl RtcSlotEventContent {
    /// Creates a new open `RtcSlotEventContent` for the given application, with encryption
    /// disabled.
    pub fn new(application: RtcApplication) -> Self {
        Self { status: RtcSlotStatus::Open, application: Some(application), encryption: None }
    }

    /// Creates a new open `RtcSlotEventContent` for the given application and encryption
    /// mechanism.
    pub fn encrypted(application: RtcApplication, encryption: RtcEncryption) -> Self {
        Self {
            status: RtcSlotStatus::Open,
            application: Some(application),
            encryption: Some(encryption),
        }
    }

    /// Creates a new closed `RtcSlotEventContent`.
    pub fn closed() -> Self {
        Self { status: RtcSlotStatus::Closed, application: None, encryption: None }
    }
}

/// The status of an [`m.rtc.slot`] event.
///
/// [`m.rtc.slot`]: RtcSlotEventContent
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum RtcSlotStatus {
    /// Members may join the slot.
    Open,

    /// The slot is closed, members cannot join it.
    Closed,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{RtcApplication, RtcSlotEventContent, RtcSlotStatus};
    use crate::{
        AnyStateEvent, StateEvent,
        rtc::{application::CallApplication, encryption::RtcEncryption},
    };

    #[test]
    fn slot_event_serialization() {
        let content =
            RtcSlotEventContent::new(RtcApplication::from(CallApplication::new("UUID".to_owned())));

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "status": "open",
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
            })
        );
    }

    #[test]
    fn encrypted_slot_event_serialization() {
        let content = RtcSlotEventContent::encrypted(
            RtcApplication::from(CallApplication::new("UUID".to_owned())),
            RtcEncryption::per_member(),
        );

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "status": "open",
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
                "encryption": {
                    "type": "org.matrix.msc4143.per_member",
                },
            })
        );
    }

    #[test]
    fn closed_slot_event_serialization() {
        let content = RtcSlotEventContent::closed();
        assert_eq!(to_json_value(&content).unwrap(), json!({ "status": "closed" }));
    }

    #[test]
    fn slot_event_deserialization() {
        let json_data = json!({
            "content": {
                "status": "open",
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
                "encryption": {
                    "type": "m.per_member",
                },
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "state_key": "m.call#ROOM",
            "type": "m.rtc.slot"
        });

        let event = from_json_value::<AnyStateEvent>(json_data).unwrap();
        assert_matches!(event, AnyStateEvent::RtcSlot(StateEvent::Original(slot_event)));
        assert_eq!(slot_event.state_key, "m.call#ROOM");
        assert_eq!(slot_event.content.status, RtcSlotStatus::Open);
        assert_matches!(slot_event.content.application, Some(RtcApplication::Call(_)));
        assert_eq!(slot_event.content.encryption, Some(RtcEncryption::per_member()));
    }

    #[test]
    fn closed_slot_event_deserialization() {
        let json_data = json!({
            "content": {
                "status": "closed",
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "state_key": "m.call#ROOM",
            "type": "m.rtc.slot"
        });

        let event = from_json_value::<AnyStateEvent>(json_data).unwrap();
        assert_matches!(event, AnyStateEvent::RtcSlot(StateEvent::Original(slot_event)));
        assert_eq!(slot_event.content.status, RtcSlotStatus::Closed);
        assert_eq!(slot_event.content.application, None);
        assert_eq!(slot_event.content.encryption, None);
    }
}
