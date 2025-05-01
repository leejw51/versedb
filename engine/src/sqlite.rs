use super::database::{Database, Result};
use async_trait::async_trait;
use rusqlite::{Connection, params};
use std::cell::UnsafeCell;
use std::error::Error;
use std::sync::Mutex;

pub struct SqliteDatabase {
    conn: UnsafeCell<Mutex<Connection>>,
}

impl SqliteDatabase {
    // Helper method to safely get mutable access to connection
    fn get_conn(&self) -> &Mutex<Connection> {
        unsafe { &*self.conn.get() }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for SqliteDatabase {
    async fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Create the table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value BLOB
            )",
            [],
        )?;

        Ok(SqliteDatabase {
            conn: UnsafeCell::new(Mutex::new(conn)),
        })
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let conn = self.get_conn().lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value) VALUES (?, ?)",
            params![String::from_utf8_lossy(key), value],
        )?;
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let conn = self.get_conn().lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM kv_store WHERE key = ?")?;
        let key_str = String::from_utf8_lossy(key);

        let result = stmt.query_row([&key_str], |row| {
            let value: Vec<u8> = row.get(0)?;
            Ok(value)
        });

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        let conn = self.get_conn().lock().unwrap();
        conn.execute(
            "DELETE FROM kv_store WHERE key = ?",
            params![String::from_utf8_lossy(key)],
        )?;
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let conn = self.get_conn().lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT key, value FROM kv_store WHERE key >= ? AND key < ? ORDER BY key")?;

        let start_str = String::from_utf8_lossy(start);
        let end_str = String::from_utf8_lossy(end);

        let rows = stmt.query_map([&start_str, &end_str], |row| {
            let key: String = row.get(0)?;
            let value: Vec<u8> = row.get(1)?;
            Ok((key.into_bytes(), value))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    async fn remove_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        // First get all entries that will be removed
        let entries = self.select_range(start, end).await?;

        // Then delete the range
        let conn = self.get_conn().lock().unwrap();
        conn.execute(
            "DELETE FROM kv_store WHERE key >= ? AND key < ?",
            params![String::from_utf8_lossy(start), String::from_utf8_lossy(end)],
        )?;

        Ok(entries)
    }

    async fn flush(&mut self) -> Result<()> {
        let conn = self.get_conn().lock().unwrap();
        conn.execute("PRAGMA wal_checkpoint(FULL)", [])?;
        Ok(())
    }
}

// SAFETY: SqliteDatabase is safe to share between threads because Connection access is protected by Mutex
unsafe impl Send for SqliteDatabase {}
unsafe impl Sync for SqliteDatabase {}
