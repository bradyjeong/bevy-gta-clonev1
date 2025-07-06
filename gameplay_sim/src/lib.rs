//! Gameplay simulation - physics, AI, rules
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;
pub use game_core;

pub mod prelude;
pub mod systems;

pub use prelude::*;

/// Main plugin for simulation systems
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // Core simulation systems will be added as they're migrated
        // app.add_systems(Update, (
        //     // TODO: Add simulation systems as they're migrated
        // ));
    }
}
