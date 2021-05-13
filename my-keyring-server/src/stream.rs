use std::{
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::web::Bytes;
use futures::Stream;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct SseStream(Receiver<Bytes>);

impl SseStream {
    pub fn new() -> (Sender<Bytes>, Self) {
        let (tx, rx) = channel(2);
        (tx, Self(rx))
    }
}

impl Stream for SseStream {
    type Item = std::io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_recv(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
