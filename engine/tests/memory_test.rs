use versedb::database::Database;
use versedb::memory::MemoryDatabase;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_memory_database() {
    // Create a new memory database
    let mut db = MemoryDatabase::open("").await.unwrap();

    // Test add and select
    db.add("key1".as_bytes(), "value1".as_bytes())
        .await
        .unwrap();
    db.add("key2".as_bytes(), "value2".as_bytes())
        .await
        .unwrap();
    db.add("key3".as_bytes(), "value3".as_bytes())
        .await
        .unwrap();

    // Test select
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
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_memory_database_remove_range() {
    // Create a new memory database
    let mut db = MemoryDatabase::open("").await.unwrap();

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
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn test_memory_database_clone() {
    // Create a new memory database
    let mut db = MemoryDatabase::open("").await.unwrap();

    // Add some data
    db.add("key1".as_bytes(), "value1".as_bytes())
        .await
        .unwrap();
    db.add("key2".as_bytes(), "value2".as_bytes())
        .await
        .unwrap();

    // Clone the database
    let db_clone = db.clone();

    // Verify data in the clone
    assert_eq!(
        db_clone.select("key1".as_bytes()).await.unwrap(),
        Some("value1".as_bytes().to_vec())
    );
    assert_eq!(
        db_clone.select("key2".as_bytes()).await.unwrap(),
        Some("value2".as_bytes().to_vec())
    );

    // Modify original and verify clone is unaffected
    db.remove("key1".as_bytes()).await.unwrap();
    assert_eq!(
        db_clone.select("key1".as_bytes()).await.unwrap(),
        Some("value1".as_bytes().to_vec())
    );
}
