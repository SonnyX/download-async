use std::net::ToSocketAddrs;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;
use hyper::client::connect::dns::Name;


#[derive(Debug, Clone)]
pub struct SocketAddrs {
  inner: std::vec::IntoIter<std::net::SocketAddr>
}

impl PartialEq for SocketAddrs {
  fn eq(&self, other: &SocketAddrs) -> bool {
    self.inner.as_slice() == other.inner.as_slice()
  }
}

impl From<Vec<std::net::SocketAddr>> for SocketAddrs {
  fn from(other: Vec<std::net::SocketAddr>) -> Self {
    SocketAddrs {
      inner: other.into_iter()
    }
  }
}

impl ToSocketAddrs for SocketAddrs {
  type Iter = std::vec::IntoIter<std::net::SocketAddr>;
  fn to_socket_addrs(&self) -> std::io::Result<std::vec::IntoIter<std::net::SocketAddr>> {
    Ok(self.inner.clone())
  }
}


/*impl Iterator for SocketAddrs {
  type Item = std::net::IpAddr;

  fn next(&mut self) -> Option<Self::Item> {
      self.inner.next().map(|sa| sa.ip())
  }
}
*/
impl Iterator for SocketAddrs {
  type Item = std::net::SocketAddr;

  fn next(&mut self) -> Option<Self::Item> {
      self.inner.next()
  }
}

#[derive(Clone)]
pub struct ResolverService {
  pub socket_addrs: SocketAddrs
}

impl ResolverService {
  pub fn new(socket_addrs: SocketAddrs) -> Self {
    ResolverService {
      socket_addrs
    }
  }
}

impl tower::Service<Name> for ResolverService {
  type Response = SocketAddrs;
  type Error = Box<dyn Error + Send + Sync>;
  // We can't "name" an `async` generated future.
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send >>;

  fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
      // This connector is always ready, but others might not be.
      Poll::Ready(Ok(()))
  }

  fn call(&mut self, _: Name) -> Self::Future {
    let socket_addrs = self.socket_addrs.clone();
    let fut = async move { 
      Ok(socket_addrs) 
    };
    Box::pin(fut)
  }
}