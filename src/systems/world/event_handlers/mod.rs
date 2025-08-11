//! Event handler systems for world management
//! 
//! These systems handle world events and replace direct coupling between plugins.
//! Each handler follows the naming convention: handle_[event_name]_[action]

pub mod spawn_validation_handler;
pub mod content_spawn_handler;
pub mod chunk_handler;
pub mod validation_to_spawn_bridge;

pub use spawn_validation_handler::*;
pub use content_spawn_handler::*;
pub use chunk_handler::*;
pub use validation_to_spawn_bridge::*;

// Export V2 handlers (now default)
pub mod spawn_validation_handler_v2;
