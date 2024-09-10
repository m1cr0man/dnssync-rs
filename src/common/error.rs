use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{method} {url} failed: {source}"))]
    RequestError {
        url: String,
        method: String,
        source: ureq::Error,
    },
    #[snafu(display("{message}"))]
    ResponseError { message: String },
    #[snafu(display("{message}: {source}"))]
    BackendError {
        message: String,
        source: Box<dyn std::error::Error>,
    },
    #[snafu(display("{message}: {source}"))]
    FrontendError {
        message: String,
        source: Box<dyn std::error::Error>,
    },
    #[snafu(display("{message}"))]
    SyncError { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;
