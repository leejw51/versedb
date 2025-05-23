pub use anyhow::Result;
use async_trait::async_trait;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Database: Send + Sync + Clone {
    /// Open a new database connection
    async fn open(path: &str) -> Result<Self>
    where
        Self: Sized;

    /// Close the database connection
    async fn close(&mut self) -> Result<()>;

    /// Add a key-value pair to the database
    async fn add(&mut self, key: &[u8], value: &[u8]) -> Result<()>;

    /// Select a value from the database by key
    async fn select(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Remove a key-value pair from the database
    async fn remove(&mut self, key: &[u8]) -> Result<()>;

    /// Select key-value pairs within a range [start, end)
    /// Returns a vector of tuples containing (key, value) pairs
    async fn select_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// Remove key-value pairs within a range [start, end)
    async fn remove_range(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// Flush any pending writes to disk to ensure data durability
    async fn flush(&mut self) -> Result<()>;
}
