use actix_web::{rt::time::Instant, web::Bytes};
use byteorder::BigEndian;
use my_keyring_shared::{request::PushRequest, security::SipHashKeys};
use tokio::{sync::mpsc::Sender, time::Duration};
use zerocopy::U128;

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
    pub async fn heartbeat(
        &mut self,
        id: U128<BigEndian>,
    ) -> (U128<BigEndian>, Result<(), crate::error::Error>) {
        if self.last_heartbeat + Duration::from_secs(15) < Instant::now() {
            self.last_heartbeat = Instant::now();
            (id, self.send("ping", "💓"))
        } else {
            (id, Ok(()))
        }
    }

    pub fn send(&mut self, id: &str, msg: &str) -> Result<(), crate::error::Error> {
        if let Some(sender) = self.sender.as_mut() {
            let msg = if id.is_empty() {
                format!(": {}\n\n", msg)
            } else {
                format!("event: {}\ndata: {}\n\n", id, msg)
            };
            sender
                .try_send(Bytes::from(msg))
                .map_err(|_| crate::error::Error::SseClosed)
        } else {
            Err(crate::error::Error::SseClosed)
        }
    }
}
