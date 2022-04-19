use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct BinariesRegistry {
    binaries: Vec<BinaryEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BinaryEntry {
    path: PathBuf,
    version: String,
}

fn _template() {
    let be = BinaryEntry {
        path: Path::new("test.txt").to_path_buf(),
        version: "123".to_string(),
    };

    // TODO use io writer here
    let pretty = serde_json::to_string_pretty(&be).unwrap();

    println!("serialized = {}", pretty);

    // Convert the JSON string back
    let deserialized: BinaryEntry = serde_json::from_str(&pretty).unwrap();

    println!("deserialized = {:?}", deserialized);
}
