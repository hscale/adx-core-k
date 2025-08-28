// ADX Core Shared Library
// Common utilities, types, and abstractions used across all services

pub mod database;
pub mod temporal;
pub mod auth;
pub mod tenant;
pub mod error;
pub mod config;

// Re-export commonly used types
pub use error::{Result, ServiceError};
pub use config::Config;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_library_basic() {
        // Basic test to ensure the shared library compiles and works
        assert_eq!(2 + 2, 4);
    }
}