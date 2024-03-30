use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize, Serialize)]
pub(super) struct Payload {
    pub(super) op: Opcode,
    pub(super) d: Value,
    #[serde(skip_serializing)]
    pub(super) s: Option<u64>,
    #[serde(skip_serializing)]
    pub(super) t: Option<Box<str>>,
}

#[repr(u8)]
#[derive(Deserialize_repr, Serialize_repr)]
pub(super) enum Opcode {
    Dispatch,
    Heartbeat,
    Identify,
    PresenceUpdate,
    Resume = 6,
    Reconnect,
    InvalidSession = 9,
    Hello,
    HeartbeatACK,
}
