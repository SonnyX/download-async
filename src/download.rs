pub struct Downloader {

}

impl Downloader {
    pub fn new() -> Self {
        Self {

        }
    }
    
    pub async fn download(self, progress: impl Progress, to: impl Write) {
        //Connect tcp stream to a hostname:port
        let tls : tokio_tls::TlsConnector = native_tls::TlsConnector::new().unexpected(concat!(file!(),":",line!())).into();
        let resolver_service = ResolverService::new(socket_addrs.into());
        let mut http_connector : HttpConnector<ResolverService> = HttpConnector::new_with_resolver(resolver_service);
        http_connector.enforce_http(false);
        let https_connector : hyper_tls::HttpsConnector<HttpConnector<ResolverService>> = (http_connector, tls).into();
        let client = Client::builder().build::<_, hyper::Body>(https_connector);
    
        // Set up a request
        let req = hyper::Request::builder();
        let req = req.uri(uri).header(/*set headers*/);
        let req = req.body(hyper::Body::empty()).unexpected(concat!(file!(),":",line!()));
    
        // Send request
        let res = client.request(req).await?;
    
        if res.status() == 200 || res.status() == 206 {
        // Initialize progress in front-end to be 0 up to maximum content_length
        let content_length : usize = res.headers().get("content-length").unexpected("Expected a content-length header, however none was found.").to_str().unexpected("Couldn't convert content-length value to str.").parse().unexpected("Couldn't parse content-length as a usize.");
        let progress_clone = progress.clone();
        std::thread::spawn(move || {
            progress.call(None, &make_args!(format!("[0, {}]", content_length)), None).unexpected(concat!(file!(),":",line!()));
        });
    
        // Set up vector where the stream will write into
        let mut download_contents = Vec::with_capacity(content_length);
        let mut downloaded = 0;
        let mut body = res.into_body();
    
        while !body.is_end_stream() {
            if let Some(chunk) = body.data().await {
            let chunk = chunk?;
            let chunk_size = chunk.len();
            downloaded += chunk_size;
            let progress_clone = progress_clone.clone();
            if downloaded*100/content_length > (downloaded-chunk_size)*100/content_length {
                std::thread::spawn(move || {
                    progress_clone.call(None, &make_args!(format!("[{},{}]", downloaded.to_string(), content_length.to_string())), None).unexpected(concat!(file!(),":",line!()));
                });
            }
            download_contents.write_all(&chunk)?;
            }
        }
        drop(client);
    
        let download_contents = std::io::Cursor::new(download_contents);
        let mut output_path = std::env::current_exe().unexpected(concat!(file!(),":",line!()));
        output_path.pop();
        let target_dir = output_path.clone();
        output_path.pop();
        let working_dir = output_path.clone();
        output_path.push("launcher_update_extracted/");
        info!("Extracting launcher update to: {:?}", output_path);
        let mut self_update_executor = output_path.clone();
    }
    
    pub fn download_blocking(self, progress: impl Progress, to: impl Write) {
        let mut rt = tokio::runtime::Builder::new().basic_scheduler().enable_time().enable_io().build().unexpected(concat!(file!(),":",line!()));
        let result = rt.enter(|| {
          rt.spawn(async move {
            self.download(progress, to).await;
          })
        });
        let _ = rt.block_on(result).unexpected(concat!(file!(),":",line!()));
    }
}

pub struct Builder {
  //headers
  //url
  //
}

impl Builder {
    pub fn build() -> Downloader {
        Downloader::new()
    }
}