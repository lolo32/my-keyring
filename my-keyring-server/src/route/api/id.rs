use log::debug;
use saphir::prelude::*;
use tokio::time::Instant;

use my_keyring_shared::request::PushRequest;

use crate::sse::SseData;
use crate::{
    sse::Sse,
    timing::{new_responder, Timing},
    SSE_POOL,
};
use my_keyring_shared::security::SipHash;
use ulid::Ulid;

pub mod response;

/// POST /api/id/request
///
/// Used to request for a push authentication request
pub async fn request(
    _req: Request,
    mut timing: Timing,
    push_request: PushRequest,
) -> Result<impl Responder, SaphirError> {
    // Generate a hash with random keys based on the `push_id` that are known
    // by the two sides (requester and remote) and are not communicated to the
    // remote.
    let response_url_sip_hash = SipHash::new(push_request.push_id.as_bytes());

    debug!(
        "UlidHash: {}\nAuthToken: {}",
        Ulid::from(response_url_sip_hash.hash),
        Ulid::from(
            SipHash::new_with_keys(
                response_url_sip_hash.keys,
                &push_request.push_id.as_bytes()[1..]
            )
            .hash
        ),
    );

    // Store the response_url_sip_hash and the information for later use
    {
        let instant = Instant::now();
        (*SSE_POOL.write().await).insert(
            response_url_sip_hash.hash,
            Sse {
                sender: None,
                last_heartbeat: Instant::now(),
                data: SseData::PushRequest(push_request, response_url_sip_hash.keys),
            },
        );
        timing.add_timing("ssew", instant.elapsed(), None);
    }

    // Generate then return the Server-Sent-Event response to the client
    Ok(new_responder(timing)
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Ulid::from(response_url_sip_hash.keys).to_string()))
}
