#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Local, Utc};
#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use versedb::RocksDbDatabase;
#[cfg(not(target_arch = "wasm32"))]
use versedb::database::Database;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new database in a temporary directory
    let mut db = RocksDbDatabase::open("rocksdb_time_records").await?;

    // Add 10 records with serial numbers and timestamps
    println!("Adding 10 records to RocksDB...");
    for i in 1..=10 {
        let local_time: DateTime<Local> = Local::now();
        let utc_time: DateTime<Utc> = Utc::now();

        let key = format!("record_{}", i);
        let value = format!(
            "Serial: {}, Local: {}, UTC: {}",
            i,
            local_time.to_rfc3339(),
            utc_time.to_rfc3339()
        );

        db.add(key.as_bytes(), value.as_bytes()).await?;
        println!("Added record {}: {}", i, value);
    }

    // Select all records
    println!("\nSelecting all records from RocksDB:");
    for i in 1..=10 {
        let key = format!("record_{}", i);
        if let Some(value) = db.select(key.as_bytes()).await? {
            println!("Key: {}, Value: {}", key, String::from_utf8_lossy(&value));
        }
    }

    // Demonstrate range operations
    println!("\nDemonstrating range operations:");
    let start_key = "record_3".as_bytes();
    let end_key = "record_7".as_bytes();

    println!("Selecting records in range [record_3, record_7):");
    let range_results = db.select_range(start_key, end_key).await?;
    for (key, value) in &range_results {
        println!(
            "Key: {}, Value: {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }

    println!("\nRemoving records in range [record_3, record_7):");
    let removed = db.remove_range(start_key, end_key).await?;
    println!("Removed {} records", removed.len());

    // Verify which records remain
    println!("\nVerifying remaining records:");
    for i in 1..=10 {
        let key = format!("record_{}", i);
        match db.select(key.as_bytes()).await? {
            Some(value) => println!("Key: {}, Value: {}", key, String::from_utf8_lossy(&value)),
            None => println!("Key: {} was removed", key),
        }
    }

    // Remove remaining records
    println!("\nRemoving all remaining records...");
    for i in vec![1, 2, 7, 8, 9, 10] {
        let key = format!("record_{}", i);
        db.remove(key.as_bytes()).await?;
    }

    // Verify no records remain
    println!("\nVerifying no records remain:");
    let mut count = 0;
    for i in 1..=10 {
        let key = format!("record_{}", i);
        if db.select(key.as_bytes()).await?.is_some() {
            count += 1;
        }
    }
    println!("Remaining records: {}", count);

    // Clean up the database
    db.close().await?;
    println!("Database closed successfully.");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
