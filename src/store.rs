use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// A simple filesystem-backed key-value store.
///
/// Each key maps to a plain file under `data_dir/`. Values are raw UTF-8 strings.
/// This mirrors what the global runnables (`store_data`, `read_data`, etc.) do in
/// shell, but provides a typed Rust API for the engine layer.
pub struct Store {
    data_dir: PathBuf,
}

impl Store {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        Store {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }

    /// Opens the default project-local store at `.ofa/data/`.
    pub fn project() -> Self {
        Self::new(".ofa/data")
    }

    /// Opens the global store at `~/.ofa/data/`.
    pub fn global() -> Option<Self> {
        let home = std::env::var("HOME").ok()?;
        Some(Self::new(PathBuf::from(home).join(".ofa/data")))
    }

    // ── Read ────────────────────────────────────────────────────────────────

    /// Read a value by key. Returns `None` if the key doesn't exist.
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let path = self.key_path(key);
        if path.exists() {
            let value =
                fs::read_to_string(&path).context(format!("Failed to read store key '{}'", key))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Returns `true` if the key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.key_path(key).exists()
    }

    /// List all keys currently in the store (sorted).
    pub fn list(&self) -> Result<Vec<String>> {
        if !self.data_dir.exists() {
            return Ok(vec![]);
        }
        let mut keys: Vec<String> = fs::read_dir(&self.data_dir)
            .context("Failed to read store directory")?
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        keys.sort();
        Ok(keys)
    }

    // ── Write ───────────────────────────────────────────────────────────────

    /// Write a value for a key, creating the data directory if needed.
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        fs::create_dir_all(&self.data_dir).context("Failed to create store directory")?;
        fs::write(self.key_path(key), value)
            .context(format!("Failed to write store key '{}'", key))
    }

    /// Delete a key. No-op if the key doesn't exist.
    pub fn delete(&self, key: &str) -> Result<()> {
        let path = self.key_path(key);
        if path.exists() {
            fs::remove_file(&path).context(format!("Failed to delete store key '{}'", key))?;
        }
        Ok(())
    }

    /// Remove all keys from the store.
    pub fn clear(&self) -> Result<()> {
        if self.data_dir.exists() {
            fs::remove_dir_all(&self.data_dir).context("Failed to clear store")?;
        }
        Ok(())
    }

    // ── Internal ────────────────────────────────────────────────────────────

    fn key_path(&self, key: &str) -> PathBuf {
        self.data_dir.join(key)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_store() -> Store {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join(format!("ofa_store_test_{}_{}", std::process::id(), id));
        Store::new(dir)
    }

    #[test]
    fn set_and_get_roundtrip() {
        let s = temp_store();
        s.set("greeting", "hello").unwrap();
        assert_eq!(s.get("greeting").unwrap(), Some("hello".to_string()));
    }

    #[test]
    fn get_missing_key_is_none() {
        let s = temp_store();
        assert_eq!(s.get("nonexistent").unwrap(), None);
    }

    #[test]
    fn delete_removes_key() {
        let s = temp_store();
        s.set("x", "1").unwrap();
        s.delete("x").unwrap();
        assert_eq!(s.get("x").unwrap(), None);
    }

    #[test]
    fn list_returns_sorted_keys() {
        let s = temp_store();
        s.set("b", "2").unwrap();
        s.set("a", "1").unwrap();
        s.set("c", "3").unwrap();
        assert_eq!(s.list().unwrap(), vec!["a", "b", "c"]);
    }

    #[test]
    fn contains_reflects_presence() {
        let s = temp_store();
        assert!(!s.contains("k"));
        s.set("k", "v").unwrap();
        assert!(s.contains("k"));
    }

    #[test]
    fn overwrite_value() {
        let s = temp_store();
        s.set("key", "first").unwrap();
        s.set("key", "second").unwrap();
        assert_eq!(s.get("key").unwrap(), Some("second".to_string()));
    }

    #[test]
    fn clear_empties_store() {
        let s = temp_store();
        s.set("a", "1").unwrap();
        s.set("b", "2").unwrap();
        s.clear().unwrap();
        assert_eq!(s.list().unwrap(), Vec::<String>::new());
    }
}
