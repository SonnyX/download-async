use crate::{decoder::Accepts, progress::Progress};
use crate::dns::ResolverService;
use std::io::Write;
use hyper::client::Client;
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use crate::dns::SocketAddrs;
use http::response::Parts;
use crate::error::Error;

type Request<T> = crate::http::Request<T>;
type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub async fn download<T: HttpBody + Send + 'static>(request: Request<T>, to: &mut impl Write, https_only: bool, progress: &mut Option<Box<dyn Progress + Send>>, socket_addrs: Option<SocketAddrs>) -> Result<Parts, Error> where T::Data: Send, T::Error: Into<BoxError> {
    let res;

    if let Some(socket_addrs) = socket_addrs {    
        //Connect tcp stream to a hostname:port
        let resolver_service = ResolverService::new(socket_addrs.into());
        let mut http_connector : HttpConnector<ResolverService> = HttpConnector::new_with_resolver(resolver_service);
        http_connector.enforce_http(https_only);
        let mut https_connector = HttpsConnector::new_with_connector(http_connector);
        https_connector.https_only(https_only);
        let client = Client::builder().build::<_, T>(https_connector);
    
        // Send request
        res = client.request(request).await.or_else(|e| Err(Error::HyperError(e.into())))?;
    } else {
        let mut https_connector = HttpsConnector::new();
        https_connector.https_only(https_only);
        let client = Client::builder().build::<_, T>(https_connector);

        // Send request
        res = client.request(request).await.or_else(|e| Err(Error::HyperError(e.into())))?;
    }

    let status = res.status();
    let (mut parts, body) = res.into_parts();
    
    if status == 200 || status == 206 {

        let mut decoder = crate::decoder::Decoder::detect(&mut parts.headers, crate::body::Body::from(body), Accepts::default());
        if !decoder.is_encoded() && progress.is_some() {
            if let Some(content_length) = parts.headers.get("content-length") {
                let content_length : usize = content_length.to_str().expect("Couldn't convert content-length value to str.").parse().expect("Couldn't parse content-length as a usize.");
                progress.as_deref_mut().map(|progress| progress.set_file_size(content_length)).unwrap().await;
            }
        }
        while !decoder.is_end_stream() {
            // todo: Add timeout for chunk
            if let Some(chunk) = decoder.data().await {
                let chunk = chunk?;
                if progress.is_some() {
                    progress.as_deref_mut().map(|progress| progress.add_to_progress(chunk.len())).unwrap().await;
                }
                to.write_all(&chunk)?;
            }
        }
        Ok::<Parts, Error>(parts)
    } else {
        Err::<Parts, Error>(Error::StatusError(status))
    }
}