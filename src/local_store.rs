use log::{error, info, warn};
use std::collections::HashMap;
use std::env;
use std::fs::{copy, remove_file, rename, File};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::TowError;
use crate::store::{AddBinaryCmd, BinaryEntry, Hashable, RemoveBinaryCmd, TowStore};

const STORE_FILENAME: &str = "towstore.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalTowStore {
    binaries: HashMap<String, BinaryEntry>,
    system: String,
    architecture: String,
    binaries_dir: PathBuf,
    store_dir: PathBuf,
}

// TODO write tests

impl<'a> TowStore<'a> for LocalTowStore {
    fn add_binary(&mut self, add: AddBinaryCmd) -> Result<(), TowError> {
        let be_hash = add.hash();
        if self.binaries.contains_key(be_hash.as_str()) {
            return Err(TowError::new(
                format!("{} is already in the store", be_hash).as_str(),
            ));
        }

        // move binary to our store
        let file_location = add.path.as_path();
        let file_name = file_location
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or_else(|| TowError::new("cannot get filename from file_location"))?;
        let new_location = self.get_binaries_dir().join(file_name);
        rename(file_location, new_location.as_path())?;

        // once moved we can add entry
        let mut be = BinaryEntry::from_add_cmd(add);
        be.path = new_location;
        self.binaries.insert(be_hash, be.clone());

        // save but if error then remove
        match self.save() {
            Err(e) => {
                error!("error while removing: {:?}", be);
                self.remove_binary(RemoveBinaryCmd {
                    name: be.name,
                    version: be.version,
                })?;
                Err(e)
            }
            _ => {
                info!("added: {:?} to the store", be);
                Ok(())
            }
        }
    }

    fn remove_binary(&mut self, rm: RemoveBinaryCmd) -> Result<(), TowError> {
        let hash = rm.hash();

        let be = self
            .binaries
            .get(hash.as_str())
            .ok_or_else(|| TowError::new(format!("{} is not in the store", hash).as_str()))?;

        remove_file(be.path.as_path())?;
        Ok(())
    }

    fn list_binaries(&'a self) -> Vec<&'a BinaryEntry> {
        self.binaries.values().collect()
    }
}

impl<'a> LocalTowStore {
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

    fn create(binaries_dir: PathBuf, store_dir: PathBuf) -> Self {
        Self {
            binaries_dir,
            store_dir,
            binaries: HashMap::new(),
            system: env::consts::OS.to_string(),
            architecture: env::consts::ARCH.to_string(),
        }
    }

    fn load(store_path: &Path) -> Result<Self, TowError> {
        let reader = File::open(store_path)?;
        let towstore: Self = serde_json::from_reader(reader)?;
        Ok(towstore)
    }

    fn save(&self) -> Result<(), TowError> {
        let store_path = self.get_store_path();
        if store_path.is_file() && create_file_backup(store_path.as_path()).is_err() {
            error!("cannot backup previous towstore file")
        }
        let writer = File::create(store_path)?;
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    fn get_binaries_dir(&'a self) -> &'a Path {
        self.binaries_dir.as_path()
    }

    fn get_store_path(&self) -> PathBuf {
        self.binaries_dir.join(STORE_FILENAME)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_test() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
    }
}
