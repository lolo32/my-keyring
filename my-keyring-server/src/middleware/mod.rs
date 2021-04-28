use log::debug;
use saphir::{http::HeaderValue, prelude::*};
use tokio::time::Instant;

use crate::timing::Timing;

pub struct LogMiddleware {}

#[middleware]
impl LogMiddleware {
    pub fn new() -> Self {
        Self {}
    }

    async fn next(
        &self,
        ctx: HttpContext,
        chain: &dyn MiddlewareChain,
    ) -> Result<HttpContext, SaphirError> {
        let instant = Instant::now();
        debug!(">>> LogMiddleware");

        println!(
            "new request on path: {}",
            ctx.state.request_unchecked().uri().path()
        );
        let ctx = chain.next(ctx).await?;
        println!(
            "new response with status: {}\t{}µs",
            ctx.state.response_unchecked().status(),
            instant.elapsed().as_micros()
        );

        debug!("<<< LogMiddleware");
        Ok(ctx)
    }
}

pub struct TimingMiddleware {}

#[middleware]
impl TimingMiddleware {
    pub fn new() -> Self {
        Self {}
    }

    async fn next(
        &self,
        mut ctx: HttpContext,
        chain: &dyn MiddlewareChain,
    ) -> Result<HttpContext, SaphirError> {
        let instant = Instant::now();
        debug!(">>> TimingMiddleware");

        ctx.state
            .request_unchecked_mut()
            .extensions_mut()
            .insert(Timing::new());

        let mut ctx = chain.next(ctx).await?;

        let res = ctx.state.response_unchecked_mut();
        let mut timing = match res.extensions_mut().get_mut::<Timing>().cloned() {
            Some(t) => t,
            None => Timing::new(),
        };
        timing.add_timing("req", instant.elapsed(), None);

        let header_value = timing.to_string();
        res.headers_mut().append(
            "Server-Timing",
            HeaderValue::from_str(&header_value).unwrap(),
        );

        debug!("<<< TimingMiddleware");
        Ok(ctx)
    }
}
