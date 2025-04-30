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

    // Add 200 records with serial numbers and timestamps
    println!("Adding 200 records...");
    for i in 1u32..=200 {
        let local_time: DateTime<Local> = Local::now();
        let utc_time: DateTime<Utc> = Utc::now();

        let key = format!("mycategory_{}", hex::encode(i.to_be_bytes()));
        let value = format!(
            "Serial: {}, Local: {}, UTC: {}",
            i,
            local_time.to_rfc3339(),
            utc_time.to_rfc3339()
        );

        db.add(key.as_bytes(), value.as_bytes()).await?;
        println!("Added key: {}", key);
    }

    // Select range of records using prefix
    println!("\nSelecting range of records from 150 to 155:");

    let start = 150i32;
    let end = 155i32;
    let start_key = format!("mycategory_{}", hex::encode(start.to_be_bytes()));
    let end_key = format!("mycategory_{}", hex::encode(end.to_be_bytes()));
    let range_result = db
        .select_range(start_key.as_bytes(), end_key.as_bytes())
        .await?;
    for (key, value) in range_result {
        println!(
            "Key: {}, Value: {}",
            String::from_utf8_lossy(&key),
            String::from_utf8_lossy(&value)
        );
    }

    // Remove all records
    println!("\nRemoving all records...");
    for i in 1i32..=200i32 {
        let key = format!("mycategory_{}", hex::encode(i.to_be_bytes()));
        db.remove(key.as_bytes()).await?;
    }

    // Verify no records remain
    println!("\nVerifying no records remain:");
    let prefix = String::from("mycategory_00000000");
    let prefix2 = String::from("mycategory_ffffffff");
    let start = prefix.as_bytes();
    let end = prefix2.as_bytes();

    let remaining = db.select_range(start, end).await?;
    println!("Remaining records: {}", remaining.len());

    // Clean up the database
    db.close().await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
