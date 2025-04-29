#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Local, Utc};
#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use versedb::database::Database;
#[cfg(not(target_arch = "wasm32"))]
use versedb::sled::SledDatabase;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new database in a temporary directory
    let mut db = SledDatabase::open("time_records_db").await?;

    // Add 10 records with serial numbers and timestamps
    println!("Adding 10 records...");
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
    println!("\nSelecting all records:");
    for i in 1..=10 {
        let key = format!("record_{}", i);
        if let Some(value) = db.select(key.as_bytes()).await? {
            println!("Key: {}, Value: {}", key, String::from_utf8_lossy(&value));
        }
    }

    // Remove all records
    println!("\nRemoving all records...");
    for i in 1..=10 {
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
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
