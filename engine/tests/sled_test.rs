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
}
