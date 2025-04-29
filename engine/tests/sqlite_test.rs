#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use tempfile::NamedTempFile;
use versedb::database::Database;
#[cfg(not(target_arch = "wasm32"))]
use versedb::sqlite::SqliteDatabase;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_sqlite_database() {
    // Create a temporary file for testing
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Test open and add
    let mut db = SqliteDatabase::open(path).await.unwrap();
    db.add("key1".as_bytes(), "value1".as_bytes())
        .await
        .unwrap();
    db.add("key2".as_bytes(), "value2".as_bytes())
        .await
        .unwrap();
    db.close().await.unwrap();

    // Test reopen and select
    let db = SqliteDatabase::open(path).await.unwrap();
    assert_eq!(
        db.select("key1".as_bytes()).await.unwrap(),
        Some("value1".as_bytes().to_vec())
    );
    assert_eq!(
        db.select("key2".as_bytes()).await.unwrap(),
        Some("value2".as_bytes().to_vec())
    );
    assert_eq!(db.select("nonexistent".as_bytes()).await.unwrap(), None);

    // Test select_range
    let mut db = SqliteDatabase::open(path).await.unwrap();
    let range = db
        .select_range("key1".as_bytes(), "key3".as_bytes())
        .await
        .unwrap();
    assert_eq!(range.len(), 2);
    assert!(range.contains(&("key1".as_bytes().to_vec(), "value1".as_bytes().to_vec())));
    assert!(range.contains(&("key2".as_bytes().to_vec(), "value2".as_bytes().to_vec())));

    // Clean up
    fs::remove_file(path).unwrap();
}
