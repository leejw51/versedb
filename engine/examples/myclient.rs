#[cfg(not(target_arch = "wasm32"))]
use versedb::client::connect;

#[tokio::main]
#[cfg(not(target_arch = "wasm32"))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a LocalSet
    let local = tokio::task::LocalSet::new();

    // Run our async code inside the LocalSet
    local
        .run_until(async move {
            // Connect to the server
            let client = connect("127.0.0.1:8000").await?;

            // Test hello world
            let result = client.helloworld("World").await?;
            println!("Hello world test: {}", result);

            // Test basic operations
            let key = b"test_key";
            let value = b"test_value";

            // Add a key-value pair
            client.add(key, value).await?;
            client.flush().await?;
            println!("Added key-value pair");

            // Select the value
            let retrieved = client.select(key).await?;
            println!("Retrieved value: {:?}", String::from_utf8_lossy(&retrieved));

            // Test select_range functionality
            // Add multiple key-value pairs for range test
            client.add(b"key1", b"value1").await?;
            client.add(b"key2", b"value2").await?;
            client.add(b"key3", b"value3").await?;
            client.flush().await?;
            println!("\nAdded multiple key-value pairs for range test");

            // Test range selection
            let range_result = client.select_range(b"key1", b"key3").await?;
            println!("Select range results:");
            for (k, v) in range_result {
                println!(
                    "Key: {:?}, Value: {:?}",
                    String::from_utf8_lossy(&k),
                    String::from_utf8_lossy(&v)
                );
            }

            // Test remove_range functionality
            println!("\nTesting remove_range...");
            let removed = client.remove_range(b"key1", b"key3").await?;
            println!("Removed range results:");
            for (k, v) in removed {
                println!(
                    "Key: {:?}, Value: {:?}",
                    String::from_utf8_lossy(&k),
                    String::from_utf8_lossy(&v)
                );
            }

            // Verify removal by trying to select the range again
            let empty_range = client.select_range(b"key1", b"key3").await?;
            println!(
                "\nVerifying removal - range should be empty: {} items",
                empty_range.len()
            );

            // Remove the remaining test key
            client.remove(key).await?;
            println!("\nCleaned up remaining test key");

            Ok(())
        })
        .await
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
