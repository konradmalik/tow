use log::{error, warn};
use std::env;
use std::fs::{copy, File};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::TowError;

const STORE_FILENAME: &str = "towstore.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct TowStore {
    binaries: Vec<BinaryEntry>,
    system: String,
    architecture: String,
    binaries_dir: PathBuf,
    store_dir: PathBuf,
}

impl<'a> TowStore {
    pub fn load_or_create(binaries_dir: &Path, store_dir: &Path) -> Result<Self, TowError> {
        let store_path_buf = store_dir.join(STORE_FILENAME);
        let store_path = store_path_buf.as_path();
        if store_path.is_dir() {
            let mut store = Self::load(store_path)?;
            store.change_binaries_path_if_needed(binaries_dir);
            return Ok(store);
        }
        Ok(Self::create(
            binaries_dir.to_path_buf(),
            store_dir.to_path_buf(),
        ))
    }

    pub fn get_binaries_dir(&'a self) -> &'a Path {
        self.binaries_dir.as_path()
    }

    pub fn get_store_path(&self) -> PathBuf {
        self.binaries_dir.join(STORE_FILENAME)
    }

    pub fn save(&self) -> Result<(), TowError> {
        let store_path = self.get_store_path();
        if store_path.is_file() && create_file_backup(store_path.as_path()).is_err() {
            error!("cannot backup previous towstore file")
        }
        let writer = File::create(store_path)?;
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn load(store_path: &Path) -> Result<Self, TowError> {
        let reader = File::open(store_path)?;
        let towstore: Self = serde_json::from_reader(reader)?;
        Ok(towstore)
    }

    pub fn create(binaries_dir: PathBuf, store_dir: PathBuf) -> Self {
        Self {
            binaries_dir,
            store_dir,
            ..Default::default()
        }
    }

    fn change_binaries_path_if_needed(&mut self, binaries_dir: &Path) {
        if self.get_binaries_dir() != binaries_dir {
            warn!(
                "changing binaries_dir from '{}' to '{}'",
                self.get_binaries_dir().display(),
                binaries_dir.display()
            );
            self.binaries_dir = binaries_dir.to_path_buf();
        }
    }
}

impl Default for TowStore {
    fn default() -> TowStore {
        TowStore {
            binaries: Vec::new(),
            system: env::consts::OS.to_string(),
            architecture: env::consts::ARCH.to_string(),
            binaries_dir: PathBuf::new(),
            store_dir: PathBuf::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct BinaryEntry {
    path: PathBuf,
    version: String,
    source: String,
}

fn create_file_backup(file_path: &Path) -> Result<(), TowError> {
    if !file_path.is_file() {
        return Err(TowError::new(
            format!("{} is not a file", file_path.display()).as_str(),
        ));
    }
    let backup = file_path.join(".bak");
    copy(file_path, backup)?;
    Ok(())
}

// TODO test
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_test() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
    }
}
