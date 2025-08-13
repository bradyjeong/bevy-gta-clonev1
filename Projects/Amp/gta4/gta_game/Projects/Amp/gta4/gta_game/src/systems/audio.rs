use bevy::prelude::*;
use rand::Rng;
use crate::components::{Player, ActiveEntity, HumanMovement, HumanAnimation};

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

pub fn footstep_system(
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<
        (&Transform, &HumanAnimation, &HumanMovement, &mut FootstepTimer),
        (With<Player>, With<ActiveEntity>),
    >,
) {
    let Ok((transform, animation, movement, mut footstep)) = player_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    footstep.timer += dt;

    // Only play footsteps when walking/running
    if animation.is_walking && movement.current_speed > 0.5 {
        let step_rate = if animation.is_running {
            footstep.step_interval * 0.6 // Faster steps when running
        } else {
            footstep.step_interval
        };

        if footstep.timer - footstep.last_step_time >= step_rate {
            // Add natural timing variation
            let mut rng = rand::thread_rng();
            let timing_variation = rng.gen_range(0.9..1.1);
            
            if footstep.timer - footstep.last_step_time >= step_rate * timing_variation {
                // Create footstep sound effect (placeholder - would be actual audio in real implementation)
                commands.spawn((
                    Transform::from_translation(transform.translation),
                    FootstepSound,
                    // In a real implementation, you'd spawn an AudioBundle here
                ));

                footstep.last_step_time = footstep.timer;
                
                // Vary the next step interval slightly for natural rhythm
                footstep.step_interval = rng.gen_range(0.45..0.55);
            }
        }
    }
}

// System to clean up footstep sound entities after a short time
pub fn cleanup_footstep_sounds(
    mut commands: Commands,
    _time: Res<Time>,
    footstep_query: Query<(Entity, &Transform), With<FootstepSound>>,
) {
    // In a real implementation, this would clean up audio entities after they finish playing
    for (entity, _transform) in footstep_query.iter() {
        // Placeholder cleanup - in real audio system, check if sound finished
        commands.entity(entity).despawn();
    }
}
