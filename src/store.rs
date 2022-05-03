use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::TowError;

pub trait TowStore {
    fn add_binary(&mut self, add: AddBinaryCmd) -> Result<(), TowError>;
    fn remove_binary(&mut self, rm: RemoveBinaryCmd) -> Result<(), TowError>;
    fn list_binaries(&self) -> Vec<&BinaryEntry>;
}

pub trait Hashable {
    fn hash(&self) -> String;
}

pub struct AddBinaryCmd {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub source: String,
}

impl AddBinaryCmd {
    pub fn new(name: String, version: String, path: PathBuf, source: String) -> Self {
        Self {
            name,
            version,
            path,
            source,
        }
    }
}

impl Hashable for AddBinaryCmd {
    fn hash(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

pub struct RemoveBinaryCmd {
    pub name: String,
    pub version: String,
}

impl RemoveBinaryCmd {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

impl Hashable for RemoveBinaryCmd {
    fn hash(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BinaryEntry {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub source: String,
}

impl BinaryEntry {
    pub fn from_add_cmd(add: AddBinaryCmd) -> Self {
        Self {
            name: add.name,
            version: add.version,
            path: add.path,
            source: add.source,
        }
    }
}

impl Hashable for BinaryEntry {
    fn hash(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

impl Display for BinaryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {} {}",
            self.name,
            self.version,
            self.path.display(),
            self.source
        ))
    }
}
