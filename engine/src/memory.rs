use super::database::{Database, Result};
use async_trait::async_trait;
use std::cell::UnsafeCell;
use std::collections::HashMap;

pub struct MemoryDatabase {
    data: UnsafeCell<HashMap<Vec<u8>, Vec<u8>>>,
}

impl Clone for MemoryDatabase {
    fn clone(&self) -> Self {
        Self {
            data: UnsafeCell::new(self.get_data().clone()),
        }
    }
}

impl MemoryDatabase {
    // Helper method to safely get access to data
    fn get_data(&self) -> &HashMap<Vec<u8>, Vec<u8>> {
        unsafe { &*self.data.get() }
    }

    // Helper method to safely get mutable access to data
    fn get_data_mut(&self) -> &mut HashMap<Vec<u8>, Vec<u8>> {
        unsafe { &mut *self.data.get() }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for MemoryDatabase {
    async fn open(_path: &str) -> Result<Self> {
        Ok(Self {
            data: UnsafeCell::new(HashMap::new()),
        })
    }

    async fn close(&mut self) -> Result<()> {
        // No need to do anything for memory database
        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.get_data_mut().insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.get_data().get(key).cloned())
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.get_data_mut().remove(key);
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();
        let start_vec = start.to_vec();
        let end_vec = end.to_vec();
        for (key, value) in self.get_data().iter() {
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

        // Collect keys to remove and their values
        let keys_to_remove: Vec<Vec<u8>> = self
            .get_data()
            .iter()
            .filter(|(key, _)| *key >= &start_vec && *key < &end_vec)
            .map(|(key, value)| {
                result.push((key.clone(), value.clone()));
                key.clone()
            })
            .collect();

        // Remove the collected keys
        let data = self.get_data_mut();
        for key in keys_to_remove {
            data.remove(&key);
        }

        Ok(result)
    }

    async fn flush(&mut self) -> Result<()> {
        // No need to flush for memory database
        Ok(())
    }
}

// SAFETY: MemoryDatabase is safe to share between threads because data access is protected by UnsafeCell
unsafe impl Send for MemoryDatabase {}
unsafe impl Sync for MemoryDatabase {}
