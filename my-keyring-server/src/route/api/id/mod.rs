use my_keyring_shared::request::RequestId;
use rand::random;
use saphir::prelude::*;
use tokio::time::Instant;

use crate::{Sse, SSE_POOL};

pub mod response;

pub async fn request(mut req: Request) -> Result<impl Responder, SaphirError> {
    let request_id = req
        .body_mut()
        .take_as::<Json<RequestId>>()
        .await
        .map(|x| Json(x))?
        .into_inner();
    let (sender, body) = saphir::hyper::body::Body::channel();
    // Generate a random u128 number, that will be used in response URI
    let response_id = (random::<u64>(), random::<u64>()).into();

    // TODO: send push notif

    {
        (*SSE_POOL.write().await).insert(
            response_id,
            Sse {
                sender,
                last_heartbeat: Instant::now(),
                request_id,
            },
        );
    }

    let res = Builder::new()
        .status(StatusCode::OK)
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONTENT_ENCODING, "entity")
        .header(header::CONTENT_TYPE, "text/event-stream")
        .body(body);

    Ok(res)
}
