# versedb

![versedb logo](versedb.png)

versedb is an embedded database designed for both native and wasm32 targets. It provides a lightweight, efficient storage solution that can be embedded directly into your applications, whether they're running natively or in **web** browsers through WebAssembly.

## Database Support

### Native Targets
- SQLite: Full-featured SQL database support
- CSV: Simple and efficient CSV file storage
- JSON: Flexible document-based storage
- Sled: High-performance embedded database

### WebAssembly (wasm32)
- IndexedDB: Browser-based persistent storage

## Easy-to-Use Interface

versedb provides a simple and intuitive interface through the `Database` trait, supporting essential operations:
- Open and close database connections
- Add, select, and remove key-value pairs
- Range queries for efficient data retrieval

The interface is designed to be consistent across all storage backends, making it easy to switch between different storage solutions without changing your application code.

