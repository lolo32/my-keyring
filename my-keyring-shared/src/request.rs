use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RequestId {
    pub push_id: Ulid,
    pub authentication_id: Ulid,
    pub server_id: Ulid,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResponseId {
    pub client_id: Ulid,
}

/// Message sent from the requester to the server to ask a password.
/// It asks for a push with the token `push_id`.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PushRequest {
    /// Push token
    pub push_id: String,
    /// Data to send to the remote, generally a mobile
    pub encrypted_data: Option<Vec<u8>>,
}
