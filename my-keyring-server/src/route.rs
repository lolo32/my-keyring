use actix_web::web;
use log::debug;

mod api;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1").configure(self::api::config));
}

// pub struct MyKeyringApiController {}

// impl MyKeyringApiController {
//     async fn test(&self, _req: Request) -> (u16, Json<B>) {
//         (
//             200,
//             Json(B {
//                 a: Ulid::new().into(),
//             }),
//         )
//     }
//
//     #[inline]
//     async fn get_api_id_response_ulid(
//         &self,
//         mut req: Request,
//     ) -> Result<impl Responder, SaphirError> {
//         let mut timing = extract_timing(&req);
//
//         let id = {
//             let instant = Instant::now();
//             let id = read_param::<Ulid>(&mut req, "id")?;
//             timing.add_timing("param", instant.elapsed(), None);
//             id.into()
//         };
//         api::id::response::get_ulid(req, timing, id).await
//     }
//
//     #[inline]
//     async fn post_api_id_response_ulid(
//         &self,
//         mut req: Request,
//     ) -> Result<impl Responder, SaphirError> {
//         let mut timing = extract_timing(&req);
//
//         let id = {
//             let instant = Instant::now();
//             let id = read_param::<Ulid>(&mut req, "id")?;
//             timing.add_timing("guard", instant.elapsed(), None);
//             id.into()
//         };
//
//         api::id::response::post_ulid(req, timing, id).await
//     }
//
//     #[inline]
//     async fn post_api_id_save(&self, mut req: Request) -> Result<impl
// Responder, SaphirError> {         let mut timing = extract_timing(&req);
//
//         let instant = Instant::now();
//         let response_id = read_body::<String>(&mut req).await?;
//         println!("{:?}", response_id);
//         timing.add_timing("body", instant.elapsed(), None);
//
//         Ok(501)
//     }
// }
//
// impl Controller for MyKeyringApiController {
//     const BASE_PATH: &'static str = "/api/v1";
//
//     fn handlers(&self) -> Vec<ControllerEndpoint<Self>>
//     where
//         Self: Sized,
//     {
//         EndpointsBuilder::new()
//             .add(Method::GET, "/test", MyKeyringApiController::test)
//             .add(
//                 Method::POST,
//                 "/save",
//                 MyKeyringApiController::post_api_id_save,
//             )
//             .add_with_guards(
//                 Method::GET,
//                 "/id/response/<id>",
//                 MyKeyringApiController::get_api_id_response_ulid,
//                 |g| {
//                     g.apply(crate::guard::GuardCheckUlid::new((
//                         "id",
//                         StatusCode::NOT_FOUND,
//                     )))
//                 },
//             )
//             .add_with_guards(
//                 Method::POST,
//                 "/id/response/<id>",
//                 MyKeyringApiController::post_api_id_response_ulid,
//                 |g| {
//                     g.apply(crate::guard::GuardCheckUlid::new((
//                         "id",
//                         StatusCode::NOT_FOUND,
//                     )))
//                 },
//             )
//             .build()
//     }
// }
