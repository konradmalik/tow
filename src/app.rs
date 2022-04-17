use crate::{download, errors::TowError};
use log::{error, info};
use std::path::Path;

const DEFAULT_BINARIES_DIR: &str = "";

pub struct App<'a> {
    binaries_dir: &'a Path,
}

impl<'a> App<'a> {
    pub fn new_from_env() -> App<'a> {
        // TODO from env
        let binaries_dir = DEFAULT_BINARIES_DIR;

        App {
            binaries_dir: Path::new(binaries_dir),
        }
    }

    pub async fn install_binary(self, url: &str) -> Result<(), TowError> {
        match url::Url::parse(url) {
            Err(e) => {
                error!("Error parsing url: {}", e);
                Err(e.into())
            }
            Ok(url) => {
                info!("downloading file: {}", url);
                match download::download_file(&url, self.binaries_dir).await {
                    Err(e) => {
                        error!("Error downloading url: {}", e);
                        Err(e)
                    }
                    Ok(()) => {
                        info!("downloaded!");
                        Ok(())
                    }
                }
            }
        }
    }
}
