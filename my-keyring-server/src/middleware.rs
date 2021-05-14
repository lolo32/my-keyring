use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::{
    dev::{MessageBody, ResponseBody, Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    http::HeaderValue,
    rt::time::Instant,
    HttpMessage,
};
use futures::future::{ok, Ready};
use futures_core::ready;

use crate::timing::Timing;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct TimingMiddleware;

// Middleware factory is `Transform` trait from actix-service crate
impl<S, B> Transform<S, ServiceRequest> for TimingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    type Response = ServiceResponse<ResponseBody<B>>;
    type Error = S::Error;
    type Transform = TimingService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TimingService { service })
    }
}

impl Default for TimingMiddleware {
    fn default() -> Self {
        Self
    }
}

pub struct TimingService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TimingService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    type Response = ServiceResponse<ResponseBody<B>>;
    type Error = Error;
    type Future = TimingResponse<S, B>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let instant = Instant::now();
        req.extensions_mut().insert(Timing::new());
        TimingResponse {
            fut: self.service.call(req),
            timing: instant,
            _phantom: PhantomData,
        }
    }
}

#[pin_project::pin_project]
pub struct TimingResponse<S, B>
where
    B: MessageBody,
    S: Service<ServiceRequest>,
{
    #[pin]
    fut: S::Future,
    timing: Instant,
    _phantom: PhantomData<B>,
}

impl<S, B> Future for TimingResponse<S, B>
where
    B: MessageBody,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<ResponseBody<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match ready!(this.fut.poll(cx)) {
            Ok(mut res) => {
                let mut timing = match res.response().extensions().get::<Timing>() {
                    Some(t) => t.clone(),
                    None => Timing::new(),
                };
                timing.add_timing("tot", this.timing.elapsed(), None);
                res.response_mut().headers_mut().insert(
                    "Server-Timing".parse().unwrap(),
                    HeaderValue::from_str(&timing.to_string()).unwrap(),
                );

                Poll::Ready(Ok(res.map_body(|_head, body| ResponseBody::Body(body))))
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
