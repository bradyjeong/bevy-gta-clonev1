use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{F16, SimpleF16Specs};

/// Observer that applies damping when F16 is spawned with specs
/// This runs automatically when an entity gets both F16 and SimpleF16Specs components
pub fn on_f16_spawned(
    trigger: Trigger<OnAdd, SimpleF16Specs>,
    mut commands: Commands,
    f16_query: Query<&SimpleF16Specs, With<F16>>,
) {
    let entity = trigger.target();
    
    if let Ok(specs) = f16_query.get(entity) {
        commands.entity(entity).insert(Damping {
            linear_damping: specs.linear_damping,
            angular_damping: specs.angular_damping,
        });
    }
}
