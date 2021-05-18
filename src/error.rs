use std::{error::Error as StdError, fmt::Display};

#[derive(Debug)]
pub enum Error {
    Decode(std::io::Error),
    TimedOut(),
    InvalidBody(BoxError),
    NoneValue(String),
    InvalidHeaderValue(http::header::InvalidHeaderValue),
    StatusError(http::StatusCode),
    IoError(std::io::Error),
    HyperError(BoxError),
    HttpError(BoxError),
}


impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("")
    }
}

impl StdError for Error { }


pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

#[allow(unused)]
pub(crate) fn decode_io(e: std::io::Error) -> Error {
    if e.get_ref().map(|r| r.is::<Error>()).unwrap_or(false) {
        *e.into_inner()
            .expect("io::Error::get_ref was Some(_)")
            .downcast::<Error>()
            .expect("StdError::is() was true")
    } else {
        Error::Decode(e)
    }
}

impl From<http::header::InvalidHeaderValue> for Error {
    #[inline(always)]
    fn from(error: http::header::InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(error)
    }
}

impl From<std::io::Error> for Error {
    #[inline(always)]
    fn from(error: std::io::Error) -> Self {
      Self::IoError(error)
    }
}

impl From<hyper::Error> for Error {
    #[inline(always)]
    fn from(error: hyper::Error) -> Self {
      Self::HyperError(error.into())
    }
}

impl From<http::Error> for Error {
    #[inline(always)]
    fn from(error: http::Error) -> Self {
      Self::HttpError(error.into())
    }
}