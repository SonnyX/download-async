use crate::{decoder::Accepts, progress::Progress};
use std::io::Write;
use hyper::body::HttpBody;
use crate::dns::SocketAddrs;
use http::{HeaderValue, header, response::Parts};
use crate::error::Error;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Represents a downloader that can be used to download resources over HTTP/HTTPS.
pub struct Downloader {
  /// The request builder used to build HTTP requests.
  request: Option<http::request::Builder>,
  /// If set to true, only HTTPS URLs will be used.
  https_only: bool,
  /// An optional progress tracker.
  progress: Option<Box<dyn Progress + Send>>,
  /// The list of sockets to use.
  sockets: Option<SocketAddrs>,
  /// If set to true, compression will be disabled.
  disabled_compression: bool
}

impl Downloader {
  /// Creates a new `Downloader`.
  pub fn new() -> Self {
    Self {
      request: Some(http::Request::builder()),
      https_only: true,
      progress: None,
      sockets: None,
      disabled_compression: false
    }
  }

  /// Sets the URI to download from.
  ///
  /// # Arguments
  ///
  /// * `uri` - The URI to download from.
  ///
  /// # Examples
  ///
  /// ```
  /// extern crate tokio;
  /// extern crate download_async;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let uri = download_async::http::Uri::from_static("https://www.example.com");
  ///   let mut downloader = download_async::Downloader::new();
  ///   downloader.use_uri(uri);
  ///   let mut buffer = vec![];
  ///   let response = downloader.download(download_async::Body::empty(), &mut buffer).await;
  /// }
  /// ```
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
  
  /// Gets a mutable reference to the headers map for the request.
  pub fn headers(&mut self) -> Option<&mut http::HeaderMap<http::HeaderValue>> {
    self.request.as_mut().map(|x| x.headers_mut()).flatten()
  }

  /// Sets the `SocketAddrs` to use for the request.
  ///
  /// # Arguments
  ///
  /// * `sockets` - The `SocketAddrs` to use for the request.
  pub fn use_sockets(&mut self, sockets: SocketAddrs) -> &mut Self {
    self.sockets = Some(sockets);
    self
  }

  /// Allows HTTP requests in addition to HTTPS requests.
  ///
  /// # Examples
  ///
  /// ```
  /// extern crate tokio;
  /// extern crate download_async;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let uri = download_async::http::Uri::from_static("https://www.example.com");
  ///   let mut downloader = download_async::Downloader::new();
  ///   downloader.use_uri(uri);
  ///   downloader.allow_http();
  ///   let mut buffer = vec![];
  ///   let response = downloader.download(download_async::Body::empty(), &mut buffer).await;
  /// }
  /// ```
  pub fn allow_http(&mut self) -> &mut Self {
    self.https_only = false;
    self
  }

  pub fn use_progress<T: Progress + Send + 'static>(&mut self, progress: T) -> &mut Self {
    self.progress = Some(Box::new(progress));
    self
  }

  /// Sends the server the appropriate headers to prevent response compression.
  ///
  /// # Examples
  ///
  /// ```
  /// extern crate tokio;
  /// extern crate download_async;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let uri = download_async::http::Uri::from_static("https://www.example.com");
  ///   let mut downloader = download_async::Downloader::new();
  ///   downloader.use_uri(uri);
  ///   downloader.disable_compression();
  ///   let mut buffer = vec![];
  ///   let response = downloader.download(download_async::Body::empty(), &mut buffer).await;
  /// }
  /// ```
  pub fn disable_compression(&mut self) -> &mut Self {
    self.disabled_compression = true;
    self
  }

  /// An async method to download a resource and write it to a writer
  ///
  /// # Arguments
  ///
  /// * `self` - The `Downloader` instance to use for the request
  /// * `body` - The request body
  /// * `to` - A mutable reference to the writer to write the downloaded data to
  ///
  /// # Returns
  ///
  /// A Result containing `Parts` if successful, or an `Error` if there was an issue with the download
  ///
  /// # Generic Parameters
  ///
  /// * `T` - The type of `HttpBody` to use for the request
  ///
  /// # Constraints
  ///
  /// * `T` must implement `HttpBody`, `Send`, and `'static`
  /// * `T::Data` must implement `Send`
  /// * `T::Error` must implement `Into<BoxError>`
  ///
  /// # Examples
  ///
  /// ```
  /// extern crate tokio;
  /// extern crate download_async;
  ///
  /// #[tokio::main]
  /// async fn main() {
  ///   let uri = download_async::http::Uri::from_static("https://www.example.com");
  ///   let mut downloader = download_async::Downloader::new();
  ///   downloader.use_uri(uri);
  ///   downloader.allow_http();
  ///   let mut buffer = vec![];
  ///   let response = downloader.download(download_async::Body::empty(), &mut buffer).await;
  /// }
  /// ```
  pub async fn download<T: HttpBody + Send + 'static>(mut self, body: T, to: &mut impl Write) -> Result<Parts, Error>  where T::Data: Send, T::Error: Into<BoxError> {
    if !self.disabled_compression {
      self.headers().ok_or_else(|| Error::NoneValue(format!("")))?.append(header::ACCEPT_ENCODING, HeaderValue::from_str(Accepts::default().as_str().ok_or_else(|| Error::NoneValue(format!("Couldn't unwrap Accepts")))?)?);
    }
    let body = self.request.take().expect("Failed to take request-builder").body(body)?;
    crate::download::download(body, to, self.https_only, &mut self.progress, self.sockets).await
  }
}
