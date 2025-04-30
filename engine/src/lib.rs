#[cfg(not(target_arch = "wasm32"))]
pub mod client;
pub mod csv;
pub mod database;
#[cfg(target_arch = "wasm32")]
pub mod idb;
pub mod json;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
#[cfg(not(target_arch = "wasm32"))]
pub mod sled;
#[cfg(not(target_arch = "wasm32"))]
pub mod sqlite;
#[cfg(not(target_arch = "wasm32"))]
pub use client::VerseDbClient;
pub use database::Database;
#[cfg(not(target_arch = "wasm32"))]
pub use server::VerseDbServer;

pub mod versedb_capnp {
    include!("../generated/proto/versedb_capnp.rs");
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
