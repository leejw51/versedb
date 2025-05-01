use super::database::{Database, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct MemoryDatabase {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
}

impl Clone for MemoryDatabase {
    fn clone(&self) -> Self {
        Self {
            data: Mutex::new(self.data.lock().unwrap().clone()),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for MemoryDatabase {
    async fn open(_path: &str) -> Result<Self> {
        Ok(Self {
            data: Mutex::new(HashMap::new()),
        })
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.data
            .lock()
            .unwrap()
            .insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.data.lock().unwrap().get(key).cloned())
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.data.lock().unwrap().remove(key);
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();
        let start_vec = start.to_vec();
        let end_vec = end.to_vec();
        let data = self.data.lock().unwrap();
        for (key, value) in data.iter() {
            if key >= &start_vec && key < &end_vec {
                result.push((key.clone(), value.clone()));
            }
        }
        Ok(result)
    }

    async fn remove_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();
        let start_vec = start.to_vec();
        let end_vec = end.to_vec();
        let mut data = self.data.lock().unwrap();

        // Collect keys to remove and their values
        let keys_to_remove: Vec<Vec<u8>> = data
            .iter()
            .filter(|(key, _)| *key >= &start_vec && *key < &end_vec)
            .map(|(key, value)| {
                result.push((key.clone(), value.clone()));
                key.clone()
            })
            .collect();

        // Remove the collected keys
        for key in keys_to_remove {
            data.remove(&key);
        }

        Ok(result)
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// SAFETY: MemoryDatabase is safe to share between threads because data access is protected by Mutex
unsafe impl Send for MemoryDatabase {}
unsafe impl Sync for MemoryDatabase {}
