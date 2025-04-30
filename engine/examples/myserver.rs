#[cfg(not(target_arch = "wasm32"))]
use versedb::server::run_server;


#[tokio::main]
#[cfg(not(target_arch = "wasm32"))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run the server on localhost:8000
    run_server("127.0.0.1:8000").await?;
    Ok(())
}


#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
