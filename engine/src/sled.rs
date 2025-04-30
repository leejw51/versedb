use crate::database::{Database, Result};
use async_trait::async_trait;
use sled::Db;

#[derive(Clone)]
pub struct SledDatabase {
    db: Db,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for SledDatabase {
    async fn open(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(SledDatabase { db })
    }

    async fn close(&mut self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.insert(key, value)?;
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key)?.map(|v| v.to_vec()))
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.db.remove(key)?;
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let range = self.db.range(start..end);
        let mut result = Vec::new();
        for item in range {
            let (key, value) = item?;
            result.push((key.to_vec(), value.to_vec()));
        }
        Ok(result)
    }
}
