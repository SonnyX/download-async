extern crate tokio;
extern crate download_async;
extern crate async_trait;
extern crate futures;

use futures::join;
use async_trait::async_trait;
use tokio::time::sleep;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Progress {
    file_size: Arc<Mutex<usize>>,
    downloaded: Arc<Mutex<usize>>,
}

impl Progress {
    pub fn new() -> Self {
      Self {
        file_size: Arc::new(Mutex::new(0)),
        downloaded: Arc::new(Mutex::new(0))
      }
    }
  }

#[async_trait]
impl download_async::Progress for Progress {
  async fn get_file_size(&self) -> usize {
    self.file_size.lock().await.clone()
  }

  async fn get_progess(&self) -> usize {
    self.downloaded.lock().await.clone()
  }

  async fn set_file_size(&mut self, size: usize) {
    *(self.file_size.lock().await) = size;
  }

  async fn add_to_progress(&mut self, amount: usize) {
    *(self.downloaded.lock().await) += amount;
  }

  async fn remove_from_progress(&mut self, amount: usize) {
    *(self.downloaded.lock().await) -= amount;
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://file-examples-com.github.io/uploads/2017/02/zip_5MB.zip".parse::<download_async::http::Uri>()?;
    let req = download_async::http::Request::builder();
    if let Some(host) = url.host() {
        let req = req.uri(url.clone()).header("host", host);
        let req = req.body(download_async::Body::empty())?;

        let mut buffer = vec![];
        let mut progress = Progress::new();
        let progress_clone = progress.clone();
        let mut progress_option : Option<&mut Progress> = Some(&mut progress);

        let response_fut = download_async::download(req, &mut buffer, true, &mut progress_option, None);
        let progress_fut = report_progress(&progress_clone);

        let (response, _) = join!(response_fut, progress_fut);

        if response.is_ok() {
            println!("Done downloading");
        } else {
            println!("Something went wrong: {}", response.err().unwrap());
        }
    }
    Ok(())
}

async fn report_progress(progress: &Progress) {
    use download_async::Progress;

    let mut complete = false;
    let mut out_of = 0;
    let mut downloaded = 0;

    while !complete {
        if out_of == 0 {
            out_of = progress.get_file_size().await;
            if out_of != 0 {
              println!("The file_size has been set to {} bytes", out_of);
            }
        }
        let temp_download = progress.get_progess().await;
        if temp_download != downloaded {
            downloaded = temp_download;
            println!("Downloaded {} out of {} bytes!", downloaded, out_of);
        }

        if downloaded >= out_of && downloaded != 0 {
            complete = true;
        }
        sleep(Duration::from_millis(1_u64)).await;
    }
}