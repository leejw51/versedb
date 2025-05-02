use clap::Parser;
use versedb::csv::CsvDatabase;
use versedb::database::Database;
#[cfg(not(target_arch = "wasm32"))]
use versedb::server::run_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server address in the format host:port
    #[arg(short, long, default_value = "127.0.0.1:8000")]
    address: String,
}

#[tokio::main]
#[cfg(not(target_arch = "wasm32"))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create a CSV database
    let db = CsvDatabase::open("data.csv").await?;

    // Run the server with the configured address
    run_server(&args.address, db).await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
