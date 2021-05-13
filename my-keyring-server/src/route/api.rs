use actix_web::web;

pub mod id;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/healthz").route(web::get().to(healthz)))
        .service(web::scope("/id").configure(self::id::config));
}

#[inline]
async fn healthz() -> &'static str {
    "Up"
}
