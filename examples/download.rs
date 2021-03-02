extern crate tokio;
extern crate download_async;
extern crate async_trait;

use async_trait::async_trait;

pub struct Progress {

}

#[async_trait]
impl download_async::Progress for Progress {
  async fn get_file_size(&self) -> usize {
    64
  }

  async fn get_progess(&self) -> usize {
    64
  }

  async fn set_file_size(&mut self, _size: usize) {

  }

  async fn add_to_progress(&mut self, _amount: usize) {

  }

  async fn remove_from_progress(&mut self, _bytes: usize) {

  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://raw.githubusercontent.com/SonnyX/download-async/master/LICENSE".parse::<download_async::http::Uri>()?;
    let req = download_async::http::Request::builder();
    if let Some(host) = url.host() {
        let req = req.uri(url.clone()).header("host", host);
        let req = req.body(download_async::Body::empty())?;

        let mut buffer = vec![];
        let mut progress : Option<&mut Progress> = None;

        let response = download_async::download(req, &mut buffer, true, &mut progress, None).await;

        if response.is_ok() {
            let text = std::str::from_utf8(&buffer).expect("Expected an utf-8 string");
            println!("output: {}", text);
        } else {
            println!("Something went wrong: {}", response.err().unwrap());
        }
    }
    Ok(())
}