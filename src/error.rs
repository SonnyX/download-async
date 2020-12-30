#[derive(Debug)]
pub struct StatusError {
    status: http::StatusCode
}

impl StatusError {
    pub fn from(status: http::StatusCode) -> Box<Self> {
        Box::new(Self {
            status
        })
    }
}

impl std::error::Error for StatusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The status code returned a {}: {:?}", self.status.as_u16(), self.status.canonical_reason())
    }
}