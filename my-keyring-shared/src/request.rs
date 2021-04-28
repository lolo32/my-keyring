use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct RequestId {
    pub client_id: Ulid,
    pub authentication_id: Ulid,
    pub server_id: Ulid,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct ResponseId {
    pub client_id: Ulid
}