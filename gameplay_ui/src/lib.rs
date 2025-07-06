//! Gameplay UI - HUD, menus, debug overlays
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;
pub use game_core;

pub mod prelude;
pub mod systems;

pub use prelude::*;

/// Main plugin for UI systems
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // UI, HUD, and debug overlay systems will be added as they're migrated
        // app.add_systems(Update, (
        //     // TODO: Add UI systems as they're migrated
        // ));
    }
}
