use super::database::{Database, Result};
use async_trait::async_trait;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub struct JsonDatabase {
    data: Mutex<BTreeMap<Vec<u8>, Value>>,
    path: String,
}

impl Clone for JsonDatabase {
    fn clone(&self) -> Self {
        Self {
            data: Mutex::new(self.data.lock().unwrap().clone()),
            path: self.path.clone(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for JsonDatabase {
    async fn open(path: &str) -> Result<Self> {
        let data = if Path::new(path).exists() {
            let contents = fs::read_to_string(path)?;
            if contents.trim().is_empty() {
                BTreeMap::new()
            } else {
                let json: Map<String, Value> = serde_json::from_str(&contents)?;
                let mut btree = BTreeMap::new();

                for (key, value) in json {
                    btree.insert(key.as_bytes().to_vec(), value);
                }

                btree
            }
        } else {
            BTreeMap::new()
        };

        Ok(JsonDatabase {
            data: Mutex::new(data),
            path: path.to_string(),
        })
    }

    async fn close(&mut self) -> Result<()> {
        self.flush().await
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let value_str = String::from_utf8(value.to_vec())?;
        let value_json = Value::String(value_str);
        self.data.lock().unwrap().insert(key.to_vec(), value_json);
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self
            .data
            .lock()
            .unwrap()
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.as_bytes().to_vec())))
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.data.lock().unwrap().remove(key);
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();
        let data = self.data.lock().unwrap();

        for (key, value) in data.range(start.to_vec()..end.to_vec()) {
            if let Some(str_value) = value.as_str() {
                result.push((key.clone(), str_value.as_bytes().to_vec()));
            }
        }

        Ok(result)
    }

    async fn remove_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();
        let mut data = self.data.lock().unwrap();

        let keys_to_remove: Vec<Vec<u8>> = data
            .range(start.to_vec()..end.to_vec())
            .filter_map(|(key, value)| {
                value
                    .as_str()
                    .map(|str_value| (key.clone(), str_value.as_bytes().to_vec()))
            })
            .inspect(|pair| result.push(pair.clone()))
            .map(|(key, _)| key)
            .collect();

        // Remove the collected keys
        for key in keys_to_remove {
            data.remove(&key);
        }

        Ok(result)
    }

    async fn flush(&mut self) -> Result<()> {
        let mut json_map = Map::new();
        let data = self.data.lock().unwrap();

        for (key, value) in data.iter() {
            let key_str = String::from_utf8(key.clone())?;
            json_map.insert(key_str, value.clone());
        }

        let json_string = serde_json::to_string_pretty(&json_map)?;
        fs::write(&self.path, json_string)?;

        Ok(())
    }
}

// SAFETY: JsonDatabase is safe to share between threads because data access is protected by Mutex
unsafe impl Send for JsonDatabase {}
unsafe impl Sync for JsonDatabase {}
