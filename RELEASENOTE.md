# Release Notes

## v1.0.0 (2025-04-30)

### Features
- Added CapnP RPC support for client-server communication
  - High-performance binary serialization with Cap'n Proto protocol
  - Asynchronous client-server architecture with Tokio runtime
  - Comprehensive API including add, select, remove, and range queries
  - Built-in connection management and error handling
  - TCP transport with configurable networking options

- Added database backends for native platforms:
  - SQLite for persistent storage
    - ACID-compliant relational database storage
    - Automatic table creation and schema management
    - Binary-safe key-value storage with UTF-8 key support
    - Efficient range queries with ordered results
    - Thread-safe concurrent access with mutex protection
  
  - Sled for embedded key-value storage
    - High-performance embedded database
    - Optimized for fast read/write operations
    - Perfect for edge computing and embedded systems

- Added IndexedDB support for WebAssembly (wasm32) target
  - Browser-based persistent storage solution
  - Enables web applications to store structured data
  - Compatible with modern web browsers
  - Seamless integration with WebAssembly builds

### Summary
This release introduces a versatile database solution with multiple storage backends and deployment options. The CapnP RPC implementation enables efficient client-server communication, while the choice of storage backends (SQLite, Sled, IndexedDB) provides flexibility for different use cases - from traditional server deployments to embedded systems and web applications. The architecture supports both local and networked operations with a consistent API across all platforms.
