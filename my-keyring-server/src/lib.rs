use std::{collections::HashMap, sync::Arc};

use actix_web::{middleware::Logger, App, HttpServer};
use byteorder::BigEndian;
use log::info;
use my_keyring_shared::RUSTC_VERSION;
use tokio::sync::RwLock;
use zerocopy::U128;

use crate::{middleware::TimingMiddleware, sse::sse_maintenance};

mod error;
mod middleware;
mod route;
mod sse;
mod stream;
mod timing;

type SseDataType = Arc<RwLock<HashMap<U128<BigEndian>, sse::Sse>>>;

pub async fn main(addr: &str) -> std::io::Result<()> {
    let sse_pool: SseDataType = Arc::new(RwLock::new(HashMap::new()));

    info!("Built with: {}", RUSTC_VERSION);

    sse_maintenance(sse_pool.clone());

    HttpServer::new(move || {
        App::new()
            .data(sse_pool.clone())
            .wrap(TimingMiddleware::default())
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
