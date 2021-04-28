use std::collections::HashMap;

use futures::future::join_all;
use log::{debug, trace};
use my_keyring_shared::request::RequestId;
use once_cell::sync::Lazy;
use saphir::prelude::*;
use tokio::{
    sync::RwLock,
    time::{Duration, Instant},
};
use ulid::Ulid;

mod guard;
mod middleware;
mod route;
mod timing;

static SSE_POOL: Lazy<RwLock<HashMap<Ulid, Sse>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn main(addr: &str) -> Result<(), SaphirError> {
    {
        let _ = SSE_POOL.read().await;
    }

    tokio::spawn(async move {
        let mut interval = tokio::time::interval_at(
            Instant::now() + Duration::from_secs(20),
            Duration::from_secs(5),
        );

        loop {
            interval.tick().await;

            let t = Instant::now();
            trace!(">>> SSE heartbeat");
            let mut to_remove = Vec::new();
            let connections = {
                let mut sse_pool = SSE_POOL.write().await;
                let mut connections = Vec::new();
                trace!("SSE 0\t{}", t.elapsed().as_micros());
                for (id, sse) in (*sse_pool).iter_mut() {
                    debug!(">>> SSE: {}", id);
                    connections.push(sse.heartbeat(*id));
                }
                join_all(connections).await
            };
            trace!("SSE 1\t{}", t.elapsed().as_micros());
            for (id, connection) in connections {
                match connection {
                    Ok(()) => {}
                    Err(_) => {
                        // remove the entry, already closed
                        to_remove.push(id);
                    }
                }
            }
            trace!("SSE 2\t{}", t.elapsed().as_micros());
            {
                let mut sse_pool = SSE_POOL.write().await;
                for id in to_remove {
                    (*sse_pool).remove(&id);
                }
            }
            trace!(">>> SSE {}", t.elapsed().as_micros());
        }
    });

    let server = Server::builder()
        .configure_listener(|l| l.server_name("MyKeyring").interface(addr))
        .configure_middlewares(|m| {
            m.apply(crate::middleware::LogMiddleware::new(), vec!["/"], None)
                .apply(crate::middleware::TimingMiddleware::new(), vec!["/"], None)
        })
        .configure_router(|r| {
            r.controller(crate::route::MyKeyringApiController {})
                .controller(A)
        })
        .build();
    // if let Err(e) = server.run().await {
    //     eprintln!("server error: {}", e);
    // }
    server.run().await
}

#[derive(Debug)]
struct Sse {
    sender: saphir::hyper::body::Sender,
    last_heartbeat: Instant,
    request_id: RequestId,
}

impl Sse {
    pub async fn heartbeat(&mut self, id: Ulid) -> (Ulid, Result<(), impl Responder>) {
        if self.last_heartbeat + Duration::from_secs(10) < Instant::now() {
            self.last_heartbeat = Instant::now();
            (id, self.send("", "").await)
        } else {
            (id, Ok(()))
        }
    }

    pub async fn send(&mut self, id: &str, msg: &str) -> Result<(), SaphirError> {
        self.sender
            .send_data(Bytes::from(format!("{}:{}\n\n", id, msg)))
            .await
            .map_err(|e| SaphirError::Other(e.to_string()))
    }
}

struct A;

#[controller(prefix = "api", version = 2)]
impl A {
    #[post("/b")]
    async fn post_b(&self, request_id: Json<RequestId>) -> (u16, Json<RequestId>) {
        (200, request_id)
    }
}
