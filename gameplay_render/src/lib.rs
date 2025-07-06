//! Gameplay rendering - LOD, culling, effects
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;
pub use game_core;

pub mod prelude;
pub mod systems;

pub use prelude::*;

/// Main plugin for rendering systems
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        // Visual and audio presentation systems will be added as they're migrated
        // app.add_systems(Update, (
        //     // TODO: Add rendering systems as they're migrated
        // ));
    }
}
