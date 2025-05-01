#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use tempfile::NamedTempFile;
use versedb::database::Database;
use versedb::yaml::YamlDatabase;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_yaml_database() {
    // Create a temporary file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Test open and add
    let mut db = YamlDatabase::open(path).await.unwrap();
    db.add("key1".as_bytes(), "value1".as_bytes())
        .await
        .unwrap();
    db.add("key2".as_bytes(), "value2".as_bytes())
        .await
        .unwrap();
    db.add("key3".as_bytes(), "value3".as_bytes())
        .await
        .unwrap();
    db.close().await.unwrap();

    // Test reopen and select
    let db = YamlDatabase::open(path).await.unwrap();
    assert_eq!(
        db.select("key1".as_bytes()).await.unwrap(),
        Some("value1".as_bytes().to_vec())
    );
    assert_eq!(
        db.select("key2".as_bytes()).await.unwrap(),
        Some("value2".as_bytes().to_vec())
    );
    assert_eq!(db.select("nonexistent".as_bytes()).await.unwrap(), None);

    // Test select_range (inclusive of key1, exclusive of key3)
    let mut db = YamlDatabase::open(path).await.unwrap();
    let range = db
        .select_range("key1".as_bytes(), "key3".as_bytes())
        .await
        .unwrap();
    assert_eq!(range.len(), 2);
    assert!(range.contains(&("key1".as_bytes().to_vec(), "value1".as_bytes().to_vec())));
    assert!(range.contains(&("key2".as_bytes().to_vec(), "value2".as_bytes().to_vec())));

    // Test remove
    db.remove("key1".as_bytes()).await.unwrap();
    assert_eq!(db.select("key1".as_bytes()).await.unwrap(), None);
    db.close().await.unwrap();

    // Clean up
    fs::remove_file(path).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_yaml_database_remove_range() {
    // Create a temporary file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Test open and add multiple entries
    let mut db = YamlDatabase::open(path).await.unwrap();

    // Add entries with ordered keys
    let entries = vec![
        ("key1", "value1"),
        ("key2", "value2"),
        ("key3", "value3"),
        ("key4", "value4"),
        ("key5", "value5"),
    ];

    for (key, value) in &entries {
        db.add(key.as_bytes(), value.as_bytes()).await.unwrap();
    }

    // Test remove_range from key2 to key4 (inclusive of key2, exclusive of key4)
    let removed = db
        .remove_range("key2".as_bytes(), "key4".as_bytes())
        .await
        .unwrap();

    // Verify the removed entries
    assert_eq!(removed.len(), 2);
    assert!(removed.contains(&("key2".as_bytes().to_vec(), "value2".as_bytes().to_vec())));
    assert!(removed.contains(&("key3".as_bytes().to_vec(), "value3".as_bytes().to_vec())));

    // Verify the remaining entries
    let remaining = db
        .select_range("key1".as_bytes(), "key6".as_bytes())
        .await
        .unwrap();
    assert_eq!(remaining.len(), 3);
    assert!(remaining.contains(&("key1".as_bytes().to_vec(), "value1".as_bytes().to_vec())));
    assert!(remaining.contains(&("key4".as_bytes().to_vec(), "value4".as_bytes().to_vec())));
    assert!(remaining.contains(&("key5".as_bytes().to_vec(), "value5".as_bytes().to_vec())));

    // Clean up
    fs::remove_file(path).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_yaml_database_flush() {
    // Create a temporary file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Test open and add
    let mut db = YamlDatabase::open(path).await.unwrap();

    // Add some data
    db.add("key1".as_bytes(), "value1".as_bytes())
        .await
        .unwrap();
    db.add("key2".as_bytes(), "value2".as_bytes())
        .await
        .unwrap();

    // Explicitly flush the data
    db.flush().await.unwrap();

    // Verify data is persisted
    assert_eq!(
        db.select("key1".as_bytes()).await.unwrap(),
        Some("value1".as_bytes().to_vec())
    );
    assert_eq!(
        db.select("key2".as_bytes()).await.unwrap(),
        Some("value2".as_bytes().to_vec())
    );

    // Clean up
    db.close().await.unwrap();
    fs::remove_file(path).unwrap();
}

// Additional test for YAML-specific functionality
#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_yaml_database_persistence_format() {
    // Create a temporary file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Add data and flush
    let mut db = YamlDatabase::open(path).await.unwrap();
    db.add("test_key".as_bytes(), "test_value".as_bytes())
        .await
        .unwrap();
    db.flush().await.unwrap();
    db.close().await.unwrap();

    // Read the raw file contents
    let contents = fs::read_to_string(path).unwrap();

    // Verify it's valid YAML format
    let docs = yaml_rust2::YamlLoader::load_from_str(&contents).unwrap();
    assert!(!docs.is_empty());

    let doc = &docs[0];
    assert!(doc.is_hash());

    let hash = doc.as_hash().unwrap();
    assert!(hash.contains_key(&yaml_rust2::yaml::Yaml::String("test_key".to_string())));
    assert_eq!(
        hash.get(&yaml_rust2::yaml::Yaml::String("test_key".to_string()))
            .unwrap()
            .as_str()
            .unwrap(),
        "test_value"
    );

    // Clean up
    fs::remove_file(path).unwrap();
}
