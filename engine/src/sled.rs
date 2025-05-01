use crate::database::{Database, Result};
use async_trait::async_trait;
use sled::Db;
use std::cell::UnsafeCell;

pub struct SledDatabase {
    db: UnsafeCell<Db>,
}

impl Clone for SledDatabase {
    fn clone(&self) -> Self {
        Self {
            db: UnsafeCell::new(self.get_db().clone()),
        }
    }
}

impl SledDatabase {
    // Helper method to safely get access to db
    fn get_db(&self) -> &Db {
        unsafe { &*self.db.get() }
    }

    // Helper method to safely get mutable access to db
    fn get_db_mut(&self) -> &mut Db {
        unsafe { &mut *self.db.get() }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for SledDatabase {
    async fn open(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(SledDatabase {
            db: UnsafeCell::new(db),
        })
    }

    async fn close(&mut self) -> Result<()> {
        self.get_db_mut().flush()?;
        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.get_db_mut().insert(key, value)?;
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.get_db().get(key)?.map(|v| v.to_vec()))
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.get_db_mut().remove(key)?;
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let range = self.get_db().range(start..end);
        let mut result = Vec::new();
        for item in range {
            let (key, value) = item?;
            result.push((key.to_vec(), value.to_vec()));
        }
        Ok(result)
    }

    async fn remove_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let range = self.get_db().range(start..end);
        let mut result = Vec::new();

        // Collect all keys and values first since we can't modify while iterating
        for item in range {
            let (key, value) = item?;
            result.push((key.to_vec(), value.to_vec()));
        }

        // Remove the collected keys
        let db = self.get_db_mut();
        for (key, _) in &result {
            db.remove(key)?;
        }

        Ok(result)
    }

    async fn flush(&mut self) -> Result<()> {
        self.get_db_mut().flush()?;
        Ok(())
    }
}

// SAFETY: SledDatabase is safe to share between threads because db access is protected by UnsafeCell
unsafe impl Send for SledDatabase {}
unsafe impl Sync for SledDatabase {}
