use super::database::{Database, Result};
use async_trait::async_trait;
use sled::Db;
use std::sync::Mutex;

pub struct SledDatabase {
    db: Mutex<Db>,
}

impl Clone for SledDatabase {
    fn clone(&self) -> Self {
        Self {
            db: Mutex::new(self.db.lock().unwrap().clone()),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for SledDatabase {
    async fn open(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db: Mutex::new(db) })
    }

    async fn close(&mut self) -> Result<()> {
        self.db.lock().unwrap().flush()?;
        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.lock().unwrap().insert(key, value)?;
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.db.lock().unwrap().get(key)?.map(|v| v.to_vec()))
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.db.lock().unwrap().remove(key)?;
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();
        let db = self.db.lock().unwrap();
        for item in db.range(start..end) {
            let (key, value) = item?;
            result.push((key.to_vec(), value.to_vec()));
        }
        Ok(result)
    }

    async fn remove_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();
        let db = self.db.lock().unwrap();

        // First collect all items in range
        let items_to_remove: Vec<(Vec<u8>, Vec<u8>)> = db
            .range(start..end)
            .filter_map(|res| res.ok())
            .map(|(key, value)| (key.to_vec(), value.to_vec()))
            .collect();

        // Then remove them and build result
        for (key, value) in &items_to_remove {
            db.remove(key)?;
            result.push((key.clone(), value.clone()));
        }

        Ok(result)
    }

    async fn flush(&mut self) -> Result<()> {
        self.db.lock().unwrap().flush()?;
        Ok(())
    }
}

// SAFETY: SledDatabase is safe to share between threads because data access is protected by Mutex
unsafe impl Send for SledDatabase {}
unsafe impl Sync for SledDatabase {}
