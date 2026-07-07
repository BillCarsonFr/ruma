//! Type for the MatrixRTC slot event ([MSC4143]).
//!
//! Stable: `m.rtc.slot`
//! Unstable: `org.matrix.msc4143.rtc.slot`
//!
//! An `m.rtc.slot` state event defines a MatrixRTC slot: a container for the members and sessions
//! of a MatrixRTC application. The `state_key` is the `slot_id`, formatted as
//! `{application.type}#{application_slot_id}` (e.g. `m.call#ROOM`). An empty content (`{}`) closes
//! the slot.
//!
//! [MSC4143]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::application::RtcApplication;

/// The content of an `m.rtc.slot` event.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    type = "org.matrix.msc4143.rtc.slot",
    alias = "m.rtc.slot",
    kind = State,
    state_key_type = String
)]
pub struct RtcSlotEventContent {
    /// The application this slot is for.
    ///
    /// This is `None` when the slot is closed (the content is an empty object).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<RtcApplication>,
}

impl RtcSlotEventContent {
    /// Creates a new open `RtcSlotEventContent` for the given application.
    pub fn new(application: RtcApplication) -> Self {
        Self { application: Some(application) }
    }

    /// Creates a new closed `RtcSlotEventContent` (empty content).
    pub fn closed() -> Self {
        Self { application: None }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{RtcApplication, RtcSlotEventContent};
    use crate::{AnyStateEvent, StateEvent, rtc::application::CallApplication};

    #[test]
    fn slot_event_serialization() {
        let content =
            RtcSlotEventContent::new(RtcApplication::from(CallApplication::new("UUID".to_owned())));

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
            })
        );
    }

    #[test]
    fn closed_slot_event_serialization() {
        let content = RtcSlotEventContent::closed();
        assert_eq!(to_json_value(&content).unwrap(), json!({}));
    }

    #[test]
    fn slot_event_deserialization() {
        let json_data = json!({
            "content": {
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
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
        assert_matches!(slot_event.content.application, Some(RtcApplication::Call(_)));
    }

    #[test]
    fn closed_slot_event_deserialization() {
        let json_data = json!({
            "content": {},
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "state_key": "m.call#ROOM",
            "type": "m.rtc.slot"
        });

        let event = from_json_value::<AnyStateEvent>(json_data).unwrap();
        assert_matches!(event, AnyStateEvent::RtcSlot(StateEvent::Original(slot_event)));
        assert_eq!(slot_event.content.application, None);
    }
}
