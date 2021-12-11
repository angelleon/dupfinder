use chrono::prelude::*;
use generic_array::GenericArray;
use hex_literal::hex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::Read;
use std::mem::take;
use std::path::Path;
use std::time::SystemTime;

type Hash = [u8; 32];

struct FileRecord {
    pub path: String,
    pub created_at: SystemTime,
}

impl FileRecord {
    pub fn new(path: String, created_at: SystemTime) -> Self {
        Self { path, created_at }
    }
}

pub struct DupFinder {
    paths: Vec<String>,
    hasher: Sha256,
    uniques: HashMap<Hash, FileRecord>,
    duplicates: HashMap<Hash, Vec<FileRecord>>,
}

impl DupFinder {
    pub fn new(paths: Vec<String>) -> Self {
        Self {
            paths,
            hasher: Sha256::new(),
            uniques: HashMap::new(),
            duplicates: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        for path in &self.paths {
            let inspected_path = Path::new(path.as_str());
            if !inspected_path.exists() {
                panic!("Provided path does not exist");
            }
        }

        let paths = self.paths.clone();
        for path in &paths {
            self.inspect(path);
        }
    }

    fn inspect(&mut self, path: &String) {
        let str_path = path.clone();
        let path = Path::new(&path);
        println!("Inspecting [{}]", str_path);
        if path.is_file() {
            let mut file = File::open(path).unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            self.hasher.reset();
            self.hasher.update(&buf);
            let mut hash: [u8; 32] = [0u8; 32];
            let hasher = take(&mut self.hasher);
            hash.clone_from_slice(&hasher.finalize()[0..32]);
            if !self.uniques.contains_key(&hash) {
                println!("Adding to unique files list: [{}]", str_path);
                let metadata = path.metadata().expect("Cannot get metedata");
                let date = metadata.created().expect("Cannot get created date");
                self.uniques.insert(hash, FileRecord::new(str_path, date));
                return;
            }
            if !self.duplicates.contains_key(&hash) {
                let metadata = path.metadata().expect("Cannot get metedata");
                let date = metadata.created().expect("Cannot get created date");
                self.duplicates
                    .insert(hash, vec![FileRecord::new(str_path, date)]);
                return;
            }
            self.duplicates
                .get_mut(&hash)
                .expect("Key does not exists")
                .push(str_path);
            return;
        } else if path.is_dir() {
            for item in path.read_dir().expect("Failed to read dir") {
                if let Ok(item) = item {
                    let item_path = String::from(item.path().to_str().unwrap());
                    self.inspect(&item_path);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hash_test() {
        let mut hasher = Sha256::new();
        let mut f = File::open(Path::new("hw.txt")).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        hasher.update(&buf);
        let hash = hasher.finalize();
        let expected = hex!("a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447");
        let value = &hash[0..32];
        assert_eq!(hash[..], expected[..]);
        assert_eq!(value, expected);
    }
}
