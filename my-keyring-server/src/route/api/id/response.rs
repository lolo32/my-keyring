use std::borrow::BorrowMut;

use actix_web::{
    http::{header, StatusCode},
    rt::time::Instant,
    web, HttpRequest, Responder,
};
use log::{debug, warn};
use my_keyring_shared::{
    request::{PushRequest, ResponseId},
    security::{SipHash, SipHashKeys},
};
use ulid::Ulid;

use crate::{
    sse::SseData,
    stream::SseStream,
    timing::{extract_timing, new_responder},
    SSE_POOL,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/{id}")
            .route(web::get().to(get_ulid))
            .route(web::post().to(post_ulid)),
    );
}

#[inline]
fn get_sse_data(sse_data: &SseData) -> Option<(&PushRequest, SipHashKeys)> {
    match sse_data {
        SseData::PushRequest(push_request, keys) => Some((push_request, *keys)),
        _ => None,
    }
}

/// POST /api/v1/id/response/[<id>>]
///
/// Process the response from the device that have the response
pub async fn post_ulid(
    req: HttpRequest,
    id: web::Path<String>,
    response_id: web::Json<ResponseId>,
) -> actix_web::Result<impl Responder> {
    debug!("a");
    let mut timing = extract_timing(&req);

    let id = match Ulid::from_string(&id) {
        Ok(id) => id.0,
        Err(e) => {
            debug!("Err Ulid: '{:?}'\t{:?}", id, e);
            return Ok(new_responder(timing, StatusCode::NOT_FOUND).finish());
        }
    }
    .into();

    // Check if the id is known and valid
    let (request_client_id, keys) = {
        let instant = Instant::now();
        let senders = SSE_POOL.read().await;
        let request_client_id = (*senders).get(&id);
        timing.add_timing("get", instant.elapsed(), None);

        match request_client_id {
            Some(sse) => {
                if let Some((pr, k)) = get_sse_data(&sse.data) {
                    (pr.clone(), k)
                } else {
                    return Ok(new_responder(timing, StatusCode::CONFLICT).finish());
                }
            }
            None => {
                warn!("SSE stream id does not exists");
                return Ok(new_responder(timing, StatusCode::NOT_FOUND).finish());
            }
        }
    };

    debug!("b");
    let sip_hash = SipHash::new_with_keys(keys, &request_client_id.push_id.as_bytes()[1..]);
    debug!("sip_hash: {:?}\t{}", sip_hash, response_id.client_id);

    if sip_hash.hash != response_id.client_id.0 {
        return Ok(new_responder(timing, StatusCode::FORBIDDEN).finish());
    }

    let mut sse = {
        let instant = Instant::now();
        let sse = {
            let mut senders = SSE_POOL.write().await;
            (*senders).remove(&id).unwrap()
        };
        timing.add_timing("rem", instant.elapsed(), None);
        sse
    };

    let sent = sse.send("auth", &id.to_string())?;

    // If the client_id (Ulid) is valid
    Ok(new_responder(timing, StatusCode::OK)
        .status(StatusCode::OK)
        .body(format!("id: {}\t{}\t{:?}", id, Ulid::new(), sent)))
}

/// GET /api/v1/id/response/[<id>]
///
/// Endpoint for SSE, the browser needs to know the `id`, that is made from:
/// - `push_token` already known by this client who asked the push
/// - `keys` that is returned by the previous step asking the push
///
/// Listening to the SSE send the push to the terminal and the associated
/// encrypted data.
pub async fn get_ulid(
    req: HttpRequest,
    id: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    debug!("aaa");
    let mut timing = extract_timing(&req);
    let id = match Ulid::from_string(&id) {
        Ok(id) => id.0,
        Err(e) => {
            debug!("Err Ulid: '{:?}'\t{:?}", id, e);
            return Ok(new_responder(timing, StatusCode::NOT_FOUND).finish());
        }
    }
    .into();
    // Retrieve the push_id associated with this `id`
    let push_id = {
        let instant = Instant::now();
        let push_data = (*SSE_POOL.read().await).get(&id).map(|sse| {
            (
                // Is there already a sse listener
                sse.sender.is_some(),
                // Check the SseData type, it must be a `PushRequest`
                get_sse_data(&sse.data).map(|(pr, _k)| pr.push_id.clone()),
            )
        });
        timing.add_timing("sser", instant.elapsed(), None);

        if let Some((already_listener, push_id)) = push_data {
            // If a previous request requested this `id`
            if already_listener {
                debug!("Trying to connect to an already accepted SSE");
                // A listener already registered, it's not allowed to listen 2 times
                return Ok(new_responder(timing, StatusCode::FORBIDDEN).finish());
            }
            // Returns if the `push_id` cannot be retrieved
            match push_id {
                // The requested `sse_id` is a `PushRequest`, so continue with the `push_id`
                Some(push_id) => push_id,
                // The requested `sse_id` is a different type, so returns an error
                None => {
                    debug!("Trying to connect to an ID that does not contains any listener");
                    return Ok(new_responder(timing, StatusCode::CONFLICT).finish());
                }
            }
        } else {
            // This `id` was not registered
            debug!("Not configured SSE id");
            return Ok(new_responder(timing, StatusCode::NOT_FOUND).finish());
        }
    };

    // Generate a Sender to send body later in the code
    let body = {
        let (sender, body) = SseStream::new();
        let instant = Instant::now();

        // Register the `sender` and update `last_heartbeat` information
        (*SSE_POOL.write().await).entry(id).and_modify(|sse| {
            sse.last_heartbeat = Instant::now();
            sse.sender = Some(sender);
        });
        timing.add_timing("ssew", instant.elapsed(), None);

        // Returns the body
        body
    };
    // Now all authentication have succeeded

    // Retrieve the keys and the associated encrypted data
    let (keys, data) = {
        (*SSE_POOL.read().await)
            .get(&id)
            .map(|sse| {
                get_sse_data(&sse.data)
                    .map(|(pr, k)| (k, pr.encrypted_data.clone().unwrap()))
                    .unwrap()
            })
            .unwrap()
    };
    // Remove the encrypted data from the server
    {
        (*SSE_POOL.write().await).entry(id).and_modify(|sse| {
            if let SseData::PushRequest(push_request, _keys) = sse.data.borrow_mut() {
                push_request.encrypted_data = None
            };
        });
    }

    // TODO: send push notification
    debug!(
        "Sending push to: {}\nKeys: {}\nData: {:?}",
        push_id, keys, data
    );

    // Generate then return the Server-Sent-Event response to the client
    Ok(new_responder(timing, StatusCode::OK)
        .insert_header(header::CacheControl(vec![header::CacheDirective::NoCache]))
        .insert_header(header::ContentEncoding::Identity)
        .insert_header((header::CONTENT_TYPE, "text/event-stream"))
        .streaming(body))
}
