use actix_web::{http::StatusCode, rt::time::Instant, web, HttpRequest, Responder};
use log::debug;
use my_keyring_shared::{request::PushRequest, security::SipHash};
use ulid::Ulid;

use crate::{
    sse::{Sse, SseData},
    timing::{extract_timing, new_responder},
    SSE_POOL,
};

pub mod response;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/request").route(web::post().to(request)))
        .service(web::resource("/save").route(web::post().to(save)))
        .service(web::scope("/response").configure(self::response::config));
}

/// POST /api/v1/id/save
/// TODO
async fn save(req: HttpRequest, response_id: String) -> impl Responder {
    let mut timing = extract_timing(&req);

    println!("{:?}", response_id);

    new_responder(timing, StatusCode::NOT_IMPLEMENTED).finish()
}

/// POST /api/v1/id/request
///
/// Used to request for a push authentication request
async fn request(req: HttpRequest, push_request: web::Json<PushRequest>) -> impl Responder {
    let mut timing = extract_timing(&req);

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
            response_url_sip_hash.hash.into(),
            Sse {
                sender: None,
                last_heartbeat: Instant::now(),
                data: SseData::PushRequest(push_request.into_inner(), response_url_sip_hash.keys),
            },
        );
        timing.add_timing("ssew", instant.elapsed(), None);
    }

    // Generate then return the Server-Sent-Event response to the client
    new_responder(timing, StatusCode::OK).body(Ulid::from(response_url_sip_hash.keys).to_string())
}
