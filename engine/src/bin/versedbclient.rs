use chrono::{Local, Utc};
use clap::Parser;
use fake::faker::company::en::CompanyName;
use fake::faker::internet::en::FreeEmail;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::{FirstName, LastName, Name};
use fake::{Fake, Faker};
use std::io::{self, Write};
#[cfg(not(target_arch = "wasm32"))]
use versedb::client::connect;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server address in the format host:port
    #[arg(short, long, default_value = "127.0.0.1:8000")]
    address: String,
}

async fn print_menu() {
    println!("\nVerseDB Interactive Client");
    println!("-------------------------");
    println!("1. Add key-value pair");
    println!("2. Remove key");
    println!("3. Select value by key");
    println!("4. Select range");
    println!("5. Remove range");
    println!("6. Hello world");
    println!("7. Flush");
    println!("11. Add many key-value pairs");
    println!("0. Exit");
    print!("\nEnter your choice: ");
    io::stdout().flush().unwrap();
}

async fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[tokio::main]
#[cfg(not(target_arch = "wasm32"))]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let local = tokio::task::LocalSet::new();

    local
        .run_until(async move {
            let client = connect(&args.address).await?;

            loop {
                print_menu().await;
                let choice = get_input("").await;

                match choice.as_str() {
                    "1" => {
                        let key = get_input("Enter key: ").await;
                        let value = get_input("Enter value: ").await;
                        client.add(key.as_bytes(), value.as_bytes()).await?;
                        println!("Key-value pair added successfully!");
                    }
                    "2" => {
                        let key = get_input("Enter key to remove: ").await;
                        client.remove(key.as_bytes()).await?;
                        println!("Key removed successfully!");
                    }
                    "3" => {
                        let key = get_input("Enter key to select: ").await;
                        let result = client.select(key.as_bytes()).await?;
                        println!("Value: {}", String::from_utf8_lossy(&result));
                    }
                    "4" => {
                        let start_key = get_input("Enter start key: ").await;
                        let end_key = get_input("Enter end key: ").await;
                        let range_result = client
                            .select_range(start_key.as_bytes(), end_key.as_bytes())
                            .await?;
                        println!("\nRange results:");
                        for (k, v) in range_result {
                            println!(
                                "Key: {}, Value: {}",
                                String::from_utf8_lossy(&k),
                                String::from_utf8_lossy(&v)
                            );
                        }
                    }
                    "5" => {
                        let start_key = get_input("Enter start key: ").await;
                        let end_key = get_input("Enter end key: ").await;
                        let removed = client
                            .remove_range(start_key.as_bytes(), end_key.as_bytes())
                            .await?;
                        println!("\nRemoved range results:");
                        for (k, v) in removed {
                            println!(
                                "Key: {}, Value: {}",
                                String::from_utf8_lossy(&k),
                                String::from_utf8_lossy(&v)
                            );
                        }
                        println!("Range removed successfully!");
                    }
                    "6" => {
                        let name = get_input("Enter name: ").await;
                        let result = client.helloworld(&name).await?;
                        println!("{}", result);
                    }
                    "7" => {
                        client.flush().await?;
                        println!("Database flushed successfully!");
                    }
                    "11" => {
                        let category = get_input("Enter category: ").await;
                        let start_num = get_input("Enter start number: ")
                            .await
                            .parse::<i32>()
                            .unwrap_or(0);
                        let end_num = get_input("Enter end number: ")
                            .await
                            .parse::<i32>()
                            .unwrap_or(10);

                        for i in start_num..=end_num {
                            let name: String = Name().fake();
                            let email: String = FreeEmail().fake();
                            let company: String = CompanyName().fake();
                            let description: String = Sentence(5..10).fake();

                            let utc_time = Utc::now().to_rfc3339();
                            let local_time = Local::now().to_rfc3339();

                            let key = format!("{}:{:010}", category, i);
                            let value = format!(
                                "name:{}|email:{}|company:{}|desc:{}|UTC:{}|LOCAL:{}",
                                name, email, company, description, utc_time, local_time
                            );

                            client.add(key.as_bytes(), value.as_bytes()).await?;
                            println!("Added - Key: {}, Value: {}", key, value);
                        }
                        println!("Multiple key-value pairs with fake data added successfully!");
                    }
                    "0" => {
                        println!("Goodbye!");
                        break;
                    }
                    _ => println!("Invalid choice! Please try again."),
                }
            }

            Ok(())
        })
        .await
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
