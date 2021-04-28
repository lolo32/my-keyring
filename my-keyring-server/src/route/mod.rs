use saphir::prelude::*;
use tokio::time::Instant;
use ulid::Ulid;

use crate::timing::Timing;

mod api;

pub struct MyKeyringApiController {}

impl MyKeyringApiController {
    #[inline]
    async fn healthz(&self, _req: Request) -> impl Responder {
        StatusCode::OK
    }

    #[inline]
    async fn post_api_id_request(&self, req: Request) -> Result<impl Responder, SaphirError> {
        api::id::request(req).await
    }

    #[inline]
    async fn post_api_id_response_ulid(
        &self,
        mut req: Request,
    ) -> Result<impl Responder, SaphirError> {
        let instant = Instant::now();
        let id = req
            .captures_mut()
            .remove("id")
            .map(|p| p.parse::<Ulid>())
            .transpose()
            .map_err(|_| SaphirError::InvalidParameter("id".to_string(), false))?
            .ok_or_else(|| SaphirError::MissingParameter("id".to_string(), false))?;

        req.extensions_mut()
            .get_mut::<Timing>()
            .unwrap()
            .add_timing("guard", instant.elapsed(), None);

        api::id::response::post_ulid(req, id).await
    }
}

impl Controller for MyKeyringApiController {
    const BASE_PATH: &'static str = "/api/v1";

    fn handlers(&self) -> Vec<ControllerEndpoint<Self>>
    where
        Self: Sized,
    {
        EndpointsBuilder::new()
            .add(Method::GET, "/healthz", MyKeyringApiController::healthz)
            .add(
                Method::POST,
                "/id/request",
                MyKeyringApiController::post_api_id_request,
            )
            .add_with_guards(
                Method::POST,
                "/id/response/<id>",
                MyKeyringApiController::post_api_id_response_ulid,
                |g| {
                    g.apply(crate::guard::GuardCheckUlid::new((
                        "id",
                        StatusCode::NOT_FOUND,
                    )))
                },
            )
            .build()
    }
}
