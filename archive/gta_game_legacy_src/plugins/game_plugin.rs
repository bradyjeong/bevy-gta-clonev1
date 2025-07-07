use bevy::prelude::*;
use crate::plugins::*;

/// Main game plugin that orchestrates all systems
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerPlugin,
            VehiclePlugin,
            UnifiedWorldPlugin,
            UIPlugin,
            WaterPlugin,
            PersistencePlugin,
            InputPlugin,
            VegetationLODPlugin,
            BatchingPlugin,
        ));
    }
}
