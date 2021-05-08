#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    Client(u16, String),
    Server(u16, String),
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Self::Hyper(err)
    }
}

impl From<(hyper::StatusCode, &str)> for Error {
    fn from((status, message): (hyper::StatusCode, &str)) -> Self {
        if status.is_client_error() {
            Self::Client(status.as_u16(), message.to_owned())
        } else {
            Self::Server(status.as_u16(), message.to_owned())
        }
    }
}
