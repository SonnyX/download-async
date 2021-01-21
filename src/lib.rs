pub use http;
mod progress;
mod download;
mod error;
mod dns;

pub use dns::SocketAddrs;
pub use download::download;
pub use error::StatusError;
pub use hyper::body::Body;
pub use progress::Progress;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
