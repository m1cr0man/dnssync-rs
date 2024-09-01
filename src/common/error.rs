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
    // TODO rename to ResponseSnafu
    #[snafu(display("{message}: {source}"))]
    FrontendError {
        message: String,
        source: Box<dyn std::error::Error>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
