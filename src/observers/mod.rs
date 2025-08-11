//! Observer pattern implementations for entity-specific events
//! 
//! This module contains observers that replace entity-specific events
//! with more efficient entity lifecycle hooks.

pub mod content_observers_simple;
pub mod content_observers;

pub use content_observers_simple::ContentObserverPlugin as SimpleContentObserverPlugin;
pub use content_observers::ContentObserverPlugin;
