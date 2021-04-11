pub use http;
mod progress;
mod download;
mod error;
mod dns;
mod builder;

pub use builder::Downloader;

pub use dns::SocketAddrs;
pub use error::StatusError;
pub use hyper::body::Body;
pub use progress::Progress;