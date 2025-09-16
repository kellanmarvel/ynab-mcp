// pub mod benchmarks; // Removed - not needed for MCP operations (development/testing only)
pub mod config;
pub mod docs;
pub mod error;
pub mod handlers;
pub mod health;
// Plugins system removed - functionality embedded directly in system handlers
// pub mod progress; // Removed - not needed for MCP operations
pub mod schemas;
pub mod secrets;
pub mod server;
#[cfg(test)]
pub mod test_utils;
// pub mod streaming; // Removed - replaced with simple_pagination (also unused)
// pub mod simple_pagination; // Removed - not needed for current MCP operations
pub mod utils;
pub mod ynab;

// Re-export key types for easy access
pub use server::YnabMcpServer;
