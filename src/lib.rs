pub use http::{header, HeaderMap, Method, Request, Response, StatusCode, Uri, Version};
pub use hyper::body::Body;
pub mod progress;
pub mod download;
pub mod error;
mod dns;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
