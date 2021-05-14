use actix_web::web;

mod api;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1").configure(self::api::config));
}
