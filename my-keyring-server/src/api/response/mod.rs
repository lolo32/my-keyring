use crate::SSE_POOL;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::str::FromStr;
use ulid::Ulid;

async fn handle_post_ulid(id: &str) -> Option<Response<Body>> {
    // POST /api/response/[Ulid]

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap();

    let id = Ulid::from_str(id);
    if id.is_err() {
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return Some(response);
    }
    let id = id.unwrap();

    {
        let senders = SSE_POOL.read().await;
        if !(*senders).contains_key(&id) {
            return None;
        }
    }

    let mut sse = {
        let mut senders = SSE_POOL.write().await;
        (*senders).remove(&id).unwrap()
    };

    let sent = sse.send("auth", &id.datetime().to_string()).await;

    // If the client_id (Ulid) is valid
    *response.body_mut() = Body::from(format!(
        "id: {}\t{}\t{:?}",
        id.datetime(),
        Ulid::new().to_string(),
        sent
    ));

    Some(response)
}

pub async fn dynamic_handle(uri: &[&str], req: Request<Body>) -> Option<Response<Body>> {
    match (req.method(), uri.get(0)) {
        (&Method::POST, Some(id)) if uri.get(1).is_none() => handle_post_ulid(id).await,

        _ => None,
    }
}
