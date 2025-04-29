#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use versedb::database::Database;
#[cfg(not(target_arch = "wasm32"))]
use versedb::json::JsonDatabase;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a temporary file for the database
    let db_path = "books_db.json";

    println!("Opening database at {}", db_path);
    let mut db = JsonDatabase::open(db_path).await?;

    // Add 10 initial books
    println!("\nAdding initial 10 books...");
    let initial_books = [
        ("book:001", "The Great Gatsby"),
        ("book:002", "1984"),
        ("book:003", "To Kill a Mockingbird"),
        ("book:004", "Pride and Prejudice"),
        ("book:005", "The Catcher in the Rye"),
        ("book:006", "The Hobbit"),
        ("book:007", "Brave New World"),
        ("book:008", "The Lord of the Rings"),
        ("book:009", "Crime and Punishment"),
        ("book:010", "Moby Dick"),
    ];

    for (key, title) in initial_books.iter() {
        db.add(key.as_bytes(), title.as_bytes()).await?;
    }

    // Display all books
    println!("\nCurrent books in database:");
    let all_books = db.select_range(b"book:001", b"book:010").await?;
    for (key, value) in all_books {
        println!("{}: {}", String::from_utf8(key)?, String::from_utf8(value)?);
    }

    // Remove 5 books
    println!("\nRemoving 5 books...");
    let books_to_remove = ["book:002", "book:004", "book:006", "book:008", "book:010"];
    for key in books_to_remove.iter() {
        db.remove(key.as_bytes()).await?;
    }

    // Display remaining books
    println!("\nRemaining books after removal:");
    let remaining_books = db.select_range(b"book:001", b"book:010").await?;
    for (key, value) in remaining_books {
        println!("{}: {}", String::from_utf8(key)?, String::from_utf8(value)?);
    }

    // Add 20 more books
    println!("\nAdding 20 more books...");
    let additional_books = [
        ("book:011", "The Odyssey"),
        ("book:012", "War and Peace"),
        ("book:013", "The Divine Comedy"),
        ("book:014", "Don Quixote"),
        ("book:015", "The Brothers Karamazov"),
        ("book:016", "Anna Karenina"),
        ("book:017", "Ulysses"),
        ("book:018", "The Iliad"),
        ("book:019", "Madame Bovary"),
        ("book:020", "The Count of Monte Cristo"),
        ("book:021", "Wuthering Heights"),
        ("book:022", "The Sound and the Fury"),
        ("book:023", "Lolita"),
        ("book:024", "The Grapes of Wrath"),
        ("book:025", "The Sun Also Rises"),
        ("book:026", "The Old Man and the Sea"),
        ("book:027", "One Hundred Years of Solitude"),
        ("book:028", "The Stranger"),
        ("book:029", "The Trial"),
        ("book:030", "The Metamorphosis"),
    ];

    for (key, title) in additional_books.iter() {
        db.add(key.as_bytes(), title.as_bytes()).await?;
    }

    // Display all books in the database
    println!("\nFinal list of all books:");
    let final_books = db.select_range(b"book:001", b"book:030").await?;
    for (key, value) in final_books {
        println!("{}: {}", String::from_utf8(key)?, String::from_utf8(value)?);
    }

    // Save changes
    println!("\nSaving database...");
    db.close().await?;

    println!("\nExample completed successfully!");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
