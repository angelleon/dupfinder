use chrono::offset::Local;
use chrono::DateTime;
use hex::encode;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::io::{ErrorKind as IOError, Read};
use std::mem::{swap, take};
use std::path::PathBuf;
use std::time::SystemTime;

type Hash = [u8; 32];

struct FileRecord {
    pub path: PathBuf,
    pub created_at: SystemTime,
}

impl FileRecord {
    pub fn new(path: PathBuf, created_at: SystemTime) -> Self {
        Self { path, created_at }
    }
}

struct ProgressDisplay {
    str_path: String,
    total_bytes: u64,
    byte_count: u64,
    enabled: bool,
    first_call: bool,
    printed_chars: usize,
}

impl ProgressDisplay {
    pub fn new(str_path: String, total_bytes: u64, byte_count: u64, enabled: bool) -> Self {
        Self {
            str_path,
            total_bytes,
            byte_count,
            enabled,
            first_call: true,
            printed_chars: 0usize,
        }
    }

    pub fn update(&mut self, bytes_read: u64) {
        if !self.enabled {
            return;
        }

        if bytes_read == 0 {
            self.byte_count = self.total_bytes;
        }
        

        self.byte_count += bytes_read;

        let mut read_status = format!("{}/{} bytes", self.byte_count, self.total_bytes);
        
        let backspace_count = self.printed_chars;
        self.printed_chars = read_status.len();

        if self.first_call {
            print!("Inspecting file [{}] ", self.str_path);
            self.first_call = false;
        }
        

        let backspaces = std::iter::repeat("\x08")
            .take(backspace_count)
            .collect::<String>();
        read_status = format!(
            "{}{}/{} bytes",
            backspaces, self.byte_count, self.total_bytes
        );

        print!("{}", read_status);
        if self.byte_count == self.total_bytes {
            println!();
        } 
    }

    pub fn read_error(&mut self, error: &std::io::Error) {
        println!(
            "Error reading file [{}], message is {}",
            self.str_path,
            error.to_string()
        );
    }
}

pub struct DupFinder {
    paths: Vec<PathBuf>,
    hasher: Sha256,
    uniques: HashMap<Hash, FileRecord>,
    duplicates: HashMap<Hash, Vec<FileRecord>>,
    min_size: u64,
    max_size: u64,
    verbose: bool,
}

impl DupFinder {
    pub fn new(paths: Vec<PathBuf>, min_size: u64, max_size: u64, verbose: bool) -> Self {
        Self {
            paths,
            hasher: Sha256::new(),
            uniques: HashMap::new(),
            duplicates: HashMap::new(),
            min_size,
            max_size,
            verbose,
        }
    }

    pub fn run(&mut self) {
        for path in &self.paths {
            if !path.exists() {
                panic!("Provided path does not exist");
            }
        }
        for i in 0..self.paths.len() {
            let path = self.paths[i].clone();
            self.inspect(path);
        }
        self.display_results();
    }

    fn display_results(&self) {
        println!("===================================================================================================================================");
        for (hash, records) in self.duplicates.iter() {
            let hash_str = encode(&hash[..6]);
            println!("Report for hash [{}]", hash_str);
            let oldest_file = self
                .uniques
                .get(hash)
                .expect("Cannot access to oldest file record");
            println!("Oldest file: {}", oldest_file.path.to_string_lossy());
            for record in records {
                let datetime: DateTime<Local> = record.created_at.into();
                println!(
                    "{} [{}]",
                    datetime.format("%F %T"),
                    record.path.to_string_lossy()
                );
            }
            println!("=");
        }
    }

    fn inspect_file(&mut self, path: PathBuf, str_path: Cow<str>) {
        let mut file: File;
        match File::open(path.clone()) {
            Ok(f) => file = f,
            Err(error) => {
                if error.kind() == IOError::PermissionDenied {
                    eprintln!("Read permission denied for file [{}]", str_path);
                } else {
                    panic!("{:?}", error);
                }
                return;
            }
        }

        let metadata = path.metadata().expect("Cannot get metedata");
        let total_bytes = metadata.len();

        if total_bytes < self.min_size || total_bytes > self.max_size {
            return;
        }

        let mut byte_count = 0u64;
        let date = metadata.created().expect("Cannot get created date");

        let mut read_count = 0u64;
        let mut buf = vec![0u8; 65535];

        let mut progress = ProgressDisplay::new(
            path.clone().to_string_lossy().into_owned(),
            total_bytes,
            byte_count,
            self.verbose,
        );

        self.hasher.reset();
        loop {
            match file.read(&mut buf) {
                Ok(n) => {
                    byte_count += n as u64;
                    read_count += 1;
                    if read_count % 16 == 0 {
                        progress.update(n as u64);
                    }
                    //progress.update(n as u64);
                    if n == 0 {
                        progress.update(0u64);
                        assert!(byte_count == total_bytes);
                        break;
                    } else {
                        self.hasher.update(&buf);
                    }
                }
                Err(error) => {
                    progress.read_error(&error);
                    break;
                }
            }
        }

        let mut hash: [u8; 32] = [0u8; 32];
        let hasher = take(&mut self.hasher);
        hash.clone_from_slice(&hasher.finalize()[0..32]);

        if !self.uniques.contains_key(&hash) {
            self.uniques.insert(hash, FileRecord::new(path, date));
            return;
        }

        let uniq_record = self.uniques.get_mut(&hash).expect("Cannot get record");
        let mut duplicate_record = FileRecord::new(path, date);
        if duplicate_record.created_at < uniq_record.created_at {
            swap(uniq_record, &mut duplicate_record);
        }

        if !self.duplicates.contains_key(&hash) {
            self.duplicates.insert(hash, vec![duplicate_record]);
            return;
        }
        self.duplicates
            .get_mut(&hash)
            .expect("Key does not exists")
            .push(duplicate_record);
        return;
    }

    fn inspect_dir(&mut self, path: PathBuf) {
        let dir_content: ReadDir;
        let path_copy = path.clone();
        let str_path = path_copy.to_string_lossy();
        match path.read_dir() {
            Ok(content) => dir_content = content,
            Err(error) => {
                if error.kind() == IOError::PermissionDenied {
                    eprintln!("Read permission denied for folder [{}]", str_path);
                } else {
                    panic!("{}", error.to_string());
                }
                return;
            }
        }
        for item in dir_content {
            if let Ok(item) = item {
                let item_path = item.path();
                self.inspect(item_path);
            }
        }
    }

    fn inspect(&mut self, path: PathBuf) {
        let cpy = path.clone();
        let str_path = cpy.to_string_lossy();
        if path.is_symlink() {
            //println!("Ignoring symlink [{}]", str_path);
            return;
        }
        if path.is_file() {
            return self.inspect_file(path, str_path);
        } else if path.is_dir() {
            return self.inspect_dir(path);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn hash_test() {
        let mut hasher = Sha256::new();
        let mut f = File::open(PathBuf::from("hw.txt")).unwrap();
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
