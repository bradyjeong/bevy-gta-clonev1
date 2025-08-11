//! World generation coordination events
//! 
//! Events for decoupled world generation across plugins:
//! - Chunk management (loading/unloading)
//! - Dynamic content spawning coordination
//! - Spawn validation (replaces direct is_on_road_spline calls)
//! 
//! All events follow AGENT.md guidelines:
//! - Lightweight (≤128 bytes)
//! - Copy/Clone for performance
//! - Cleared every frame
//! - Multiple readers supported

pub mod chunk_events;
pub mod content_events;
pub mod validation_events;

pub use chunk_events::*;
pub use content_events::*;
pub use validation_events::*;
