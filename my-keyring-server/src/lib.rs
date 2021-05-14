use std::{collections::HashMap, sync::Arc};

use actix_web::{
    dev::Service, http::HeaderValue, middleware::Logger, rt::time::Instant, App, HttpMessage,
    HttpServer,
};
use byteorder::BigEndian;
use futures::future::join_all;
use log::{debug, info, trace};
use my_keyring_shared::RUSTC_VERSION;
use tokio::{sync::RwLock, time::Duration};
use zerocopy::U128;

use crate::timing::Timing;

mod error;
mod route;
mod sse;
mod stream;
mod timing;

type SseDataType = Arc<RwLock<HashMap<U128<BigEndian>, sse::Sse>>>;

pub async fn main(addr: &str) -> std::io::Result<()> {
    let sse_pool: SseDataType = Arc::new(RwLock::new(HashMap::new()));

    info!("Built with: {}", RUSTC_VERSION);

    let sp = sse_pool.clone();
    tokio::spawn(async move {
        let sse_pool = sp;
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
                let mut sse_pool = sse_pool.write().await;
                for id in to_remove {
                    (*sse_pool).remove(&id);
                }
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .data(sse_pool.clone())
            .wrap_fn(|req, srv| {
                let instant = Instant::now();
                req.extensions_mut().insert(Timing::new());
                let fut = srv.call(req);

                async move {
                    let mut res = fut.await?;
                    let mut timing = match res.response().extensions().get::<Timing>() {
                        Some(t) => t.clone(),
                        None => Timing::new(),
                    };
                    timing.add_timing("tot", instant.elapsed(), None);
                    res.response_mut().headers_mut().insert(
                        "Server-Timing".parse().unwrap(),
                        HeaderValue::from_str(&timing.to_string()).unwrap(),
                    );
                    Ok(res)
                }
            })
            .wrap(Logger::default())
            .configure(self::route::config)
    })
    .bind(addr)?
    .run()
    .await
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
