use std::cmp::min;
use std::fs::File;
use std::io;
use std::io::Write;
use url::Url;

use futures_util::StreamExt;
use reqwest;

#[derive(Debug)]
pub enum Error {
    RequestError { e: reqwest::Error },
    IOError { e: io::Error },
    StringError { e: String },
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::RequestError { e }
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::StringError { e }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IOError { e }
    }
}

pub async fn download_file(client: &reqwest::Client, url: &Url, path: &str) -> Result<(), Error> {
    // Reqwest setup
    let url_str = url.as_str();
    let res = client.get(url_str).send().await?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let pb = progress::setup_progress_bar(url_str, total_size);

    // download chunks
    let mut file = File::create(path)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url_str, path));
    return Ok(());
}

mod progress {
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn setup_progress_bar(url: &str, total_size: u64) -> ProgressBar {
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
        pb.set_message(format!("Downloading {}", url));
        return pb;
    }
}
