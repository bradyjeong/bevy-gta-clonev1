use bevy::prelude::*;
use rand::Rng;
use crate::components::{Player, ActiveEntity, HumanMovement, HumanAnimation};
use crate::systems::timing_service::{TimingService, EntityTimerType, ManagedTiming};

// Legacy FootstepTimer component - now replaced by unified timing service
#[derive(Component)]
pub struct FootstepTimer {
    pub timer: f32,
    pub last_step_time: f32,
    pub step_interval: f32,
}

impl Default for FootstepTimer {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            timer: 0.0,
            last_step_time: 0.0,
            step_interval: rng.gen_range(0.45..0.55), // Natural variation in step timing
        }
    }
}

#[derive(Component)]
pub struct FootstepSound;

/// Unified footstep system using the timing service
pub fn footstep_system(
    mut commands: Commands,
    mut timing_service: ResMut<TimingService>,
    mut player_query: Query<
        (Entity, &Transform, &HumanAnimation, &HumanMovement, Option<&ManagedTiming>),
        (With<Player>, With<ActiveEntity>),
    >,
) {
    let Ok((entity, transform, animation, movement, managed_timing)) = player_query.single_mut() else {
        return;
    };

    // Register entity for timing management if not already managed
    if managed_timing.is_none() {
        let mut rng = rand::thread_rng();
        let step_interval = rng.gen_range(0.45..0.55); // Natural variation in step timing
        timing_service.register_entity(entity, EntityTimerType::FootstepAudio, step_interval);
        commands.entity(entity).insert(ManagedTiming::new(EntityTimerType::FootstepAudio));
        return; // Skip this frame to let the timer initialize
    }

    // Only play footsteps when walking/running
    if animation.is_walking && movement.current_speed > 0.5 {
        // Adjust timing based on movement speed
        let speed_multiplier = if animation.is_running { 0.6 } else { 1.0 };
        
        // Check if enough time has passed for next footstep (handled by timing service)
        if timing_service.should_update_entity(entity) {
            // Create footstep sound effect
            commands.spawn((
                Transform::from_translation(transform.translation),
                FootstepSound,
                ManagedTiming::new(EntityTimerType::FootstepAudio),
            ));
            
            // Add variation to next step interval for natural rhythm
            let mut rng = rand::thread_rng();
            let new_interval = rng.gen_range(0.45..0.55) * speed_multiplier;
            timing_service.register_entity(entity, EntityTimerType::FootstepAudio, new_interval);
        }
    }
}

// System to clean up footstep sound entities after a short time
pub fn cleanup_footstep_sounds(
    mut commands: Commands,
    mut timing_service: ResMut<TimingService>,
    footstep_query: Query<(Entity, &Transform, Option<&ManagedTiming>), With<FootstepSound>>,
) {
    // Use unified timing service for cleanup intervals
    if !timing_service.should_run_system(crate::systems::timing_service::SystemType::AudioCleanup) {
        return;
    }
    
    for (entity, _transform, managed_timing) in footstep_query.iter() {
        // Register for cleanup timing if not already managed
        if managed_timing.is_none() {
            timing_service.register_entity(entity, EntityTimerType::FootstepAudio, 1.0);
            commands.entity(entity).insert(ManagedTiming::new(EntityTimerType::FootstepAudio));
        }
        
        // Clean up entity if enough time has passed (1 second for footstep sounds)
        if timing_service.should_update_entity(entity) {
            timing_service.unregister_entity(entity);
            commands.entity(entity).despawn();
        }
    }
}
