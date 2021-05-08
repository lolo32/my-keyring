use std::{net::SocketAddr, time::SystemTime};

use humantime::format_rfc3339_millis;
use saphir::{http::HeaderValue, prelude::*};
use serde::Serialize;
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
        #[derive(Debug, Serialize)]
        struct Log {
            remote_addr: String,

            request_method: String,
            request_uri: String,
            request_version: String,
            request_time_utc: String,

            response_duration_ms: f64,
            response_status: u16,
        }

        let instant = Instant::now();

        let mut log = {
            let req = ctx.state.request_unchecked();

            Log {
                remote_addr: req
                    .peer_addr()
                    .cloned()
                    .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 0)))
                    .ip()
                    .to_string(),
                request_method: req.method().to_string(),
                request_uri: req.uri().to_string(),
                request_version: format!("{:?}", req.version()),
                request_time_utc: format_rfc3339_millis(SystemTime::now()).to_string(),
                response_duration_ms: 0.,
                response_status: 0,
            }
        };

        println!(
            "new request on path: {}",
            ctx.state.request_unchecked().uri().path()
        );
        let ctx = chain.next(ctx).await?;
        let res = ctx.state.response_unchecked();
        println!(
            "new response with status: {}\t{}µs",
            ctx.state.response_unchecked().status(),
            instant.elapsed().as_micros()
        );

        log.response_status = res.status().as_u16();
        log.response_duration_ms = instant.elapsed().as_secs_f64() * 1000.;
        let json = serde_json::to_string(&log).unwrap();

        eprintln!("{}", json);

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
        timing.add_timing("tot", instant.elapsed(), None);

        let header_value = timing.to_string();
        res.headers_mut().append(
            "Server-Timing",
            HeaderValue::from_str(&header_value).unwrap(),
        );

        Ok(ctx)
    }
}
