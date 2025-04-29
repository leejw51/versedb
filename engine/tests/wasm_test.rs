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
