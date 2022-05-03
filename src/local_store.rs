use log::{error, info, warn};
use std::collections::HashMap;
use std::env;
use std::fs::{copy, remove_file, rename, File};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::TowError;
use crate::store::{AddBinaryCmd, BinaryEntry, Hashable, RemoveBinaryCmd, TowStore};

const STORE_FILENAME: &str = "towstore.json";
const STORE_BACKUP_FILENAME: &str = ".towstore.json.bak";

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalTowStore {
    binaries: HashMap<String, BinaryEntry>,
    system: String,
    architecture: String,
    binaries_dir: PathBuf,
    store_dir: PathBuf,
}

impl TowStore for LocalTowStore {
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
                error!("error while adding: {}", be);
                self.remove_binary(RemoveBinaryCmd {
                    name: be.name,
                    version: be.version,
                })?;
                Err(e)
            }
            _ => {
                info!("added: {} to the store", be);
                Ok(())
            }
        }
    }

    fn remove_binary(&mut self, rm: RemoveBinaryCmd) -> Result<(), TowError> {
        let hash = rm.hash();

        let be = self
            .binaries
            .remove(hash.as_str())
            .ok_or_else(|| TowError::new(format!("{} is not in the store", hash).as_str()))?;

        remove_file(be.path.as_path())?;

        match self.save() {
            Err(e) => {
                error!("error while saving store: {}", e);
                Err(e)
            }
            _ => {
                info!("removed: {} from the store", be);
                Ok(())
            }
        }
    }

    fn list_binaries(&self) -> Vec<&BinaryEntry> {
        self.binaries.values().collect()
    }
}

impl LocalTowStore {
    pub fn load_or_create(binaries_dir: &Path, store_dir: &Path) -> Result<Self, TowError> {
        let store_path_buf = store_dir.join(STORE_FILENAME);
        let store_path = store_path_buf.as_path();
        if store_path.is_file() {
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
        if store_path.is_file()
            && create_file_backup(store_path.as_path(), STORE_BACKUP_FILENAME).is_err()
        {
            error!("cannot backup previous towstore file")
        }
        let writer = File::create(store_path)?;
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    fn get_binaries_dir(&self) -> &Path {
        self.binaries_dir.as_path()
    }

    fn get_store_path(&self) -> PathBuf {
        self.store_dir.join(STORE_FILENAME)
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

fn create_file_backup(file_path: &Path, backup_file_name: &str) -> Result<(), TowError> {
    if !file_path.is_file() {
        return Err(TowError::new(
            format!("{} is not a file", file_path.display()).as_str(),
        ));
    }
    let backup = file_path.with_file_name(backup_file_name);
    copy(file_path, backup)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_dir;

    const FAKE_BINARY_NAME: &str = "test";
    const FAKE_BINARY_VERSION: &str = "latest";

    #[test]
    fn test_add_binary() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let mut store = temp_store(temp_path);
        // make sure it's empty
        assert_eq!(store.list_binaries().len(), 0);
        // add fake bin
        add_fake_binary(&mut store, FAKE_BINARY_NAME.to_string()).unwrap();
        // make sure it's there
        assert_eq!(store.list_binaries().len(), 1)
    }

    #[test]
    fn test_add_binary_duplicate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let mut store = temp_store(temp_path);
        // make sure it's empty
        assert_eq!(store.list_binaries().len(), 0);
        // add fake bin
        add_fake_binary(&mut store, FAKE_BINARY_NAME.to_string()).unwrap();
        // add another
        let duplicate = add_fake_binary(&mut store, FAKE_BINARY_NAME.to_string());
        assert!(duplicate.is_err());
        assert!(duplicate
            .unwrap_err()
            .to_string()
            .contains("already in the store"));
        // make sure it's still 1
        assert_eq!(store.list_binaries().len(), 1)
    }

    #[test]
    fn test_remove_binary() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let mut store = temp_store(temp_path);
        // make sure it's empty
        assert_eq!(store.list_binaries().len(), 0);
        // add fake bin
        add_fake_binary(&mut store, FAKE_BINARY_NAME.to_string()).unwrap();
        // make sure it's there
        assert_eq!(store.list_binaries().len(), 1);
        // remove it
        store
            .remove_binary(RemoveBinaryCmd::new(
                FAKE_BINARY_NAME.to_string(),
                FAKE_BINARY_VERSION.to_string(),
            ))
            .unwrap();
        // make sure it's empty
        assert_eq!(store.list_binaries().len(), 0);
    }

    #[test]
    fn test_remove_non_existent_binary() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let mut store = temp_store(temp_path);
        // make sure it's empty
        assert_eq!(store.list_binaries().len(), 0);
        // remove something
        let removal = store.remove_binary(RemoveBinaryCmd::new(
            FAKE_BINARY_NAME.to_string(),
            FAKE_BINARY_VERSION.to_string(),
        ));
        // make sure it errored
        assert!(removal.is_err());
        assert!(removal
            .unwrap_err()
            .to_string()
            .contains("not in the store"));
        // make sure it's empty
        assert_eq!(store.list_binaries().len(), 0);
    }

    #[test]
    fn temp_store_create_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        // setup first store
        let mut store1 = LocalTowStore::load_or_create(temp_path, temp_path).unwrap();
        assert_eq!(store1.list_binaries().len(), 0);
        add_fake_binary(&mut store1, FAKE_BINARY_NAME.to_string()).unwrap();
        assert_eq!(store1.list_binaries().len(), 1);
        // after saving it must dump the json file, so anoter load or create must load it
        let store2 = LocalTowStore::load_or_create(temp_path, temp_path).unwrap();
        assert_eq!(store2.list_binaries().len(), 1);
        // add another binary so that backup is created
        add_fake_binary(&mut store1, "fake123".to_string()).unwrap();
        // backup file should also be present
        let backup_store_file = store2
            .get_store_path()
            .with_file_name(STORE_BACKUP_FILENAME);

        assert!(backup_store_file.is_file());
    }

    fn _print_files_in_dir(dir: &Path) {
        let paths = read_dir(dir).unwrap();

        for path in paths {
            println!("Name: {}", path.unwrap().path().display())
        }
    }

    fn temp_store(temp_path: &Path) -> LocalTowStore {
        LocalTowStore::load_or_create(temp_path, temp_path).unwrap()
    }

    fn add_fake_binary(store: &mut LocalTowStore, name: String) -> Result<(), TowError> {
        // fake binary
        let fake_binary_path = store.get_binaries_dir().join("test.bin");
        File::create(fake_binary_path.as_path()).unwrap();

        // add to store
        store.add_binary(AddBinaryCmd::new(
            name,
            FAKE_BINARY_VERSION.to_string(),
            fake_binary_path,
            "fake".to_string(),
        ))
    }
}
