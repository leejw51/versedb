#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use tempfile::NamedTempFile;
use versedb::database::Database;
use versedb::json::JsonDatabase;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_json_database_basic_operations() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Test open and add
    let mut db = JsonDatabase::open(path).await.unwrap();
    db.add(b"key1", b"value1").await.unwrap();
    db.add(b"key2", b"value2").await.unwrap();
    db.close().await.unwrap();

    // Test reopen and select
    let db = JsonDatabase::open(path).await.unwrap();
    assert_eq!(db.select(b"key1").await.unwrap(), Some(b"value1".to_vec()));
    assert_eq!(db.select(b"key2").await.unwrap(), Some(b"value2".to_vec()));
    assert_eq!(db.select(b"nonexistent").await.unwrap(), None);
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_json_database_remove() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let mut db = JsonDatabase::open(path).await.unwrap();
    db.add(b"key1", b"value1").await.unwrap();
    db.add(b"key2", b"value2").await.unwrap();

    // Test remove
    db.remove(b"key1").await.unwrap();
    assert_eq!(db.select(b"key1").await.unwrap(), None);
    assert_eq!(db.select(b"key2").await.unwrap(), Some(b"value2".to_vec()));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_json_database_range_select() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let mut db = JsonDatabase::open(path).await.unwrap();
    db.add(b"a", b"1").await.unwrap();
    db.add(b"b", b"2").await.unwrap();
    db.add(b"c", b"3").await.unwrap();
    db.add(b"d", b"4").await.unwrap();

    // Test range select
    let results = db.select_range(b"b", b"d").await.unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (b"b".to_vec(), b"2".to_vec()));
    assert_eq!(results[1], (b"c".to_vec(), b"3".to_vec()));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_json_database_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Write some data and close
    {
        let mut db = JsonDatabase::open(path).await.unwrap();
        db.add(b"key1", b"value1").await.unwrap();
        db.add(b"key2", b"value2").await.unwrap();
        db.close().await.unwrap();
    }

    // Verify the file exists and contains valid JSON
    let contents = fs::read_to_string(path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert!(json.is_object());
    assert_eq!(json["key1"].as_str(), Some("value1"));
    assert_eq!(json["key2"].as_str(), Some("value2"));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_json_database_remove_range() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let mut db = JsonDatabase::open(path).await.unwrap();
    
    // Add entries with ordered keys
    let entries = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
        (b"key5", b"value5"),
    ];

    for (key, value) in &entries {
        db.add(*key, *value).await.unwrap();
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

    // Test persistence after remove_range
    db.close().await.unwrap();
    
    // Reopen and verify the changes persisted
    let db = JsonDatabase::open(path).await.unwrap();
    let all_entries = db.select_range(b"key1", b"key6").await.unwrap();
    assert_eq!(all_entries.len(), 3);
    assert!(all_entries.contains(&(b"key1".to_vec(), b"value1".to_vec())));
    assert!(all_entries.contains(&(b"key4".to_vec(), b"value4".to_vec())));
    assert!(all_entries.contains(&(b"key5".to_vec(), b"value5".to_vec())));
}
