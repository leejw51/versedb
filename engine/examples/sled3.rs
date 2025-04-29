#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Local, Utc};
#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use versedb::database::Database;
#[cfg(not(target_arch = "wasm32"))]
use versedb::sled::SledDatabase;

#[cfg(not(target_arch = "wasm32"))]
fn get_dir_size(path: &str) -> u64 {
    let mut total_size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }
    }
    total_size
}

#[cfg(not(target_arch = "wasm32"))]
fn cleanup_db_files(path: &str) -> Result<(), Box<dyn Error>> {
    let db_path = Path::new(path);
    if db_path.exists() {
        fs::remove_dir_all(db_path)?;
        fs::create_dir_all(db_path)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new database in a temporary directory
    let db_name = "big_data_db";
    let mut db = SledDatabase::open(db_name).await?;

    // Create a large string (about 1MB per item)
    let large_string = "x".repeat(1 * 1024 * 1024);

    // Add 100 records with timestamps
    println!("Adding 100 large records (about 100MB total)...");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new())?;

    for i in 1..=100 {
        let local_time: DateTime<Local> = Local::now();
        let utc_time: DateTime<Utc> = Utc::now();

        let key = format!("big_record_{}", i);
        let value = format!(
            "Serial: {}, Local: {}, UTC: {}, Data: {}",
            i,
            local_time.to_rfc3339(),
            utc_time.to_rfc3339(),
            large_string
        );

        db.add(key.as_bytes(), value.as_bytes()).await?;
        println!("Added record {}", i);
    }

    println!(
        "\nDatabase size after adding records: {} bytes",
        get_dir_size(db_name)
    );

    // Select all records using select_range
    println!("\nSelecting all records using select_range...");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new())?;

    let start_key = "big_record_".as_bytes();
    let end_key = "big_record_z".as_bytes();
    let records = db.select_range(start_key, end_key).await?;

    println!("Total records retrieved: {}", records.len());
    println!("First record size: {} bytes", records[0].1.len());

    // Remove all records
    println!("\nRemoving all records...");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new())?;

    for i in 1..=100 {
        let key = format!("big_record_{}", i);
        db.remove(key.as_bytes()).await?;
        println!("Removed record {}", i);
    }

    println!(
        "\nDatabase size after removing records (before compaction): {} bytes",
        get_dir_size(db_name)
    );

    // Verify records were removed by trying to select them again
    println!("\nVerifying records were removed...");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new())?;

    let records = db.select_range(start_key, end_key).await?;
    println!("Records remaining after removal: {}", records.len());
    if records.is_empty() {
        println!("All records successfully removed!");
    } else {
        println!("Warning: Some records still remain in the database");
    }

    // Close the database
    println!("\nClosing database...");
    db.close().await?;

    // Manually clean up the database files
    println!("Cleaning up database files...");
    cleanup_db_files(db_name)?;

    println!(
        "Database size after cleanup: {} bytes",
        get_dir_size(db_name)
    );

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
