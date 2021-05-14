use actix_web::{rt::time::Instant, web::Bytes};
use byteorder::BigEndian;
use futures::future::join_all;
use log::{debug, trace};
use my_keyring_shared::{request::PushRequest, security::SipHashKeys};
use tokio::{sync::mpsc::Sender, time::Duration};
use zerocopy::U128;

use crate::SseDataType;

#[derive(Debug)]
#[non_exhaustive]
pub enum SseData {
    PushRequest(PushRequest, SipHashKeys),
    SendToken(String),
}

#[derive(Debug)]
pub struct Sse {
    added: Instant,
    timeout: Duration,
    sender: Option<Sender<Bytes>>,
    last_heartbeat: Instant,
    data: SseData,
}

impl Sse {
    pub fn new(timeout: u64, data: SseData) -> Self {
        Self {
            added: Instant::now(),
            timeout: Duration::from_secs(timeout),
            sender: None,
            last_heartbeat: Instant::now(),
            data,
        }
    }

    pub fn get_data(&self) -> &SseData {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut SseData {
        &mut self.data
    }

    pub fn get_sender(&self) -> Option<&Sender<Bytes>> {
        self.sender.as_ref()
    }

    pub fn set_sender(&mut self, sender: Sender<Bytes>) {
        self.sender = Some(sender);
    }

    pub fn refresh_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    pub async fn heartbeat(
        &mut self,
        id: U128<BigEndian>,
        added: Instant,
    ) -> (U128<BigEndian>, Result<(), crate::error::Error>) {
        if added.elapsed() > self.timeout {
            (id, Err(crate::error::Error::Timeout))
        } else if self.last_heartbeat + Duration::from_secs(15) < Instant::now() {
            self.refresh_heartbeat();
            (id, self.send("ping", "💓"))
        } else {
            (id, Ok(()))
        }
    }

    pub fn send(&mut self, id: &str, msg: &str) -> Result<(), crate::error::Error> {
        if let Some(sender) = self.sender.as_mut() {
            let mut msg_out = Vec::with_capacity(6);
            if !id.is_empty() {
                msg_out.push("event: ");
                msg_out.push(id);
                msg_out.push("\n");
            }
            msg_out.push("data: ");
            msg_out.push(msg);
            msg_out.push("\n\n");
            let msg = msg_out.concat();
            sender
                .try_send(Bytes::from(msg))
                .map_err(|_| crate::error::Error::SseClosed)
        } else {
            Err(crate::error::Error::NotConnected)
        }
    }
}

pub fn sse_maintenance(sse_pool: SseDataType) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval_at(
            Instant::now() + Duration::from_secs(15),
            Duration::from_secs(5),
        );

        loop {
            interval.tick().await;

            trace!(">>> SSE heartbeat");
            let mut to_remove = Vec::new();
            let connections = {
                let mut sse_pool = sse_pool.write().await;
                let mut connections = Vec::new();
                for (id, sse) in (*sse_pool).iter_mut() {
                    debug!(">>> SSE: {}", id);
                    connections.push(sse.heartbeat(*id, sse.added));
                }
                join_all(connections).await
            };
            for (id, connection) in connections {
                match connection {
                    Ok(()) => {}
                    Err(_) => {
                        // remove the entry, already closed
                        to_remove.push(id);
                    }
                }
            }
            {
                let mut sse_pool = sse_pool.write().await;
                for id in to_remove {
                    (*sse_pool).remove(&id);
                }
            }
        }
    });
}
