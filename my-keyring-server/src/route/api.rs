use actix_web::{http::StatusCode, web, HttpRequest, Responder};

use crate::timing::{extract_timing, new_responder};

pub mod id;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/healthz").route(web::get().to(healthz)))
        .service(web::scope("/id").configure(self::id::config));
}

#[inline]
async fn healthz(req: HttpRequest) -> impl Responder {
    let mut timing = extract_timing(&req);
    new_responder(timing, StatusCode::OK).body("Up")
}
