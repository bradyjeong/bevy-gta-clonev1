use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::config::GameConfig;
use crate::systems::physics::PhysicsUtilities;
use crate::components::MissingSpecsWarned;

/// Common system parameters for vehicle movement systems
/// Reduces boilerplate in function signatures
#[derive(SystemParam)]
pub struct VehicleParams<'w, 's, T: Asset + Send + Sync + 'static> {
    pub config: Res<'w, GameConfig>,
    pub time: Res<'w, Time>,
    pub commands: Commands<'w, 's>,
    pub warned: Query<'w, 's, (), With<MissingSpecsWarned>>,
    pub specs: Res<'w, Assets<T>>,
}

impl<'w, 's, T: Asset + Send + Sync + 'static> VehicleParams<'w, 's, T> {
    pub fn dt(&self) -> f32 {
        PhysicsUtilities::stable_dt(&self.time)
    }
}

/// Helper to validate specs and handle warning/tagging if missing
/// Returns Some(&T) if specs are valid, None (and handles warning) if missing
pub fn validate_specs<'a, T: Asset>(
    specs: &'a Assets<T>,
    commands: &mut Commands,
    warned: &Query<(), With<MissingSpecsWarned>>,
    entity: Entity,
    handle: &Handle<T>,
) -> Option<&'a T> {
    if let Some(spec) = specs.get(handle) {
        Some(spec)
    } else {
        if !warned.contains(entity) {
            warn!(
                "Entity {:?} missing loaded specs - will skip until loaded",
                entity
            );
            commands.entity(entity).insert(MissingSpecsWarned);
        }
        None
    }
}
