//! Gameplay UI prelude - commonly used types and traits

// Re-export core dependencies
pub use game_core::prelude::*;
pub use gameplay_sim::prelude::*;
pub use gameplay_render::prelude::*;

// UI modules
pub use crate::ui::*;
pub use crate::debug::*;
pub use crate::performance::*;
pub use crate::plugins::*;

// Re-export timing service (selective to avoid conflicts)
pub use crate::timing_service::{TimingService, EntityTimer, EntityTimerType, SystemType};
pub use crate::config_loader::*;

// Main plugin
pub use crate::UiPlugin;
