#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Local, Utc};
#[cfg(not(target_arch = "wasm32"))]
use versedb::sled::SledDatabase;
#[cfg(not(target_arch = "wasm32"))]
use versedb::database::Database;
#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;

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

    // Select all records using select_range
    println!("\nSelecting all records using select_range:");
    let start_key = "record_".as_bytes();
    let end_key = "record_z".as_bytes(); // Using 'z' to include all records
    let records = db.select_range(start_key, end_key).await?;
    
    for (key, value) in records {
        println!(
            "Key: {}, Value: {}",
            String::from_utf8_lossy(&key),
            String::from_utf8_lossy(&value)
        );
    }

    // Remove all records
    println!("\nRemoving all records...");
    for i in 1..=10 {
        let key = format!("record_{}", i);
        db.remove(key.as_bytes()).await?;
    }

    // Verify no records remain using select_range
    println!("\nVerifying no records remain using select_range:");
    let records = db.select_range(start_key, end_key).await?;
    println!("Remaining records: {}", records.len());

    // Clean up the database
    db.close().await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
