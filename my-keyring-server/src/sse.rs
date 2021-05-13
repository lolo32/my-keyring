use actix_web::rt::time::Instant;
use my_keyring_shared::{request::PushRequest, security::SipHashKeys};
use tokio::time::Duration;
use tokio::sync::mpsc::Sender;
use actix_web::web::Bytes;
use actix_web::Error;
use tokio::sync::mpsc::error::TrySendError;

#[derive(Debug)]
#[non_exhaustive]
pub enum SseData {
    PushRequest(PushRequest, SipHashKeys),
    SendToken(String),
}

#[derive(Debug)]
pub struct Sse {
    pub sender: Option<Sender<Bytes>>,
    pub last_heartbeat: Instant,
    pub data: SseData,
}

impl Sse {
    pub async fn heartbeat(&mut self, id: u128) -> (u128, Result<(), TrySendError<Bytes>>) {
        if self.last_heartbeat + Duration::from_secs(15) < Instant::now() {
            self.last_heartbeat = Instant::now();
            (id, self.send("heart", "💓"))
        } else {
            (id, Ok(()))
        }
    }

    pub fn send(&mut self, id: &str, msg: &str) -> Result<(), TrySendError<Bytes>> {
        if let Some(sender) = self.sender.as_mut() {
                let msg = if id.is_empty() {
                    format!(": {}\n\n", msg)
                } else {
                    format!("event: {}\ndata: {}\n\n", id, msg)
                };
                sender
                    .try_send(Bytes::from(msg))
        } else {
            Err(TrySendError::Closed(Default::default()))
        }
    }
}
