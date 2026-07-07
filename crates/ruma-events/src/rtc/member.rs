//! Type for the MatrixRTC member event ([MSC4143]).
//!
//! Stable: `m.rtc.member`
//! Unstable: `org.matrix.msc4143.rtc.member`
//!
//! An `m.rtc.member` event represents a participant's presence in a MatrixRTC slot. It is a
//! [sticky event] ([MSC4354]): the `sticky_key` (a copy of `member.id`) is used to track the
//! membership across updates. A member connects by sending an event with the full membership data,
//! and disconnects by sending an event carrying only the `slot_id`, `sticky_key` and a
//! `disconnect_reason`.
//!
//! [sticky event]: crate::sticky
//! [MSC4143]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143
//! [MSC4354]: https://github.com/matrix-org/matrix-spec-proposals/pull/4354

use ruma_common::{OwnedDeviceId, OwnedUserId};
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use super::{application::RtcApplication, transport::RtcTransport};
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
    /// The slot this member belongs to.
    pub slot_id: String,

    /// Unique key used to track this membership across updates.
    ///
    /// This is a copy of `member.id`.
    pub sticky_key: String,

    /// The application type this membership is for.
    ///
    /// Absent in a disconnecting event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<RtcApplication>,

    /// Identifies this participation instance.
    ///
    /// Absent in a disconnecting event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<RtcMember>,

    /// The transports describing how to access this participant's media.
    ///
    /// Empty in a disconnecting event.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rtc_transports: Vec<RtcTransport>,

    /// Protocol versions and capabilities supported by the sending client.
    ///
    /// Empty in a disconnecting event.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<String>,

    /// The reason this member disconnected.
    ///
    /// Only present in a disconnecting event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disconnect_reason: Option<DisconnectReason>,
}

impl RtcMemberEventContent {
    /// Creates a new connecting `RtcMemberEventContent`.
    pub fn new(
        slot_id: String,
        application: RtcApplication,
        member: RtcMember,
        rtc_transports: Vec<RtcTransport>,
        versions: Vec<String>,
    ) -> Self {
        Self {
            slot_id,
            sticky_key: member.id.clone(),
            application: Some(application),
            member: Some(member),
            rtc_transports,
            versions,
            disconnect_reason: None,
        }
    }

    /// Creates a new disconnecting `RtcMemberEventContent`.
    pub fn disconnected(
        slot_id: String,
        sticky_key: String,
        disconnect_reason: DisconnectReason,
    ) -> Self {
        Self {
            slot_id,
            sticky_key,
            application: None,
            member: None,
            rtc_transports: Vec::new(),
            versions: Vec::new(),
            disconnect_reason: Some(disconnect_reason),
        }
    }
}

/// Identifies a MatrixRTC participation instance.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RtcMember {
    /// Identifier used to distinguish multiple participations.
    ///
    /// This MUST be unique.
    pub id: String,

    /// The Matrix device ID of the participating device.
    ///
    /// Claimed because it is not verified without encryption.
    pub claimed_device_id: OwnedDeviceId,

    /// The Matrix user ID of the participant.
    ///
    /// Claimed because it is not verified without encryption.
    pub claimed_user_id: OwnedUserId,
}

impl RtcMember {
    /// Creates a new `RtcMember`.
    pub fn new(id: String, claimed_device_id: OwnedDeviceId, claimed_user_id: OwnedUserId) -> Self {
        Self { id, claimed_device_id, claimed_user_id }
    }
}

/// The reason a member disconnected from a MatrixRTC session.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DisconnectReason {
    /// High-level category of the disconnection or error.
    pub class: DisconnectClass,

    /// Machine-readable identifier of the specific cause.
    pub reason: String,

    /// Optional human-readable explanation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl DisconnectReason {
    /// Creates a new `DisconnectReason`.
    pub fn new(class: DisconnectClass, reason: String) -> Self {
        Self { class, reason, description: None }
    }
}

