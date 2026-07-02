use std::path::{Path, PathBuf};

use anyhow::Result;

pub struct Store {
    data_dir: PathBuf,
}

impl Store {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        Store {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }

    pub fn get(&self) -> Result<String> {
        Ok(self.data_dir.to_str().unwrap().to_string())
    }
}
