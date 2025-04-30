use chrono::Utc;
use std::error::Error;
use versedb::csv::CsvDatabase;
use versedb::database::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new CSV database
    let mut db = CsvDatabase::open("books.csv").await?;

    // List of random book names
    let book_names = vec![
        "The Great Gatsby",
        "To Kill a Mockingbird",
        "1984",
        "Pride and Prejudice",
        "The Catcher in the Rye",
        "The Hobbit",
        "Brave New World",
        "The Lord of the Rings",
        "Animal Farm",
        "The Alchemist",
    ];

    // Insert items with UTC timestamps as keys
    for book_name in book_names.iter() {
        let timestamp = Utc::now().timestamp_nanos().to_string();
        let key = timestamp.as_bytes().to_vec();
        db.add(&key, book_name.as_bytes()).await?;
    }

    // Display all items
    println!("Book Store Contents:");
    println!("-------------------");

    // Read all items using select_range from 0 to maximum
    let start_key = "0".as_bytes();
    let end_key = "9999999999999999999".as_bytes(); // Large number to include all timestamps
    let all_items = db.select_range(start_key, end_key).await?;

    // Print items with index
    for (index, (key, value)) in all_items.iter().enumerate() {
        let timestamp = String::from_utf8(key.clone())?;
        let book_name = String::from_utf8(value.clone())?;
        println!("{}. Timestamp: {}", index + 1, timestamp);
        println!("   Book: {}", book_name);
        println!("   -------------------");
    }

    // Close the database
    db.close().await?;

    Ok(())
}
