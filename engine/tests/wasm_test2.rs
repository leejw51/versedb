use versedb::database::{Database, Result};
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use versedb::idb::IdbDatabaseWrapper;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

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

    // Test selecting a range of keys
    let start_key = b"key2" as &[u8];
    let end_key = b"key4" as &[u8];

    // Get all keys in range
    let results = db.select_range(start_key, end_key).await?;

    // Verify the results
    assert_eq!(results.len(), 3);
    assert_eq!(&results[0].0, b"key2");
    assert_eq!(&results[0].1, b"value2");
    assert_eq!(&results[1].0, b"key3");
    assert_eq!(&results[1].1, b"value3");
    assert_eq!(&results[2].0, b"key4");
    assert_eq!(&results[2].1, b"value4");

    // Clean up
    for (key, _) in &entries {
        db.remove(key).await?;
    }

    // Close database
    db.close().await?;
    Ok(())
}
