use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use hyper::{
    body::Bytes,
    service::Service,
    {Body, Method, Request, Response, Server, StatusCode},
};
use std::collections::HashMap;
use tokio::{
    sync::RwLock,
    time::{Duration, Instant},
};
use ulid::Ulid;

mod api;

lazy_static::lazy_static! {
    static ref SSE_POOL: RwLock<HashMap<Ulid, Sse>> = RwLock::const_new(HashMap::new());
}

pub async fn main(addr: &SocketAddr) {
    lazy_static::initialize(&SSE_POOL);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;
            let t = Instant::now();
            println!("<<< SSE heartbeat");
            let mut to_remove = Vec::new();
            {
                let mut sse_pool = SSE_POOL.write().await;
                for (id, sse) in (*sse_pool).iter_mut() {
                    match sse.heartbeat().await {
                        Ok(()) => {}
                        Err(_) => {
                            // remove the entry, already closed
                            to_remove.push(*id);
                        }
                    }
                }
            }
            {
                let mut sse_pool = SSE_POOL.write().await;
                for id in to_remove {
                    (*sse_pool).remove(&id);
                }
            }
            println!(">>> {}", t.elapsed().as_micros());
        }
    });
    let server = Server::bind(&addr).serve(MakeSvc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

struct MakeSvc;

impl<T> Service<T> for MakeSvc {
    type Response = Svc;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: T) -> Self::Future {
        let fut = async move { Ok(Svc) };
        Box::pin(fut)
    }
}

struct Svc;

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let instant = Instant::now();
        let method = req.method().clone();
        let uri = req.uri().clone();

        Box::pin(async move {
            let res = handle(req).await.unwrap_or_else(|| {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap()
            });

            println!(
                "{} {} {} {}µs",
                method,
                uri,
                res.status(),
                instant.elapsed().as_micros()
            );

            Ok(res)
        })
    }
}

async fn handle(req: Request<Body>) -> Option<Response<Body>> {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let res = match (&method, uri.path()) {
        (&Method::POST, "/api/request") => api::request(req).await,

        _ => {
            let uri = uri.path().replace("%2f", "/").replace("%2F", "/");
            let uri: Vec<&str> = uri.as_str().split('/').collect();
            let uri = uri[1..].to_vec();

            if uri.get(0) == Some(&"api") {
                return api::dynamic_handle(&uri[1..], req).await;
            }
            return None;
        }
    };

    Some(res)
}

#[derive(Debug)]
struct Sse {
    sender: hyper::body::Sender,
}

impl Sse {
    pub async fn heartbeat(&mut self) -> Result<(), hyper::Error> {
        self.send("", "").await
    }

    pub async fn send(&mut self, id: &str, msg: &str) -> Result<(), hyper::Error> {
        self.sender
            .send_data(Bytes::from(format!("{}:{}\n\n", id, msg)))
            .await
    }
}
