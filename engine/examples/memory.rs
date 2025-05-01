#[cfg(not(target_arch = "wasm32"))]
use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use versedb::database::Database;
#[cfg(not(target_arch = "wasm32"))]
use versedb::memory::MemoryDatabase;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new memory database for books
    let mut db = MemoryDatabase::open("").await?;

    // Add 20 books
    let books = vec![
        ("The Great Gatsby", "F. Scott Fitzgerald"),
        ("1984", "George Orwell"),
        ("To Kill a Mockingbird", "Harper Lee"),
        ("Pride and Prejudice", "Jane Austen"),
        ("The Catcher in the Rye", "J.D. Salinger"),
        ("The Hobbit", "J.R.R. Tolkien"),
        ("Brave New World", "Aldous Huxley"),
        ("The Lord of the Rings", "J.R.R. Tolkien"),
        ("Crime and Punishment", "Fyodor Dostoevsky"),
        ("The Brothers Karamazov", "Fyodor Dostoevsky"),
        ("Moby Dick", "Herman Melville"),
        ("War and Peace", "Leo Tolstoy"),
        ("The Odyssey", "Homer"),
        ("The Iliad", "Homer"),
        ("Don Quixote", "Miguel de Cervantes"),
        ("The Divine Comedy", "Dante Alighieri"),
        ("Les Misérables", "Victor Hugo"),
        ("The Count of Monte Cristo", "Alexandre Dumas"),
        ("Anna Karenina", "Leo Tolstoy"),
        ("Wuthering Heights", "Emily Brontë"),
    ];

    for (title, author) in books {
        db.add(title.as_bytes(), author.as_bytes()).await?;
    }

    println!("\nInitial database contents:");
    let all_entries = db.select_range(&[0], &[255]).await?;
    for (title, author) in all_entries {
        println!(
            "{} by {}",
            String::from_utf8_lossy(&title),
            String::from_utf8_lossy(&author)
        );
    }
    println!("\nDatabase populated successfully.");

    // Retrieve a specific book
    if let Some(author) = db.select("The Great Gatsby".as_bytes()).await? {
        println!(
            "Author of 'The Great Gatsby': {}",
            String::from_utf8_lossy(&author)
        );
    }

    // Get books in a range (A-M)
    let entries = db.select_range("A".as_bytes(), "M".as_bytes()).await?;
    println!("\nBooks from A to M:");
    for (title, author) in entries {
        println!(
            "{} by {}",
            String::from_utf8_lossy(&title),
            String::from_utf8_lossy(&author)
        );
    }

    // Remove a book
    db.remove("The Great Gatsby".as_bytes()).await?;
    println!("\nAfter removing 'The Great Gatsby':");
    println!(
        "Book exists: {}",
        db.select("The Great Gatsby".as_bytes()).await?.is_some()
    );

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
