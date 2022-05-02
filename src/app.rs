use crate::store::{AddBinaryCmd, BinaryEntry, RemoveBinaryCmd};
use crate::{download, errors::TowError, local_store, store};
use log::{error, info};
use std::env;
use std::path::{Path, PathBuf};

const TOW_BINARIES_DIR_ENV: &str = "TOW_BINARIES_DIR";
const TOW_STORE_DIR_ENV: &str = "TOW_STORE_DIR";
const TOW_DATA_FOLDER_NAME: &str = "tow";
const DEFAULT_BINARY_VERSION: &str = "latest";

// TODO tests for remove and list

pub struct App<T: store::TowStore<'static>> {
    store: T,
}

impl App<local_store::LocalTowStore> {
    pub fn new_from_env() -> Result<Self, TowError> {
        let binaries_dir = env::var(TOW_BINARIES_DIR_ENV).map_or_else(
            |_| default_bin_dir(),
            |x| Path::new(x.as_str()).to_path_buf(),
        );
        let store_dir = env::var(TOW_STORE_DIR_ENV).map_or_else(
            |_| default_data_dir().join(TOW_DATA_FOLDER_NAME),
            |x| Path::new(x.as_str()).to_path_buf(),
        );
        Self::new_from_dirs(binaries_dir, store_dir)
    }

    pub fn new_from_dirs(binaries_dir: PathBuf, store_dir: PathBuf) -> Result<Self, TowError> {
        match local_store::LocalTowStore::load_or_create(
            binaries_dir.as_path(),
            store_dir.as_path(),
        ) {
            Err(e) => {
                error!("error while loading or creating TowStore: {}", e);
                Err(e)
            }
            Ok(store) => Ok(App { store }),
        }
    }
}

impl<T> App<T>
where
    T: store::TowStore<'static>,
{
    pub async fn install(
        &mut self,
        url: &str,
        name: Option<&str>,
        version: Option<&str>,
    ) -> Result<PathBuf, TowError> {
        match url::Url::parse(url) {
            Err(e) => {
                error!("Error parsing url: {}", e);
                Err(e.into())
            }
            Ok(url) => {
                info!("downloading url: {}", url);
                match download::download_file(&url, env::temp_dir().as_path()).await {
                    Err(e) => {
                        error!("Error downloading url: {}", e);
                        Err(e)
                    }
                    Ok(path) => {
                        info!("downloaded to {}", path.display());
                        let resolved_name = name
                            .unwrap_or_else(|| path.file_name().and_then(|x| x.to_str()).unwrap());
                        let resolved_version = version.unwrap_or(DEFAULT_BINARY_VERSION);
                        self.store.add_binary(AddBinaryCmd::new(
                            resolved_name.to_string(),
                            resolved_version.to_string(),
                            path.to_owned(),
                            url.to_string(),
                        ))?;
                        Ok(path)
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, name: String, version: String) -> Result<(), TowError> {
        let rm = RemoveBinaryCmd::new(name, version);
        self.store.remove_binary(rm)
    }

    pub fn list(&'static self) -> Vec<&'static BinaryEntry> {
        self.store.list_binaries()
    }
}

#[cfg(target_os = "macos")]
fn default_bin_dir() -> PathBuf {
    dirs::home_dir()
        .map(|x| x.join(".local").join("bin"))
        .expect("cannot get user's home dir")
}

#[cfg(target_os = "macos")]
fn default_data_dir() -> PathBuf {
    dirs::home_dir()
        .map(|x| x.join(".local").join("share"))
        .expect("cannot get user's home dir")
}

#[cfg(target_os = "linux")]
fn default_bin_dir() -> PathBuf {
    dirs::executable_dir().expect("cannot get user's bin dir")
}

#[cfg(target_os = "linux")]
fn default_data_dir() -> PathBuf {
    dirs::data_dir().expect("cannot get user's data dir")
}

#[cfg(target_os = "windows")]
fn default_bin_dir() -> PathBuf {
    panic!("no windows support yet, sorry!")
}

#[cfg(target_os = "windows")]
fn default_data_dir() -> PathBuf {
    panic!("no windows support yet, sorry!")
}

#[cfg(test)]
mod test {
    use super::*;
    use mockito::mock;

    #[test]
    fn test_install() {
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
        let mut app = App::new_from_dirs(temp_path.to_path_buf(), temp_path.to_path_buf()).unwrap();

        tokio_test::block_on(app.install(url.as_str(), None, None)).unwrap();
        assert!(temp_path.join(filename).is_file());
    }
}
