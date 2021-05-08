use std::collections::HashMap;

use futures::future::join_all;
use log::{debug, info, trace};
use my_keyring_shared::{request::PushRequest, RUSTC_VERSION};
use once_cell::sync::Lazy;
use saphir::prelude::*;
use tokio::{
    sync::RwLock,
    time::{Duration, Instant},
};

mod guard;
mod middleware;
mod route;
mod sse;
mod timing;
mod utils;

static SSE_POOL: Lazy<RwLock<HashMap<u128, sse::Sse>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn main(addr: &str) -> Result<(), SaphirError> {
    {
        let _ = SSE_POOL.read().await;
    }

    info!("Built with: {}", RUSTC_VERSION);

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
                let mut sse_pool = SSE_POOL.write().await;
                let mut connections = Vec::new();
                for (id, sse) in (*sse_pool).iter_mut() {
                    debug!(">>> SSE: {}", id);
                    connections.push(sse.heartbeat(*id));
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
                let mut sse_pool = SSE_POOL.write().await;
                for id in to_remove {
                    (*sse_pool).remove(&id);
                }
            }
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

struct A;

#[controller(prefix = "api", version = 2)]
impl A {
    #[get("/b")]
    async fn get_b(&self, request_id: Json<PushRequest>) -> (u16, Json<PushRequest>) {
        (200, request_id)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn my_test() {
        let a = vec![0xFF, 0x00, b'a'];
        let js = serde_json::json!({
            "name": "toto",
            "value": a
        });
        println!("{}", js);
        assert!(false);
    }
}
