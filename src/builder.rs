use crate::{decoder::Accepts, progress::Progress};
use std::io::Write;
use hyper::body::HttpBody;
use crate::dns::SocketAddrs;
use http::{HeaderValue, header, response::Parts};
use crate::error::Error;

type BoxError = Box<dyn std::error::Error + Send + Sync>;


pub struct Downloader {
  request: Option<http::request::Builder>,
  https_only: bool,
  progress: Option<Box<dyn Progress + Send>>,
  sockets: Option<SocketAddrs>,
  disabled_compression: bool
}

impl Downloader {
  pub fn new() -> Self {
    Self {
      request: Some(http::Request::builder()),
      https_only: true,
      progress: None,
      sockets: None,
      disabled_compression: false
    }
  }

  pub fn use_uri(&mut self, uri: http::Uri) -> &mut Self {
    if let Some(host) = uri.host() {
      let builder = self.request.take().expect("Failed to take request-builder");
      self.request = Some(builder.uri(uri.clone())
                                 .header("host", host));
    } else {
      log::error!("URI {} is not a valid URI", &uri);
    }
    self
  }

  pub fn headers(&mut self) -> Option<&mut http::HeaderMap<http::HeaderValue>> {
    self.request.as_mut().map(|x| x.headers_mut()).flatten()
  }

  pub fn use_sockets(&mut self, sockets: SocketAddrs) -> &mut Self {
    self.sockets = Some(sockets);
    self
  }

  pub fn allow_http(&mut self) -> &mut Self {
    self.https_only = false;
    self
  }

  pub fn use_progress<T: Progress + Send + 'static>(&mut self, progress: T) -> &mut Self {
    self.progress = Some(Box::new(progress));
    self
  }

  pub fn disable_compression(&mut self) -> &mut Self {
    self.disabled_compression = true;
    self
  }

  pub async fn download<T: HttpBody + Send + 'static>(mut self, body: T, to: &mut impl Write) -> Result<Parts, Error>  where T::Data: Send, T::Error: Into<BoxError> {
    if !self.disabled_compression {
      self.headers().ok_or_else(|| Error::NoneValue(format!("")))?.append(header::ACCEPT_ENCODING, HeaderValue::from_str(Accepts::default().as_str().ok_or_else(|| Error::NoneValue(format!("Couldn't unwrap Accepts")))?)?);
    }
    let body = self.request.take().expect("Failed to take request-builder").body(body)?;
    crate::download::download(body, to, self.https_only, &mut self.progress, self.sockets).await
  }
}