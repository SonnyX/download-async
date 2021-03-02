use crate::progress::Progress;
use crate::dns::ResolverService;
use crate::error::StatusError;
use std::io::Write;
use hyper::client::Client;
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use crate::dns::SocketAddrs;
use http::response::Parts;

type Request<T> = crate::http::Request<T>;
type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn download<T: HttpBody + Send + 'static>(request: Request<T>, to: &mut impl Write, https_only: bool, progress: &mut Option<&mut impl Progress>, socket_addrs: Option<SocketAddrs>) -> Result<Parts, Error> where T::Data: Send, T::Error: Into<Error> {
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
        res = client.request(request).await?;
    } else {
        let mut https_connector = HttpsConnector::new();
        https_connector.https_only(https_only);
        let client = Client::builder().build::<_, T>(https_connector);

        // Send request
        res = client.request(request).await?;
    }

    let status = res.status();
    let (parts, mut body) = res.into_parts();
    if status == 200 || status == 206 {
        if progress.is_some() {
            if let Some(content_length) = parts.headers.get("content-length") {
                let content_length : usize = content_length.to_str().expect("Couldn't convert content-length value to str.").parse().expect("Couldn't parse content-length as a usize.");
                progress.as_deref_mut().map(|progress| progress.set_file_size(content_length)).unwrap().await;
            }
        }
        while !body.is_end_stream() {
            // todo: Add timeout for chunk
            if let Some(chunk) = body.data().await {
                let chunk = chunk?;
                progress.as_deref_mut().map(|progress| progress.add_to_progress(chunk.len())).unwrap().await;
                to.write_all(&chunk)?;
            }
        }
        Ok::<Parts, Error>(parts)
    } else {
        Err::<Parts, Error>(StatusError::from(status))
    }
}

/*
pub fn download_blocking(self, progress: impl Progress, to: impl Write) -> Result<(), Box<dyn Error>>  {
    let mut rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let result = rt.enter(|| {
        rt.spawn(async move {
        self.download(progress, to).await;
        })
    });
    let _ = rt.block_on(result).unexpected(concat!(file!(),":",line!()));
}
*/