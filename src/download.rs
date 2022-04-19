use crate::errors::TowError;
use futures_util::StreamExt;
use log::warn;
use reqwest::header;
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use url::Url;

pub async fn download_file(url: &Url, path: &Path) -> Result<PathBuf, TowError> {
    if !path.is_dir() {
        return Err(TowError::new(&format!(
            "'{}' is not a directory",
            path.display()
        )));
    };
    // disable auto-decompression so that content-length has some meaning
    let client = reqwest::Client::builder()
        .no_deflate()
        .no_gzip()
        .no_brotli()
        .build()?;
    let url_str = url.as_str();
    let res = client.get(url_str).send().await?;
    let content_length = get_content_length(&res).unwrap_or_else(|| {
        warn!("cannot extract content-length");
        0
    });
    let filename = get_filename(res.headers())?;
    let full_path = path.join(filename);

    let pb = progress::setup_progress_bar(url_str, content_length);

    // download chunks
    let mut file = File::create(&full_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|_| "Error while downloading file".to_owned())?;
        file.write_all(&chunk)?;
        let new = min(downloaded + (chunk.len() as u64), content_length);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url_str, full_path.display()));
    Ok(full_path)
}

fn get_content_length(response: &reqwest::Response) -> Option<u64> {
    response.content_length()
}

fn get_filename(headers: &header::HeaderMap) -> Result<&str, TowError> {
    let header = match headers.get(header::CONTENT_DISPOSITION) {
        None => Err(TowError::new(&format!(
            "no {} header",
            header::CONTENT_DISPOSITION
        ))),
        Some(cd) => Ok(cd.to_str()?),
    }?;
    parse_filename_from_content_disposition(header)
}

fn parse_filename_from_content_disposition(header: &str) -> Result<&str, TowError> {
    match header.split_once("filename=") {
        None => Err(TowError::new(&format!(
            "cannot get filename from '{}'",
            header
        ))),
        Some(tup) => {
            let mut extracted = tup.1.split_once(';').map(|x| x.0).unwrap_or(tup.1);
            if extracted.starts_with('\"') {
                extracted = &extracted[1..];
            }
            if extracted.ends_with('\"') {
                extracted = &extracted[0..extracted.len() - 1]
            }
            Ok(extracted)
        }
    }
}
mod progress {
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn setup_progress_bar(url: &str, total_size: u64) -> ProgressBar {
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
        pb.set_message(format!("Downloading {}", url));
        pb
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockito::mock;

    #[test]
    fn test_parsing_content_disposition() {
        struct TestData<'a> {
            header: &'a str,
            expected: &'a str,
        }

        let inputs = [
            TestData {
                header: "attachment; filename=content.txt",
                expected: "content.txt",
            },
            TestData {
                header: r#"attachment; filename="EURO rates"; filename*=utf-8''%e2%82%ac%20rates"#,
                expected: "EURO rates",
            },
            TestData {
                header: "attachment; filename=omáèka.jpg",
                expected: "omáèka.jpg",
            },
            TestData {
                header: "attachment; filename=EXAMPLE- I'm ößä.dat; filename*=iso-8859-1''EXAMPLE-%20I%27m%20%F6%DF%E4.dat",
                expected: "EXAMPLE- I'm ößä.dat",
            },
        ];

        for td in inputs {
            let parsed = parse_filename_from_content_disposition(td.header)
                .expect("error not expected here");
            assert_eq!(parsed, td.expected)
        }
    }

    #[test]
    fn test_download_file() {
        let filename = "hello.txt";
        let endpoint = "/helloworld";
        let root_url = &mockito::server_url();

        let _m = mock("GET", endpoint)
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header(
                "content-disposition",
                &format!("attachment: filename={}", filename),
            )
            .with_body("Hello world!")
            .create();

        let url = Url::parse(&format!("{}{}", root_url, endpoint)).unwrap();

        // this is gets deleted once it goes out of scope
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // 1 - success
        tokio_test::block_on(download_file(&url, &temp_path)).unwrap();
        assert!(temp_path.join(filename).is_file());

        // 2 - failure
        let err = tokio_test::block_on(download_file(&url, &temp_path.join(filename))).unwrap_err();
        assert!(err.to_string().contains("not a directory"));
    }
}
