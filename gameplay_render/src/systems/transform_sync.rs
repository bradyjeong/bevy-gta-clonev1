use bevy::prelude::*;
use bevy_rapier3d::prelude::RigidBody;

/// Single source of truth for entity positions - prevents fighting between systems
#[derive(Component, Default, Reflect)]
pub struct TransformSync {
    pub target_translation: Vec3,
    pub target_rotation: Quat,
    pub smoothing_speed: f32,
}
impl TransformSync {
    #[allow(dead_code)]
    pub fn new(smoothing_speed: f32) -> Self {
        Self {
            target_translation: Vec3::ZERO,
            target_rotation: Quat::IDENTITY,
            smoothing_speed,
        }
    }
}

/// System that smoothly syncs transforms for NON-PHYSICS entities only
/// WARNING: This system should NOT be used on RigidBody entities!
pub fn sync_transforms_system(
    mut query: Query<(&mut Transform, &mut TransformSync), Without<RigidBody>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs().min(0.016); // Cap at 60fps for stability
    
    for (mut transform, sync) in query.iter_mut() {
        // Smooth position
        transform.translation = transform.translation.lerp(
            sync.target_translation, 
            sync.smoothing_speed * dt
        );
        
        // Smooth rotation
        transform.rotation = transform.rotation.slerp(
            sync.target_rotation,
            sync.smoothing_speed * dt
        );
        
        // Once close enough, snap to target to prevent infinite micro-movements
        if transform.translation.distance(sync.target_translation) < 0.01 {
            transform.translation = sync.target_translation;
        }
        if transform.rotation.angle_between(sync.target_rotation) < 0.01 {
            transform.rotation = sync.target_rotation;
        }
    }
}

pub struct TransformSyncPlugin;
impl Plugin for TransformSyncPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<TransformSync>()
            .add_systems(PostUpdate, sync_transforms_system);
    }
}
