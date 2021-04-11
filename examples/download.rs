extern crate tokio;
extern crate download_async;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut downloader = download_async::Downloader::new();
  downloader.use_uri("https://raw.githubusercontent.com/SonnyX/download-async/master/LICENSE".parse::<download_async::http::Uri>()?);
  let mut buffer = vec![];
  let response = downloader.download(download_async::Body::empty(), &mut buffer).await;
  if response.is_ok() {
      let text = std::str::from_utf8(&buffer).expect("Expected an utf-8 string");
      println!("output: {}", text);
  } else {
      println!("Something went wrong: {}", response.err().unwrap());
  }
  Ok(())
}