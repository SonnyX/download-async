use crate::progress::Progress;
use crate::dns::ResolverService;
use crate::error::StatusError;
use std::io::Write;
use std::error::Error;
use hyper::client::Client;
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

pub async fn download<T: HttpBody + Send + 'static>(request: crate::http::Request<T>, mut progress: impl Progress, mut to: impl Write, socket_addrs: crate::dns::SocketAddrs, https_only: bool) -> Result<(), Box<dyn Error>> where T::Data: Send, T::Error: Into<Box<dyn Error + Send + Sync>> {
    //Connect tcp stream to a hostname:port
    let resolver_service = ResolverService::new(socket_addrs.into());
    let mut http_connector : HttpConnector<ResolverService> = HttpConnector::new_with_resolver(resolver_service);
    http_connector.enforce_http(https_only);
    let mut https_connector = HttpsConnector::new_with_connector(http_connector);
    https_connector.https_only(https_only);
    let client = Client::builder().build::<_, T>(https_connector);

    // Send request
    let res = client.request(request).await?;
    let status = res.status();

    if status == 200 || status == 206 {
        // Initialize progress in front-end to be 0 up to maximum content_length
        if let Some(content_length) = res.headers().get("content-length") {
            let content_length : usize = content_length.to_str().expect("Couldn't convert content-length value to str.").parse().expect("Couldn't parse content-length as a usize.");
            progress.set_file_size(content_length).await;
        }
    
        // Set up vector where the stream will write into
        let mut body = res.into_body();
    
        while !body.is_end_stream() {
            if let Some(chunk) = body.data().await {
                let chunk = chunk?;
                to.write_all(&chunk)?;
            }
        }
        drop(client);

        Ok::<(), Box<dyn Error>>(())
    } else {
        drop(client);

        Err::<(), Box<dyn Error>>(StatusError::from(status))
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