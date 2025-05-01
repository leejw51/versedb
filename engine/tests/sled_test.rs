#[cfg(not(target_arch = "wasm32"))]
mod sled_tests {
    use std::fs;
    use tempfile::tempdir;
    use versedb::database::Database;
    use versedb::sled::SledDatabase;

    #[tokio::test]
    async fn test_sled_database_operations() {
        // Create a temporary directory for the database
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();

        // Test opening the database
        let mut db = SledDatabase::open(db_path).await.unwrap();

        // Test adding data
        let key = b"test_key";
        let value = b"test_value";
        db.add(key, value).await.unwrap();

        // Test selecting data
        let retrieved = db.select(key).await.unwrap().unwrap();
        assert_eq!(retrieved, value);

        // Test removing data
        db.remove(key).await.unwrap();
        let removed = db.select(key).await.unwrap();
        assert!(removed.is_none());

        // Test closing the database
        db.close().await.unwrap();

        // Clean up
        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_sled_database_multiple_operations() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let mut db = SledDatabase::open(db_path).await.unwrap();

        // Test multiple insertions
        let test_data = vec![
            (b"key1".as_slice(), b"value1".as_slice()),
            (b"key2".as_slice(), b"value2".as_slice()),
            (b"key3".as_slice(), b"value3".as_slice()),
        ];

        for (key, value) in &test_data {
            db.add(key, value).await.unwrap();
        }

        // Verify all data was stored correctly
        for (key, value) in &test_data {
            let retrieved = db.select(key).await.unwrap().unwrap();
            assert_eq!(retrieved, *value);
        }

        // Clean up
        db.close().await.unwrap();
        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_sled_database_remove_range() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        let mut db = SledDatabase::open(db_path).await.unwrap();

        // Add entries with ordered keys
        let test_data = vec![
            (b"key1".as_slice(), b"value1".as_slice()),
            (b"key2".as_slice(), b"value2".as_slice()),
            (b"key3".as_slice(), b"value3".as_slice()),
            (b"key4".as_slice(), b"value4".as_slice()),
            (b"key5".as_slice(), b"value5".as_slice()),
        ];

        for (key, value) in &test_data {
            db.add(key, value).await.unwrap();
        }

        // Test remove_range from key2 to key4 (inclusive of key2, exclusive of key4)
        let removed = db.remove_range(b"key2", b"key4").await.unwrap();

        // Verify the removed entries
        assert_eq!(removed.len(), 2);
        assert!(removed.contains(&(b"key2".to_vec(), b"value2".to_vec())));
        assert!(removed.contains(&(b"key3".to_vec(), b"value3".to_vec())));

        // Verify the remaining entries
        let remaining = db.select_range(b"key1", b"key6").await.unwrap();
        assert_eq!(remaining.len(), 3);
        assert!(remaining.contains(&(b"key1".to_vec(), b"value1".to_vec())));
        assert!(remaining.contains(&(b"key4".to_vec(), b"value4".to_vec())));
        assert!(remaining.contains(&(b"key5".to_vec(), b"value5".to_vec())));

        // Verify specific entries are gone
        assert!(db.select(b"key2").await.unwrap().is_none());
        assert!(db.select(b"key3").await.unwrap().is_none());

        // Clean up
        db.close().await.unwrap();
        temp_dir.close().unwrap();
    }
}
