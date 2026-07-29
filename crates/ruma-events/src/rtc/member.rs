//! Type for the MatrixRTC member event ([MSC4143]).
//!
//! Stable: `m.rtc.member`
//! Unstable: `org.matrix.msc4143.rtc.member`
//!
//! An `m.rtc.member` event represents a participant's membership in a MatrixRTC slot. It is a
//! [sticky event] ([MSC4354]): the `sticky_key` (a copy of `member.id`) is used to track the
//! membership across updates. A member joins by sending an event with `member.membership` set to
//! [`RtcMembership::Join`] and the full membership data, and leaves by sending an event with
//! `member.membership` set to [`RtcMembership::Leave`] and, optionally, a
//! [`leave_reason`][RtcMemberEventContent::leave_reason].
//!
//! [sticky event]: crate::sticky
//! [MSC4143]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143
//! [MSC4354]: https://github.com/matrix-org/matrix-spec-proposals/pull/4354

use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use super::{application::RtcApplication, transport::RtcTransports};
use crate::PrivOwnedStr;

/// The content of an `m.rtc.member` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(
    type = "org.matrix.msc4143.rtc.member",
    alias = "m.rtc.member",
    kind = MessageLike
)]
pub struct RtcMemberEventContent {
    /// The `state_key` of the slot that is being joined.
    pub slot_id: String,

    /// The information identifying the member.
    pub member: RtcMember,

    /// The application that is running in the slot.
    ///
    /// Required if the membership is [`RtcMembership::Join`]. The application's type is the same
    /// as in the [`m.rtc.slot`] event.
    ///
    /// [`m.rtc.slot`]: super::slot
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<RtcApplication>,

    /// The transports describing how to exchange real-time data with this member.
    #[serde(default, skip_serializing_if = "RtcTransports::is_empty")]
    pub transports: RtcTransports,

    /// The reason why the member left the slot.
    ///
    /// Only used if the membership is [`RtcMembership::Leave`], and should only be set if the user
    /// actually attempted to join the slot before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leave_reason: Option<LeaveReason>,

    /// The sticky key for the ephemeral map algorithm as defined in the addendum of [MSC4354].
    ///
    /// This has the same value as `member.id`.
    ///
    /// [MSC4354]: https://github.com/matrix-org/matrix-spec-proposals/pull/4354
    pub sticky_key: String,
}

impl RtcMemberEventContent {
    /// Creates a new `RtcMemberEventContent` for joining a slot.
    ///
    /// The `member_id` must be unique for each join of the same user, which means that a different
    /// identifier needs to be used when leaving and then rejoining a slot.
    pub fn join(
        slot_id: String,
        member_id: String,
        application: RtcApplication,
        transports: RtcTransports,
    ) -> Self {
        Self {
            slot_id,
            member: RtcMember::new(member_id.clone(), RtcMembership::Join),
            application: Some(application),
            transports,
            leave_reason: None,
            sticky_key: member_id,
        }
    }

    /// Creates a new `RtcMemberEventContent` for leaving a slot.
    ///
    /// The `member_id` must be the one of the `m.rtc.member` event that was previously used to join
    /// the slot.
    pub fn leave(slot_id: String, member_id: String) -> Self {
        Self {
            slot_id,
            member: RtcMember::new(member_id.clone(), RtcMembership::Leave),
            application: None,
            transports: RtcTransports::new(),
            leave_reason: None,
            sticky_key: member_id,
        }
    }
}

/// The information identifying a member of a MatrixRTC slot.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RtcMember {
    /// The identifier to distinguish multiple members, even for the same user and device.
    ///
    /// This MUST be unique for each join of the same user.
    pub id: String,

    /// The intended membership status.
    pub membership: RtcMembership,
}

impl RtcMember {
    /// Creates a new `RtcMember`.
    pub fn new(id: String, membership: RtcMembership) -> Self {
        Self { id, membership }
    }
}

