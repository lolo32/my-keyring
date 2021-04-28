use log::{info, trace};
use my_keyring_shared::request::ResponseId;
use saphir::prelude::*;
use tokio::time::Instant;
use ulid::Ulid;

use crate::{timing::extract_timing, SSE_POOL};

pub async fn post_ulid(mut req: Request, id: Ulid) -> Result<impl Responder, SaphirError> {
    // POST /api/response/[Ulid]
    info!("post_ulid: {}", id);

    let mut timing = extract_timing(&req);

    let mut instant = Instant::now();
    let request_client_id = {
        let senders = SSE_POOL.read().await;
        if !(*senders).contains_key(&id) {
            info!("SSE stream id does not exists");
            return Ok(Builder::new()
                .status(StatusCode::NOT_FOUND)
                .extension(timing));
        }
        timing.add_timing("contains", instant.elapsed(), None);
        instant = Instant::now();
        (*senders).get(&id).unwrap().request_id
    };
    timing.add_timing("get", instant.elapsed(), None);

    instant = Instant::now();
    let response_id = match req
        .body_mut()
        .take_as::<Json<ResponseId>>()
        .await
        .map(|x| Json(x))
    {
        Ok(j) => j.into_inner(),
        Err(_) => {
            return Ok(Builder::new()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .extension(timing))
        }
    };
    timing.add_timing("serde", instant.elapsed(), None);

    if request_client_id.client_id != response_id.client_id {
        return Ok(Builder::new()
            .status(StatusCode::FORBIDDEN)
            .extension(timing));
    }

    instant = Instant::now();
    let mut sse = {
        let mut senders = SSE_POOL.write().await;
        (*senders).remove(&id).unwrap()
    };
    timing.add_timing("rem", instant.elapsed(), None);

    let sent = sse.send("auth", &id.datetime().to_string()).await;

    // If the client_id (Ulid) is valid
    Ok(Builder::new()
        .status(StatusCode::OK)
        .body(format!(
            "id: {}\t{}\t{:?}",
            id.datetime(),
            Ulid::new().to_string(),
            sent
        ))
        .extension(timing))
}
