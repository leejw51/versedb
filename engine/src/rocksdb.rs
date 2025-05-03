use super::database::{Database, Result};
use async_trait::async_trait;
use rocksdb::{ColumnFamilyDescriptor, DB, IteratorMode, Options, ReadOptions, WriteOptions};
use std::sync::{Arc, Mutex};

pub struct RocksDbDatabase {
    db: Arc<Mutex<DB>>,
    path: String,
}

impl Clone for RocksDbDatabase {
    fn clone(&self) -> Self {
        // Just clone the Arc, which will share the same DB instance
        Self {
            db: Arc::clone(&self.db),
            path: self.path.clone(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Database for RocksDbDatabase {
    async fn open(path: &str) -> Result<Self> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.set_keep_log_file_num(10);
        options.set_max_total_wal_size(64 * 1024 * 1024); // 64MB
        options.set_write_buffer_size(64 * 1024 * 1024); // 64MB

        let db = DB::open(&options, path)?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            path: path.to_string(),
        })
    }

    async fn close(&mut self) -> Result<()> {
        // RocksDB doesn't have an explicit close method
        // Dropping the DB instance will close it automatically
        // We can flush to ensure all data is persisted
        self.db.lock().unwrap().flush()?;
        Ok(())
    }

    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let write_opts = WriteOptions::default();
        self.db.lock().unwrap().put_opt(key, value, &write_opts)?;
        Ok(())
    }

    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let read_opts = ReadOptions::default();
        match self.db.lock().unwrap().get_opt(key, &read_opts)? {
            Some(value) => Ok(Some(value)),
            None => Ok(None),
        }
    }

    async fn remove(&mut self, key: &[u8]) -> Result<()> {
        let write_opts = WriteOptions::default();
        self.db.lock().unwrap().delete_opt(key, &write_opts)?;
        Ok(())
    }

    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut result = Vec::new();
        let db = self.db.lock().unwrap();

        let mut read_opts = ReadOptions::default();
        read_opts.set_iterate_lower_bound(start.to_vec());
        read_opts.set_iterate_upper_bound(end.to_vec());

        let iter = db.iterator_opt(IteratorMode::Start, read_opts);
        for item in iter {
            let (key, value) = item?;
            // Since the iterator might return keys outside our range despite the bounds,
            // we double-check the key is in our desired range
            if key.as_ref() >= start && key.as_ref() < end {
                result.push((key.to_vec(), value.to_vec()));
            }
        }

        Ok(result)
    }

    async fn remove_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        // First collect all items in range
        let items = self.select_range(start, end).await?;

        if !items.is_empty() {
            let db = self.db.lock().unwrap();
            let write_opts = WriteOptions::default();

            // Using a WriteBatch for better performance
            let mut batch = rocksdb::WriteBatch::default();

            for (key, _) in &items {
                batch.delete(key);
            }

            // Execute the batch delete
            db.write_opt(batch, &write_opts)?;
        }

        Ok(items)
    }

    async fn flush(&mut self) -> Result<()> {
        self.db.lock().unwrap().flush()?;
        Ok(())
    }
}

// SAFETY: RocksDbDatabase is safe to share between threads because data access is protected by Mutex
unsafe impl Send for RocksDbDatabase {}
unsafe impl Sync for RocksDbDatabase {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_basic_operations() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = tempdir()?;
        let path = temp_dir.path().to_str().unwrap();

        // Open a new database
        let mut db = RocksDbDatabase::open(path).await?;

        // Test adding and selecting
        let key = b"test_key";
        let value = b"test_value";
        db.add(key, value).await?;

        let retrieved = db.select(key).await?;
        assert_eq!(retrieved, Some(value.to_vec()));

        // Test non-existent key
        let non_existent = db.select(b"non_existent").await?;
        assert_eq!(non_existent, None);

        // Test removing
        db.remove(key).await?;
        let after_remove = db.select(key).await?;
        assert_eq!(after_remove, None);

        // Close the database
        db.close().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_range_operations() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = tempdir()?;
        let path = temp_dir.path().to_str().unwrap();

        // Open a new database
        let mut db = RocksDbDatabase::open(path).await?;

        // Add several key-value pairs
        for i in 0..10 {
            let key = format!("key_{:02}", i).into_bytes();
            let value = format!("value_{}", i).into_bytes();
            db.add(&key, &value).await?;
        }

        // Test select_range
        let start = b"key_03";
        let end = b"key_07";
        let range_results = db.select_range(start, end).await?;

        assert_eq!(range_results.len(), 4); // key_03, key_04, key_05, key_06

        // Verify the content of the range
        for (i, (key, value)) in range_results.iter().enumerate() {
            let expected_key = format!("key_{:02}", i + 3).into_bytes();
            let expected_value = format!("value_{}", i + 3).into_bytes();
            assert_eq!(key, &expected_key);
            assert_eq!(value, &expected_value);
        }

        // Test remove_range
        let removed = db.remove_range(start, end).await?;
        assert_eq!(removed.len(), 4);

        // Verify keys were removed
        for i in 3..7 {
            let key = format!("key_{:02}", i).into_bytes();
            let result = db.select(&key).await?;
            assert_eq!(result, None);
        }

        // Verify keys outside the range still exist
        for i in vec![0, 1, 2, 7, 8, 9] {
            let key = format!("key_{:02}", i).into_bytes();
            let result = db.select(&key).await?;
            assert_eq!(result, Some(format!("value_{}", i).into_bytes()));
        }

        // Close the database
        db.close().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_clone_and_concurrent_access() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = tempdir()?;
        let path = temp_dir.path().to_str().unwrap();

        // Open a new database and add some data
        let mut db1 = RocksDbDatabase::open(path).await?;
        db1.add(b"key1", b"value1").await?;

        // Clone the database and verify both instances can access the data
        let db2 = db1.clone();

        let result1 = db1.select(b"key1").await?;
        let result2 = db2.select(b"key1").await?;

        assert_eq!(result1, Some(b"value1".to_vec()));
        assert_eq!(result2, Some(b"value1".to_vec()));

        // Close both databases
        db1.close().await?;

        Ok(())
    }
}
