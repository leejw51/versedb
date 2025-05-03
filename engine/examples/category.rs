use anyhow::Result;
use versedb::database::Database;
use versedb::yaml::YamlDatabase;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
    // Create a new YAML database for variable-length category examples
    let mut db = YamlDatabase::open("variable_categories.yaml").await?;
    println!("Variable-length category database opened successfully!");

    // EXAMPLE 1: Different category depths - demonstrating variable length keys
    println!("\n=== VARIABLE LENGTH CATEGORY EXAMPLES ===");

    // One-level categories
    db.add(b"products:apple", b"Red fruit").await?;

    // Two-level categories
    db.add(b"products:electronics:laptop", b"Portable computer")
        .await?;

    // Three-level categories
    db.add(
        b"products:electronics:phones:smartphone",
        b"Mobile device with apps",
    )
    .await?;

    // Four-level categories
    db.add(
        b"products:electronics:phones:accessories:case",
        b"Phone protection",
    )
    .await?;

    // Query different depths
    println!("Products (all levels):");
    let all_products = db.select_range(b"products:", b"products:\xff").await?;
    for (key, value) in all_products {
        let key_str = String::from_utf8_lossy(&key);
        let depth = key_str.matches(':').count();
        println!(
            "{} (depth: {}) -> {}",
            key_str,
            depth,
            String::from_utf8_lossy(&value)
        );
    }

    // EXAMPLE 2: Keys that might seem ambiguous but aren't due to exact matching
    println!("\n=== COLLISION AVOIDANCE EXAMPLES ===");

    // Similar looking keys that don't collide
    db.add(b"user:1", b"User with ID 1").await?;
    db.add(b"user:10", b"User with ID 10").await?;
    db.add(b"user:100", b"User with ID 100").await?;

    // Keys with shared prefixes
    db.add(b"item:chair", b"Furniture to sit on").await?;
    db.add(b"item:chair:office", b"Chair for office use")
        .await?;
    db.add(b"item:chair:dining", b"Chair for dining table")
        .await?;

    // Edge case - empty value part
    db.add(b"tag:important:", b"Items marked important with no subtype")
        .await?;
    db.add(b"tag:important:high", b"High importance items")
        .await?;

    // Verify exact matches work correctly
    println!("Exact matches for similar keys:");
    println!(
        "user:1 -> {}",
        String::from_utf8_lossy(&db.select(b"user:1").await?.unwrap_or_default())
    );
    println!(
        "user:10 -> {}",
        String::from_utf8_lossy(&db.select(b"user:10").await?.unwrap_or_default())
    );
    println!(
        "user:100 -> {}",
        String::from_utf8_lossy(&db.select(b"user:100").await?.unwrap_or_default())
    );

    // EXAMPLE 3: Hierarchical data with queries at different levels
    println!("\n=== HIERARCHICAL DATA QUERIES ===");

    // Location data with continent > country > city > district
    db.add(
        b"location:europe:france:paris:montmartre",
        b"Artistic district in Paris",
    )
    .await?;
    db.add(
        b"location:europe:france:paris:louvre",
        b"Famous museum district",
    )
    .await?;
    db.add(
        b"location:europe:france:nice",
        b"City on the French Riviera",
    )
    .await?;
    db.add(b"location:europe:germany:berlin", b"Capital of Germany")
        .await?;
    db.add(b"location:asia:japan:tokyo", b"Capital of Japan")
        .await?;

    // Query by continent
    println!("European locations:");
    let europe_locs = db
        .select_range(b"location:europe:", b"location:europe:\xff")
        .await?;
    for (key, value) in europe_locs {
        println!(
            "  {}: {}",
            String::from_utf8_lossy(&key),
            String::from_utf8_lossy(&value)
        );
    }

    // Query by country
    println!("\nLocations in France:");
    let france_locs = db
        .select_range(b"location:europe:france:", b"location:europe:france:\xff")
        .await?;
    for (key, value) in france_locs {
        println!(
            "  {}: {}",
            String::from_utf8_lossy(&key),
            String::from_utf8_lossy(&value)
        );
    }

    // Query by city
    println!("\nLocations in Paris:");
    let paris_locs = db
        .select_range(
            b"location:europe:france:paris:",
            b"location:europe:france:paris:\xff",
        )
        .await?;
    for (key, value) in paris_locs {
        println!(
            "  {}: {}",
            String::from_utf8_lossy(&key),
            String::from_utf8_lossy(&value)
        );
    }

    // EXAMPLE 4: Mixed data types with the same prefix
    println!("\n=== MIXED DATA TYPES WITH SAME PREFIX ===");

    // Different types of "user" data
    db.add(b"user:profile:alex", b"Alex Smith, Designer")
        .await?;
    db.add(b"user:settings:alex", b"dark_mode=true").await?;
    db.add(b"user:activity:alex:login", b"Last login: yesterday")
        .await?;
    db.add(b"user:activity:alex:purchase", b"Last purchase: 3 days ago")
        .await?;

    // Get all data for a user across different categories
    println!("All data for Alex:");
    let alex_data = db.select_range(b"user:", b"user:\xff").await?;
    for (key, value) in alex_data {
        let key_str = String::from_utf8_lossy(&key);
        if key_str.contains(":alex") || key_str.contains(":alex:") {
            println!("  {}: {}", key_str, String::from_utf8_lossy(&value));
        }
    }

    // EXAMPLE 5: Time-based hierarchical data
    println!("\n=== TIME-BASED HIERARCHICAL DATA ===");

    // Logs with year:month:day:hour format
    db.add(b"logs:2023:01:01:00", b"New Year system check")
        .await?;
    db.add(b"logs:2023:01:01:12", b"Noon status update").await?;
    db.add(b"logs:2023:05:15:09", b"Morning error report")
        .await?;
    db.add(b"logs:2023:05:15:14", b"Afternoon warning").await?;
    db.add(b"logs:2023:05:15:18", b"Evening system restart")
        .await?;

    // Query a specific day's logs
    println!("Logs from May 15, 2023:");
    let day_logs = db
        .select_range(b"logs:2023:05:15:", b"logs:2023:05:15:\xff")
        .await?;
    for (key, value) in day_logs {
        let key_str = String::from_utf8_lossy(&key);
        let hour = key_str.split(':').nth(4).unwrap_or("unknown");
        println!("  {}:00 - {}", hour, String::from_utf8_lossy(&value));
    }

    // Flush changes to disk
    db.flush().await?;
    println!("\nVariable-length category database saved and closed successfully!");

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
