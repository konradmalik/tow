use crate::{download, errors::TowError};
use log::{error, info};
use std::env;
use std::path::{Path, PathBuf};

pub struct App {
    binaries_dir: PathBuf,
}

impl App {
    pub fn new_from_env() -> Self {
        let binaries_dir = env::var("TOW_BINARIES_DIR").map_or_else(
            |_| default_bin_dir(),
            |x| Path::new(x.as_str()).to_path_buf(),
        );
        Self::new(binaries_dir)
    }

    pub fn new(binaries_dir: PathBuf) -> Self {
        App { binaries_dir }
    }

    pub async fn install_binary(self, url: &str) -> Result<PathBuf, TowError> {
        match url::Url::parse(url) {
            Err(e) => {
                error!("Error parsing url: {}", e);
                Err(e.into())
            }
            Ok(url) => {
                info!("downloading url: {}", url);
                match download::download_file(&url, self.binaries_dir.as_path()).await {
                    Err(e) => {
                        error!("Error downloading url: {}", e);
                        Err(e)
                    }
                    Ok(path) => {
                        info!("downloaded to {}", path.display());
                        Ok(path)
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn default_bin_dir() -> PathBuf {
    dirs::home_dir()
        .map(|x| x.join(".local").join("bin"))
        .expect("cannot get user's home dir")
}

#[cfg(target_os = "linux")]
fn default_bin_dir() -> PathBuf {
    dirs::executable_dir().expect("cannot get user's bin dir")
}

#[cfg(target_os = "windows")]
fn default_bin_dir() -> PathBuf {
    panic!("no windows support yet, sorry!")
}

#[cfg(test)]
mod test {
    use super::*;
    use mockito::mock;

    #[test]
    fn test_install_binary() {
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

        let url = format!("{}{}", root_url, endpoint);

        // this is gets deleted once it goes out of scope
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let app = App::new(temp_path.to_path_buf());

        tokio_test::block_on(app.install_binary(url.as_str())).unwrap();
        assert!(temp_path.join(filename).is_file());
    }
}
