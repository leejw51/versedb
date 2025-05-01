use super::database::{Database, Result};
use async_trait::async_trait;
use serde_yaml::{self, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub struct YamlDatabase {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    path: String,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for YamlDatabase {
    async fn open(path: &str) -> Result<Self> {
        let mut data = HashMap::new();

        if Path::new(path).exists() {
            let contents = fs::read_to_string(path)?;
            if !contents.trim().is_empty() {
                let yaml: Value = serde_yaml::from_str(&contents)?;
                if let Value::Mapping(map) = yaml {
                    for (key, value) in map {
                        if let (Value::String(k), Value::String(v)) = (key, value) {
                            data.insert(k.as_bytes().to_vec(), v.as_bytes().to_vec());
                        }
                    }
                }
            }
        }

        Ok(Self {
            data: Mutex::new(data),
            path: path.to_string(),
        })
    }

    async fn close(&mut self) -> Result<()> {
        self.flush().await
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
        let mut map = serde_yaml::Mapping::new();
        let data = self.data.lock().unwrap();

        for (key, value) in data.iter() {
            let key_str = String::from_utf8(key.clone())?;
            let value_str = String::from_utf8(value.clone())?;
            map.insert(Value::String(key_str), Value::String(value_str));
        }

        let yaml = Value::Mapping(map);
        let yaml_str = serde_yaml::to_string(&yaml)?;
        fs::write(&self.path, yaml_str)?;

        Ok(())
    }
}

// SAFETY: YamlDatabase is safe to share between threads because data access is protected by Mutex
unsafe impl Send for YamlDatabase {}
unsafe impl Sync for YamlDatabase {}
