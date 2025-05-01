use versedb::database::{Database, Result};
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use versedb::idb::IdbDatabaseWrapper;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen_test]
async fn test_idb_basic_operations() -> Result<()> {
    // Open database
    let mut db = IdbDatabaseWrapper::open("test_db").await?;

    // Test add
    let key: &[u8] = b"test_key";
    let value: &[u8] = b"test_value";
    db.add(key, value).await?;

    // Test select
    let result = db.select(key).await?;
    assert_eq!(result, Some(value.to_vec()));

    // Test remove
    db.remove(key).await?;
    let result = db.select(key).await?;
    assert_eq!(result, None);

    // Close database
    db.close().await?;
    Ok(())
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen_test]
async fn test_idb_multiple_entries() -> Result<()> {
    let mut db = IdbDatabaseWrapper::open("test_db_multiple").await?;

    // Add multiple entries
    let entries = vec![
        (b"key1" as &[u8], b"value1" as &[u8]),
        (b"key2" as &[u8], b"value2" as &[u8]),
        (b"key3" as &[u8], b"value3" as &[u8]),
    ];

    for (key, value) in &entries {
        db.add(key, value).await?;
    }

    // Verify all entries
    for (key, value) in &entries {
        let result = db.select(key).await?;
        assert_eq!(result, Some(value.to_vec()));
    }

    // Remove all entries
    for (key, _) in &entries {
        db.remove(key).await?;
    }

    // Verify all entries are removed
    for (key, _) in &entries {
        let result = db.select(key).await?;
        assert_eq!(result, None);
    }

    db.close().await?;
    Ok(())
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen_test]
async fn test_idb_remove_range() -> Result<()> {
    let mut db = IdbDatabaseWrapper::open("test_db_remove_range").await?;

    // Add entries with ordered keys
    let entries = vec![
        (b"key1" as &[u8], b"value1" as &[u8]),
        (b"key2" as &[u8], b"value2" as &[u8]),
        (b"key3" as &[u8], b"value3" as &[u8]),
        (b"key4" as &[u8], b"value4" as &[u8]),
        (b"key5" as &[u8], b"value5" as &[u8]),
    ];

    // Add all entries
    for (key, value) in &entries {
        db.add(key, value).await?;
    }

    // Test remove_range from key2 to key4 (inclusive of key2, exclusive of key4)
    let removed = db.remove_range(b"key2", b"key4").await?;

    // Verify the removed entries
    assert_eq!(removed.len(), 2);
    assert!(removed.contains(&(b"key2".to_vec(), b"value2".to_vec())));
    assert!(removed.contains(&(b"key3".to_vec(), b"value3".to_vec())));

    // Verify the remaining entries
    let remaining = db.select_range(b"key1", b"key6").await?;
    assert_eq!(remaining.len(), 3);
    assert!(remaining.contains(&(b"key1".to_vec(), b"value1".to_vec())));
    assert!(remaining.contains(&(b"key4".to_vec(), b"value4".to_vec())));
    assert!(remaining.contains(&(b"key5".to_vec(), b"value5".to_vec())));

    // Verify specific entries are gone
    assert_eq!(db.select(b"key2").await?, None);
    assert_eq!(db.select(b"key3").await?, None);

    // Close database
    db.close().await?;
    Ok(())
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen_test]
async fn test_idb_select_range() -> Result<()> {
    // Open database
    let mut db = IdbDatabaseWrapper::open("test_db_range").await?;

    // Add multiple entries with sequential keys
    let entries = vec![
        (b"key1" as &[u8], b"value1" as &[u8]),
        (b"key2" as &[u8], b"value2" as &[u8]),
        (b"key3" as &[u8], b"value3" as &[u8]),
        (b"key4" as &[u8], b"value4" as &[u8]),
        (b"key5" as &[u8], b"value5" as &[u8]),
    ];

    // Add all entries
    for (key, value) in &entries {
        db.add(key, value).await?;
    }

    // Test selecting a range of keys (inclusive of key2, exclusive of key4)
    let start_key = b"key2" as &[u8];
    let end_key = b"key4" as &[u8];

    // Get all keys in range
    let results = db.select_range(start_key, end_key).await?;

    // Verify the results
    assert_eq!(results.len(), 2);
    assert_eq!(&results[0].0, b"key2");
    assert_eq!(&results[0].1, b"value2");
    assert_eq!(&results[1].0, b"key3");
    assert_eq!(&results[1].1, b"value3");

    // Clean up
    for (key, _) in &entries {
        db.remove(key).await?;
    }

    // Close database
    db.close().await?;
    Ok(())
}
