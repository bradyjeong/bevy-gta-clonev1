//! # AMP Core
//!
//! This crate provides core error handling and utilities for the AMP Game Engine.
//! It defines the primary error types and result aliases used throughout the engine.

/// A specialized `Result` type for operations that may fail within the AMP engine.
///
/// This type is used as the return type for functions that may encounter errors
/// during execution. It provides a convenient way to handle both successful
/// results and error conditions.
pub type Result<T> = std::result::Result<T, Error>;

/// The main error type for the AMP Game Engine.
///
/// This enum represents all possible errors that can occur within the engine.
/// It uses `thiserror` for automatic error trait implementations and provides
/// structured error handling across all engine components.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O related errors (file operations, network, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        /// Error message
        message: String,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message
        message: String,
    },

    /// Resource loading errors
    #[error("Failed to load resource '{resource}': {reason}")]
    ResourceLoad {
        /// Resource name
        resource: String,
        /// Failure reason
        reason: String,
    },

    /// Validation errors
    #[error("Validation failed: {message}")]
    Validation {
        /// Error message
        message: String,
    },

    /// Invalid state errors
    #[error("Invalid state: {message}")]
    InvalidState {
        /// Error message
        message: String,
    },

    /// GPU-related errors
    #[error("GPU error: {message}")]
    Gpu {
        /// Error message
        message: String,
    },

    /// Generic error for cases not covered by specific variants
    #[error("Internal error: {message}")]
    Internal {
        /// Error message
        message: String,
    },
}

impl Error {
    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a new resource loading error
    pub fn resource_load<R: Into<String>, S: Into<String>>(resource: R, reason: S) -> Self {
        Self::ResourceLoad {
            resource: resource.into(),
            reason: reason.into(),
        }
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a new invalid state error
    pub fn invalid_state<S: Into<String>>(message: S) -> Self {
        Self::InvalidState {
            message: message.into(),
        }
    }

    /// Create a new GPU error
    pub fn gpu<S: Into<String>>(message: S) -> Self {
        Self::Gpu {
            message: message.into(),
        }
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::configuration("Invalid config value");
        assert!(matches!(err, Error::Configuration { .. }));
        assert_eq!(err.to_string(), "Configuration error: Invalid config value");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let err = Error::from(io_err);
        assert!(matches!(err, Error::Io(_)));
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_serialization_error() {
        let err = Error::serialization("Failed to parse JSON");
        assert!(matches!(err, Error::Serialization { .. }));
        assert_eq!(err.to_string(), "Serialization error: Failed to parse JSON");
    }

    #[test]
    fn test_resource_load_error() {
        let err = Error::resource_load("texture.png", "File corrupted");
        assert!(matches!(err, Error::ResourceLoad { .. }));
        assert_eq!(
            err.to_string(),
            "Failed to load resource 'texture.png': File corrupted"
        );
    }

    #[test]
    fn test_validation_error() {
        let err = Error::validation("Input value out of range");
        assert!(matches!(err, Error::Validation { .. }));
        assert_eq!(
            err.to_string(),
            "Validation failed: Input value out of range"
        );
    }

    #[test]
    fn test_invalid_state_error() {
        let err = Error::invalid_state("Component not initialized");
        assert!(matches!(err, Error::InvalidState { .. }));
        assert_eq!(err.to_string(), "Invalid state: Component not initialized");
    }

    #[test]
    fn test_gpu_error() {
        let err = Error::gpu("GPU device lost");
        assert!(matches!(err, Error::Gpu { .. }));
        assert_eq!(err.to_string(), "GPU error: GPU device lost");
    }

    #[test]
    fn test_internal_error() {
        let err = Error::internal("Unexpected condition");
        assert!(matches!(err, Error::Internal { .. }));
        assert_eq!(err.to_string(), "Internal error: Unexpected condition");
    }

    #[test]
    fn test_result_alias() {
        fn test_function() -> Result<i32> {
            Ok(42)
        }

        fn test_function_error() -> Result<i32> {
            Err(Error::internal("Test error"))
        }

        assert_eq!(test_function().unwrap(), 42);
        assert!(test_function_error().is_err());
    }

    #[test]
    fn test_error_debug() {
        let err = Error::configuration("Test config error");
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Configuration"));
        assert!(debug_str.contains("Test config error"));
    }

    #[test]
    fn test_error_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let err = Error::from(io_err);

        let error_chain = format!("{err}");
        assert!(error_chain.contains("I/O error"));
        assert!(error_chain.contains("Access denied"));
    }
}
