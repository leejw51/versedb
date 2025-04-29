use anyhow::anyhow;
use js_sys::Uint8Array;
use std::error::Error;
use versedb::database::{Database, Result};
use versedb::idb::IdbDatabaseWrapper;
use wasm_bindgen::prelude::*;

pub struct VerseDb {
    db: IdbDatabaseWrapper,
}

impl VerseDb {
    pub async fn new(db_name: &str) -> anyhow::Result<Self> {
        let db = IdbDatabaseWrapper::open(db_name)
            .await
            .map_err(|e| anyhow!("Failed to open database: {}", e))?;
        Ok(Self { db })
    }

    pub async fn store(&mut self, key: &[u8], value: &[u8]) -> anyhow::Result<()> {
        self.db
            .add(key, value)
            .await
            .map_err(|e| anyhow!("Failed to store data: {}", e))
    }

    pub async fn get(&self, key: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        self.db
            .select(key)
            .await
            .map_err(|e| anyhow!("Failed to get data: {}", e))
    }

    pub async fn delete(&mut self, key: &[u8]) -> anyhow::Result<()> {
        self.db
            .remove(key)
            .await
            .map_err(|e| anyhow!("Failed to delete data: {}", e))
    }

    pub async fn get_range(
        &self,
        start: &[u8],
        end: &[u8],
    ) -> anyhow::Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.db
            .select_range(start, end)
            .await
            .map_err(|e| anyhow!("Failed to get range: {}", e))
    }
}

// Helper functions for string conversion
pub fn string_to_bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

pub fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
