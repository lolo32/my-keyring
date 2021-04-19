use crate::{Sse, SSE_POOL};
use hyper::{header::HeaderValue, Body, Request, Response, StatusCode};
use my_keyring_shared::request::RequestId;

mod response;

pub async fn request(req: Request<Body>) -> Response<Body> {
    let (sender, body) = Body::channel();
    let b = hyper::body::to_bytes(req).await.expect("body");
    let request_id: RequestId = serde_json::from_slice(&b).expect("RequestId");

    // TODO: send push notif

    let mut senders = SSE_POOL.write().await;
    (*senders).insert(request_id.client_id, Sse { sender });

    Response::builder()
        .status(StatusCode::OK)
        .header("Cache-Control", HeaderValue::from_static("no-cache"))
        .header(
            "Content-Type",
            HeaderValue::from_static("text/event-stream"),
        )
        .body(body)
        .unwrap()
}

pub async fn dynamic_handle(uri: &[&str], req: Request<Body>) -> Option<Response<Body>> {
    if uri.get(0) == Some(&"response") {
        return response::dynamic_handle(&uri[1..], req).await;
    }

    None
}
