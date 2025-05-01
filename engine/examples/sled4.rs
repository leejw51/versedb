use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use versedb::database::Database;
#[cfg(not(target_arch = "wasm32"))]
use versedb::sled::SledDatabase;

const PREFIX_CATEGORY: &str = "category";

#[derive(Serialize, Deserialize)]
struct CategoryItem {
    id: u32,
    timestamp: DateTime<Utc>,
    data: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new database
    let mut db = SledDatabase::open("category_db").await?;

    // Add 100 items
    println!("Adding 100 items to mycategory...");
    for i in 1..=100 {
        let item = CategoryItem {
            id: i,
            timestamp: Utc::now(),
            data: format!("Item data {}", i),
        };

        let key = format!("{}_mycategory_{:010}", PREFIX_CATEGORY, i);
        let value = serde_json::to_vec(&item)?;

        db.add(key.as_bytes(), &value).await?;
        println!("Added item {}", i);
    }

    // Select all items using range
    println!("\nSelecting all items using range:");
    let prefix = format!("{}_mycategory_", PREFIX_CATEGORY);
    let mut end_key = prefix.clone().into_bytes();
    end_key.push(0xff);

    let pairs = db.select_range(prefix.as_bytes(), &end_key).await?;
    println!("Found {} items", pairs.len());

    // Display first 5 items as sample
    for (i, (key, value)) in pairs.iter().enumerate() {
        let item: CategoryItem = serde_json::from_slice(&value)?;
        println!(
            "Item {}: Key={}, ID={}, Timestamp={}, Data={}",
            i + 1,
            String::from_utf8_lossy(key),
            item.id,
            item.timestamp,
            item.data
        );
    }
    use std::io::Read;
    // Clean up
    db.close().await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
