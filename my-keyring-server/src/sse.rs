use std::io::ErrorKind;

use my_keyring_shared::{request::PushRequest, security::SipHashKeys};
use tokio::time::{Duration, Instant};

#[derive(Debug)]
#[non_exhaustive]
pub enum SseData {
    PushRequest(PushRequest, SipHashKeys),
    SendToken(String),
}

#[derive(Debug)]
pub struct Sse {
    // pub sender: Option<saphir::hyper::body::Sender>,
    pub sender: Option<bool>,
    pub last_heartbeat: Instant,
    pub data: SseData,
}

impl Sse {
    pub async fn heartbeat(&mut self, id: u128) -> (u128, std::io::Result<()>) {
        if self.last_heartbeat + Duration::from_secs(15) < Instant::now() {
            self.last_heartbeat = Instant::now();
            (id, self.send("heart", "💓").await)
        } else {
            (id, Ok(()))
        }
    }

    pub async fn send(&mut self, id: &str, msg: &str) -> std::io::Result<()> {
        if let Some(sender) = self.sender.as_mut() {
            //     let msg = if id.is_empty() {
            //         format!(": {}\n\n", msg)
            //     } else {
            //         format!("event: {}\ndata: {}\n\n", id, msg)
            //     };
            //     sender
            //         .send_data(Bytes::from(msg))
            //         .await
            //         .map_err(|e| SaphirError::Other(e.to_string()))
            // } else {
            //     Err(SaphirError::Other("No listener attached".to_owned()))
            // }
            let msg = if id.is_empty() {
                format!(": {}\n\n", msg)
            } else {
                format!("event: {}\ndata: {}\n\n", id, msg)
            };
            todo!()
        } else {
            Err(std::io::Error::new(
                ErrorKind::BrokenPipe,
                "No listener attached",
            ))
        }
    }
}
