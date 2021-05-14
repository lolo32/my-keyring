use core::fmt;

use actix_web::{http::StatusCode, ResponseError};

#[derive(Debug)]
pub enum Error {
    NotConnected,
    SseClosed,
    Timeout,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::SseClosed => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotConnected => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Timeout => StatusCode::REQUEST_TIMEOUT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
