#![cfg(not(target_arch = "wasm32"))]

use tempfile::tempdir;
use versedb::{Database, RocksDbDatabase};

#[tokio::test]
async fn test_rocksdb_basic_operations() -> anyhow::Result<()> {
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
async fn test_rocksdb_range_operations() -> anyhow::Result<()> {
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
async fn test_rocksdb_data_persistence() -> anyhow::Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let path = temp_dir.path().to_str().unwrap();

    // Open a database and add data
    {
        let mut db = RocksDbDatabase::open(path).await?;
        db.add(b"persistent_key", b"persistent_value").await?;
        db.flush().await?; // Ensure data is written to disk
        db.close().await?; // Close explicitly
    }

    // Open the same database again and verify data persists
    {
        let db = RocksDbDatabase::open(path).await?;
        let value = db.select(b"persistent_key").await?;
        assert_eq!(value, Some(b"persistent_value".to_vec()));
    }

    Ok(())
}

#[tokio::test]
async fn test_rocksdb_concurrent_operations() -> anyhow::Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let path = temp_dir.path().to_str().unwrap();

    // Open a database
    let db1 = RocksDbDatabase::open(path).await?;

    // Clone the database for concurrent access
    let mut db2 = db1.clone();

    // Add data with one instance
    let mut db1 = db1;
    db1.add(b"shared_key", b"original_value").await?;

    // Read back with the clone
    let value = db2.select(b"shared_key").await?;
    assert_eq!(value, Some(b"original_value".to_vec()));

    // Update with the clone
    db2.add(b"shared_key", b"updated_value").await?;

    // Verify update is visible to original instance
    let updated = db1.select(b"shared_key").await?;
    assert_eq!(updated, Some(b"updated_value".to_vec()));

    // Only need to close one instance since they share the same DB
    db1.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_rocksdb_large_values() -> anyhow::Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let path = temp_dir.path().to_str().unwrap();

    // Open a database
    let mut db = RocksDbDatabase::open(path).await?;

    // Create a large value (1MB)
    let large_value = vec![0x55; 1024 * 1024];

    // Store and retrieve the large value
    db.add(b"large_key", &large_value).await?;
    let retrieved = db.select(b"large_key").await?;

    assert_eq!(retrieved, Some(large_value));

    // Clean up
    db.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_rocksdb_binary_keys() -> anyhow::Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let path = temp_dir.path().to_str().unwrap();

    // Open a database
    let mut db = RocksDbDatabase::open(path).await?;

    // Create binary keys with non-UTF8 data
    let binary_key1 = vec![0xFF, 0x00, 0x55, 0xAA, 0x11, 0x22];
    let binary_key2 = vec![0xFF, 0x00, 0x55, 0xAA, 0x11, 0x23]; // Just one byte different

    // Store with binary keys
    db.add(&binary_key1, b"binary_value_1").await?;
    db.add(&binary_key2, b"binary_value_2").await?;

    // Retrieve with binary keys
    let value1 = db.select(&binary_key1).await?;
    let value2 = db.select(&binary_key2).await?;

    assert_eq!(value1, Some(b"binary_value_1".to_vec()));
    assert_eq!(value2, Some(b"binary_value_2".to_vec()));

    // Test range query with binary keys
    let range_results = db.select_range(&binary_key1, &binary_key2).await?;
    assert_eq!(range_results.len(), 1);
    assert_eq!(range_results[0].0, binary_key1);
    assert_eq!(range_results[0].1, b"binary_value_1".to_vec());

    // Clean up
    db.close().await?;

    Ok(())
}
