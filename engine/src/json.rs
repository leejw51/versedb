use super::database::{Database, Result};
use async_trait::async_trait;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct JsonDatabase {
    data: BTreeMap<Vec<u8>, Value>,
    path: String,
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
            data,
            path: path.to_string(),
        })
    }

    async fn close(&mut self) -> Result<()> {
        let mut json_map = Map::new();

        for (key, value) in &self.data {
            let key_str = String::from_utf8(key.clone())?;
            json_map.insert(key_str, value.clone());
        }

        let json_string = serde_json::to_string_pretty(&json_map)?;
        fs::write(&self.path, json_string)?;

        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let value_str = String::from_utf8(value.to_vec())?;
        let value_json = Value::String(value_str);
        self.data.insert(key.to_vec(), value_json);
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self
            .data
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.as_bytes().to_vec())))
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.data.remove(key);
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();

        for (key, value) in self.data.range(start.to_vec()..end.to_vec()) {
            if let Some(str_value) = value.as_str() {
                result.push((key.clone(), str_value.as_bytes().to_vec()));
            }
        }

        Ok(result)
    }
}
