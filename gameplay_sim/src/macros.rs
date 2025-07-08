//! Macro re-exports for the gameplay simulation crate

// Re-export logging macros from tracing (primary)
pub use tracing::{info, warn, error, debug};

// If you need Bevy versions, use aliases:
pub use bevy::log::{info as bevy_info, warn as bevy_warn, error as bevy_error, debug as bevy_debug};
