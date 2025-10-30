use bevy::prelude::*;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, _app: &mut App) {
        // REMOVED: Duplicate HanabiPlugin - already in GameCorePlugin
        // All particle systems are registered in VehiclePlugin
    }
}
