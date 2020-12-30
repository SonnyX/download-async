pub use http;
pub use hyper::body::Body;
mod progress;
pub use progress::Progress;
mod download;
pub use download::download;
mod error;
pub use error::StatusError;
mod dns;
pub use dns::SocketAddrs;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
