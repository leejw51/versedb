use anyhow::Result;
use versedb::database::Database;
use versedb::yaml::YamlDatabase;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new YAML database
    let mut db = YamlDatabase::open("test.yaml").await?;
    println!("Database opened successfully!");

    // Add some key-value pairs
    db.add(b"name", b"Alice").await?;
    db.add(b"age", b"30").await?;
    db.add(b"city", b"New York").await?;
    println!("Added initial data");

    // Select and print a value
    if let Some(name) = db.select(b"name").await? {
        println!("Name: {}", String::from_utf8_lossy(&name));
    }

    // Select a range of values (from "a" to "n")
    let range_results = db.select_range(b"a", b"n").await?;
    println!("\nRange results (a-n):");
    for (key, value) in range_results {
        println!(
            "{}: {}",
            String::from_utf8_lossy(&key),
            String::from_utf8_lossy(&value)
        );
    }

    // Remove a key
    db.remove(b"age").await?;
    println!("\nRemoved 'age' entry");

    // Try to select the removed key
    if let Some(_) = db.select(b"age").await? {
        println!("Age still exists (unexpected)");
    } else {
        println!("Age was successfully removed");
    }

    // Add more data for range removal
    db.add(b"score1", b"100").await?;
    db.add(b"score2", b"95").await?;
    db.add(b"score3", b"88").await?;
    println!("\nAdded score data");

    // Remove a range of scores
    let removed = db.remove_range(b"score1", b"score3").await?;
    println!("\nRemoved scores:");
    for (key, value) in removed {
        println!(
            "Removed {}: {}",
            String::from_utf8_lossy(&key),
            String::from_utf8_lossy(&value)
        );
    }

    // Flush changes to disk
    db.flush().await?;
    println!("\nFlushed changes to disk");

    // Close the database
    db.close().await?;
    println!("Database closed successfully!");

    Ok(())
}
