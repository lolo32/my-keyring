use actix_web::web;
use log::debug;

mod api;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1").configure(self::api::config));
}
