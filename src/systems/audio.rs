use bevy::prelude::*;
use crate::components::{Player, ActiveEntity, HumanMovement, HumanAnimation};
use crate::GlobalRng;



#[derive(Component)]
pub struct FootstepTimer {
    pub timer: Timer,
}

impl FootstepTimer {
    pub fn new(interval: f32) -> Self {
        Self {
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct FootstepSound {
    pub cleanup_timer: Timer,
}

impl Default for FootstepSound {
    fn default() -> Self {
        Self {
            cleanup_timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

/// Simple footstep system
pub fn footstep_system(
    mut commands: Commands,
    time: Res<Time>,
    mut global_rng: ResMut<GlobalRng>,
    mut player_query: Query<
        (Entity, &Transform, &HumanAnimation, &HumanMovement, Option<&mut FootstepTimer>),
        (With<Player>, With<ActiveEntity>),
    >,
) {
    let Ok((entity, transform, animation, movement, mut timer)) = player_query.single_mut() else {
        return;
    };

    // Add timer component if it doesn't exist
    if timer.is_none() {
        let interval = global_rng.gen_range(0.45..0.55);
        commands.entity(entity).insert(FootstepTimer::new(interval));
        return;
    }
    
    let timer = timer.as_mut().unwrap();

    // Only process footsteps if walking
    if animation.is_walking && movement.current_speed > 0.5 {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            // Spawn footstep sound
            commands.spawn((
                Transform::from_translation(transform.translation),
                FootstepSound::default(),
            ));
            
            // Add variation to next step interval
            let new_interval = global_rng.gen_range(0.45..0.55);
            let speed_multiplier = if animation.is_running { 0.6 } else { 1.0 };
            timer.timer.set_duration(std::time::Duration::from_secs_f32(new_interval * speed_multiplier));
        }
    }
}

// System to clean up footstep sound entities after a short time
pub fn cleanup_footstep_sounds(
    mut commands: Commands,
    time: Res<Time>,
    mut footstep_query: Query<(Entity, &mut FootstepSound)>,
) {
    for (entity, mut footstep) in footstep_query.iter_mut() {
        footstep.cleanup_timer.tick(time.delta());
        if footstep.cleanup_timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