/// The high-level category of a [`DisconnectReason`].
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum DisconnectClass {
    /// The user deliberately ended their participation.
    UserAction,

    /// An error occurred on the client.
    ClientError,

    /// An error occurred on the server.
    ServerError,

    /// The member was redirected to another session.
    Redirection,

    /// A failure occurred that cannot be recovered from.
    PermanentFailure,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{owned_device_id, owned_user_id};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{DisconnectClass, DisconnectReason, RtcMember, RtcMemberEventContent};
    use crate::{
        AnyMessageLikeEvent, MessageLikeEvent,
        rtc::{application::CallApplication, transport::RtcTransport},
    };

    #[cfg(feature = "unstable-msc4195")]
    #[test]
    fn connecting_member_event_serialization() {
        let content = RtcMemberEventContent::new(
            "m.call#ROOM".to_owned(),
            CallApplication::new("UUID".to_owned()).into(),
            RtcMember::new(
                "xyzABCDEF0123".to_owned(),
                owned_device_id!("DEVICEID"),
                owned_user_id!("@user:matrix.domain"),
            ),
            vec![RtcTransport::livekit("https://livekit.example.com/jwt".to_owned())],
            vec!["v0".to_owned()],
        );

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "slot_id": "m.call#ROOM",
                "sticky_key": "xyzABCDEF0123",
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
                "member": {
                    "id": "xyzABCDEF0123",
                    "claimed_device_id": "DEVICEID",
                    "claimed_user_id": "@user:matrix.domain",
                },
                "rtc_transports": [
                    {
                        "type": "livekit",
                        "livekit_service_url": "https://livekit.example.com/jwt",
                    }
                ],
                "versions": ["v0"],
            })
        );
    }

    #[test]
    fn connecting_member_event_custom_transport_serialization() {
        let transport = RtcTransport::new(
            "org.example.transport",
            json!({ "url": "https://sfu.example.com" }).as_object().unwrap().clone(),
        )
        .unwrap();
        let content = RtcMemberEventContent::new(
            "m.call#ROOM".to_owned(),
            CallApplication::new("UUID".to_owned()).into(),
            RtcMember::new(
                "xyzABCDEF0123".to_owned(),
                owned_device_id!("DEVICEID"),
                owned_user_id!("@user:matrix.domain"),
            ),
            vec![transport],
            vec!["v0".to_owned()],
        );

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "slot_id": "m.call#ROOM",
                "sticky_key": "xyzABCDEF0123",
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
                "member": {
                    "id": "xyzABCDEF0123",
                    "claimed_device_id": "DEVICEID",
                    "claimed_user_id": "@user:matrix.domain",
                },
                "rtc_transports": [
                    {
                        "type": "org.example.transport",
                        "url": "https://sfu.example.com",
                    }
                ],
                "versions": ["v0"],
            })
        );
    }

    #[test]
    fn disconnecting_member_event_serialization() {
        let content = RtcMemberEventContent::disconnected(
            "m.call#ROOM".to_owned(),
            "xyzABCDEF0123".to_owned(),
            DisconnectReason {
                class: DisconnectClass::ServerError,
                reason: "ice_failed".to_owned(),
                description: Some("Failed to establish peer-to-peer connection via ICE".to_owned()),
            },
        );

        assert_eq!(
            to_json_value(&content).unwrap(),
            json!({
                "slot_id": "m.call#ROOM",
                "sticky_key": "xyzABCDEF0123",
                "disconnect_reason": {
                    "class": "server_error",
                    "reason": "ice_failed",
                    "description": "Failed to establish peer-to-peer connection via ICE",
                },
            })
        );
    }

    #[test]
    fn connecting_member_event_deserialization() {
        let json_data = json!({
            "content": {
                "slot_id": "m.call#ROOM",
                "sticky_key": "xyzABCDEF0123",
                "application": {
                    "type": "m.call",
                    "m.call.id": "UUID",
                },
                "member": {
                    "id": "xyzABCDEF0123",
                    "claimed_device_id": "DEVICEID",
                    "claimed_user_id": "@user:matrix.domain",
                },
                "rtc_transports": [
                    {
                        "type": "livekit",
                        "livekit_service_url": "https://livekit.example.com/jwt",
                    }
                ],
                "versions": ["v0"],
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
        assert_eq!(member_event.content.slot_id, "m.call#ROOM");
        assert_eq!(member_event.content.sticky_key, "xyzABCDEF0123");
        assert_eq!(member_event.content.rtc_transports.len(), 1);
        assert_matches!(member_event.content.member, Some(member));
        assert_eq!(member.id, "xyzABCDEF0123");
    }

    #[test]
    fn disconnecting_member_event_deserialization() {
        let json_data = json!({
            "content": {
                "slot_id": "m.call#ROOM",
                "sticky_key": "xyzABCDEF0123",
                "disconnect_reason": {
                    "class": "user_action",
                    "reason": "hangup",
                },
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
        assert_matches!(member_event.content.member, None);
        assert_matches!(member_event.content.disconnect_reason, Some(reason));
        assert_eq!(reason.class, DisconnectClass::UserAction);
        assert_eq!(reason.reason, "hangup");
    }
}
