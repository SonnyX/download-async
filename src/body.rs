use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_core::Stream;
use http_body::Body as HttpBody;
use pin_project_lite::pin_project;
use tokio::time::Sleep;

/// An asynchronous request body.
pub struct Body {
    inner: Inner,
}

// The `Stream` trait isn't stable, so the impl isn't public.
pub(crate) struct ImplStream(Body);

enum Inner {
    Reusable(Bytes),
    Streaming {
        body: Pin<
            Box<
                dyn HttpBody<Data = Bytes, Error = Box<dyn std::error::Error + Send + Sync>>
                    + Send
                    + Sync,
            >,
        >,
        timeout: Option<Pin<Box<Sleep>>>,
    },
}

pin_project! {
    struct WrapStream<S> {
        #[pin]
        inner: S,
    }
}

struct WrapHyper(hyper::Body);

impl Body {
    /// Wrap a futures `Stream` in a box inside `Body`.
    ///
    /// # Example
    ///
    /// ```
    /// # use reqwest::Body;
    /// # use futures_util;
    /// # fn main() {
    /// let chunks: Vec<Result<_, ::std::io::Error>> = vec![
    ///     Ok("hello"),
    ///     Ok(" "),
    ///     Ok("world"),
    /// ];
    ///
    /// let stream = futures_util::stream::iter(chunks);
    ///
    /// let body = Body::wrap_stream(stream);
    /// # }
    /// ```
    ///
    /// # Optional
    ///
    /// This requires the `stream` feature to be enabled.
    #[cfg(feature = "stream")]
    pub fn wrap_stream<S>(stream: S) -> Body
    where
        S: futures_core::stream::TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        Body::stream(stream)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn wrap(body: hyper::Body) -> Body {
        Body {
            inner: Inner::Streaming {
                body: Box::pin(WrapHyper(body)),
                timeout: None,
            },
        }
    }

    pub(crate) fn empty() -> Body {
        Body::reusable(Bytes::new())
    }

    pub(crate) fn reusable(chunk: Bytes) -> Body {
        Body {
            inner: Inner::Reusable(chunk),
        }
    }

    pub(crate) fn into_stream(self) -> ImplStream {
        ImplStream(self)
    }

    #[cfg(feature = "multipart")]
    pub(crate) fn content_length(&self) -> Option<u64> {
        match self.inner {
            Inner::Reusable(ref bytes) => Some(bytes.len() as u64),
            Inner::Streaming { ref body, .. } => body.size_hint().exact(),
        }
    }
}

impl From<hyper::Body> for Body {
    #[inline]
    fn from(body: hyper::Body) -> Body {
        Self {
            inner: Inner::Streaming {
                body: Box::pin(WrapHyper(body)),
                timeout: None,
            },
        }
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

// ===== impl ImplStream =====

impl HttpBody for ImplStream {
    type Data = Bytes;
    type Error = crate::error::Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let opt_try_chunk = match self.0.inner {
            Inner::Streaming {
                ref mut body,
                ref mut timeout,
            } => {
                if let Some(ref mut timeout) = timeout {
                    if let Poll::Ready(()) = timeout.as_mut().poll(cx) {
                        return Poll::Ready(Some(Err(crate::error::Error::TimedOut())));
                    }
                }
                futures_core::ready!(Pin::new(body).poll_data(cx))
                    .map(|opt_chunk| opt_chunk.map(Into::into).map_err(|e| crate::error::Error::InvalidBody(e)))
            }
            Inner::Reusable(ref mut bytes) => {
                if bytes.is_empty() {
                    None
                } else {
                    Some(Ok(std::mem::replace(bytes, Bytes::new())))
                }
            }
        };

        Poll::Ready(opt_try_chunk)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        match self.0.inner {
            Inner::Streaming { ref body, .. } => body.is_end_stream(),
            Inner::Reusable(ref bytes) => bytes.is_empty(),
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        match self.0.inner {
            Inner::Streaming { ref body, .. } => body.size_hint(),
            Inner::Reusable(ref bytes) => {
                let mut hint = http_body::SizeHint::default();
                hint.set_exact(bytes.len() as u64);
                hint
            }
        }
    }
}

impl Stream for ImplStream {
    type Item = Result<Bytes, crate::error::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.poll_data(cx)
    }
}

// ===== impl WrapStream =====

impl<S, D, E> HttpBody for WrapStream<S>
where
    S: Stream<Item = Result<D, E>>,
    D: Into<Bytes>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Data = Bytes;
    type Error = E;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let item = futures_core::ready!(self.project().inner.poll_next(cx)?);

        Poll::Ready(item.map(|val| Ok(val.into())))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

// ===== impl WrapHyper =====

impl HttpBody for WrapHyper {
    type Data = Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        // safe pin projection
        Pin::new(&mut self.0)
            .poll_data(cx)
            .map(|opt| opt.map(|res| res.map_err(Into::into)))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        self.0.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        HttpBody::size_hint(&self.0)
    }
}
