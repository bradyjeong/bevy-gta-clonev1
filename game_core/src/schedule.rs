//! Global system scheduling and ordering
//! 
//! This module defines the global system sets that ensure proper ordering
//! across the entire game. All systems must belong to exactly one of these sets.

use bevy::prelude::*;

/// Global system sets that define execution order across the entire application
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GlobalSystemSet {
    /// Systems that run before physics (input processing, AI decisions)
    PrePhysics,
    /// Core physics simulation systems
    Physics,
    /// Systems that run after physics (visual effects, audio, rendering prep)
    PostPhysics,
    /// Rendering preparation and optimization systems
    RenderPrep,
}

impl GlobalSystemSet {
    /// Configure the global system set ordering for an app
    pub fn configure_sets(app: &mut App) {
        app.configure_sets(
            Update,
            (
                GlobalSystemSet::PrePhysics,
                GlobalSystemSet::Physics,
                GlobalSystemSet::PostPhysics,
                GlobalSystemSet::RenderPrep,
            ).chain(),
        );
    }
}

/// Macro to ensure a system is properly assigned to a global set
#[macro_export]
macro_rules! add_global_system {
    ($app:expr_2021, $system:expr_2021, $set:expr_2021) => {
        $app.add_systems(Update, $system.in_set($set))
    };
}

/// Re-export for convenience
pub use GlobalSystemSet::{PrePhysics, Physics, PostPhysics, RenderPrep};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_set_ordering() {
        let mut app = App::new();
        GlobalSystemSet::configure_sets(&mut app);
        
        // Test that the app builds successfully with the configured sets
        app.update();
    }
}