/// The membership status of an [`RtcMember`].
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum RtcMembership {
    /// The member intends to be joined to the slot.
    Join,

    /// The member intends to have left the slot.
    Leave,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The reason why a member left a MatrixRTC slot.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct LeaveReason {
    /// The identifier for the specific leave cause.
    pub code: LeaveCode,

    /// An optional human-readable explanation of the leave reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl LeaveReason {
    /// Creates a new `LeaveReason` with the given code.
    pub fn new(code: LeaveCode) -> Self {
        Self { code, reason: None }
    }
}

/// The code of a [`LeaveReason`].
///
/// Further values may be introduced by application and/or transport specifications.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum LeaveCode {
    /// The member left intentionally, e.g. by hanging up a call.
    Leave,

    /// The member left through a scheduled delayed leave event.
    DelayedLeave,

    /// The member left because the slot was closed midway through the session.
    SlotClosed,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{LeaveCode, LeaveReason, RtcMemberEventContent, RtcMembership};
    use crate::{
        AnyMessageLikeEvent, MessageLikeEvent,
        rtc::{
            application::CallApplication,
            transport::{RtcTransport, RtcTransports},
        },
    };

    #[cfg(feature = "unstable-msc4195")]
    #[test]
    fn joining_member_event_serialization() {
        let mut transports = RtcTransports::new();
        transports.published =
            vec![RtcTransport::livekit("https://livekit.example.com/jwt".to_owned())];
        transports.can_subscribe = vec!["livekit".to_owned()];

        let content = RtcMemberEventContent::join(
            "m.call#ROOM".to_owned(),
            "xyzABCDEF0123".to_owned(),
            CallApplication::new("UUID".to_owned()).into(),
            transports,
        );

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "slot_id": "m.call#ROOM",
                "member": {
                    "id": "xyzABCDEF0123",
                    "membership": "join",
                },
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
                "transports": {
                    "published": [
                        {
                            "type": "livekit",
                            "livekit_service_url": "https://livekit.example.com/jwt",
                        }
                    ],
                    "can_subscribe": ["livekit"],
                },
                "sticky_key": "xyzABCDEF0123",
            })
        );
    }

    #[test]
    fn joining_member_event_custom_transport_serialization() {
        let transport = RtcTransport::new(
            "org.example.transport",
            json!({ "url": "https://sfu.example.com" }).as_object().unwrap().clone(),
        )
        .unwrap();
        let mut transports = RtcTransports::new();
        transports.published = vec![transport];

        let content = RtcMemberEventContent::join(
            "m.call#ROOM".to_owned(),
            "xyzABCDEF0123".to_owned(),
            CallApplication::new("UUID".to_owned()).into(),
            transports,
        );

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "slot_id": "m.call#ROOM",
                "member": {
                    "id": "xyzABCDEF0123",
                    "membership": "join",
                },
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
                "transports": {
                    "published": [
                        {
                            "type": "org.example.transport",
                            "url": "https://sfu.example.com",
                        }
                    ],
                },
                "sticky_key": "xyzABCDEF0123",
            })
        );
    }

    #[test]
    fn leaving_member_event_serialization() {
        let mut content =
            RtcMemberEventContent::leave("m.call#ROOM".to_owned(), "xyzABCDEF0123".to_owned());
        content.leave_reason = Some(LeaveReason {
            code: LeaveCode::DelayedLeave,
            reason: Some("Client lost connectivity".to_owned()),
        });

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "slot_id": "m.call#ROOM",
                "member": {
                    "id": "xyzABCDEF0123",
                    "membership": "leave",
                },
                "leave_reason": {
                    "code": "delayed_leave",
                    "reason": "Client lost connectivity",
                },
                "sticky_key": "xyzABCDEF0123",
            })
        );
    }

    #[test]
    fn leaving_member_event_without_reason_serialization() {
        let content =
            RtcMemberEventContent::leave("m.call#ROOM".to_owned(), "xyzABCDEF0123".to_owned());

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "slot_id": "m.call#ROOM",
                "member": {
                    "id": "xyzABCDEF0123",
                    "membership": "leave",
                },
                "sticky_key": "xyzABCDEF0123",
            })
        );
    }

    #[test]
    fn joining_member_event_deserialization() {
        let json_data = json!({
            "content": {
                "slot_id": "m.call#ROOM",
                "member": {
                    "id": "xyzABCDEF0123",
                    "membership": "join",
                },
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
                "transports": {
                    "published": [
                        {
                            "type": "livekit",
                            "livekit_service_url": "https://livekit.example.com/jwt",
                        }
                    ],
                    "can_subscribe": ["livekit"],
                },
                "sticky_key": "xyzABCDEF0123",
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:matrix.domain",
            "type": "m.rtc.member"
        });

        let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
        assert_matches!(
            event,
            AnyMessageLikeEvent::RtcMember(MessageLikeEvent::Original(member_event))
        );
        let content = member_event.content;
        assert_eq!(content.slot_id, "m.call#ROOM");
        assert_eq!(content.sticky_key, "xyzABCDEF0123");
        assert_eq!(content.member.id, "xyzABCDEF0123");
        assert_eq!(content.member.membership, RtcMembership::Join);
        assert!(content.application.is_some());
        assert_eq!(content.transports.published.len(), 1);
        assert_eq!(content.transports.can_subscribe, ["livekit"]);
        assert_matches!(content.leave_reason, None);
    }

    #[test]
    fn leaving_member_event_deserialization() {
        let json_data = json!({
            "content": {
                "slot_id": "m.call#ROOM",
                "member": {
                    "id": "xyzABCDEF0123",
                    "membership": "leave",
                },
                "leave_reason": {
                    "code": "slot_closed",
                },
                "sticky_key": "xyzABCDEF0123",
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:matrix.domain",
            "type": "m.rtc.member"
        });

        let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
        assert_matches!(
            event,
            AnyMessageLikeEvent::RtcMember(MessageLikeEvent::Original(member_event))
        );
        let content = member_event.content;
        assert_eq!(content.member.membership, RtcMembership::Leave);
        assert_eq!(content.application, None);
        assert!(content.transports.is_empty());
        assert_matches!(content.leave_reason, Some(leave_reason));
        assert_eq!(leave_reason.code, LeaveCode::SlotClosed);
        assert_eq!(leave_reason.reason, None);
    }
}
