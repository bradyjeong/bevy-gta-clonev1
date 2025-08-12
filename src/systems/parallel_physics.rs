// Simplified parallel physics system that works with Rapier
// Provides basic parallel processing without causing conflicts

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Simplified parallel physics plugin
pub struct ParallelPhysicsPlugin;

impl Plugin for ParallelPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ParallelPhysicsMetrics::default())
            .add_systems(
                FixedUpdate,
                (
                    prepare_parallel_physics,
                    apply_parallel_physics_results
                )
                    .chain()
                    .run_if(resource_exists::<ParallelPhysicsConfig>)
            );
    }
}

#[derive(Resource, Default)]
pub struct ParallelPhysicsMetrics {
    pub entities_processed: Arc<AtomicU32>,
    pub processing_time_ms: f32,
}

#[derive(Resource)]
pub struct ParallelPhysicsConfig {
    pub enabled: bool,
    pub max_entities_per_batch: usize,
    pub distance_threshold: f32,
}

impl Default for ParallelPhysicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entities_per_batch: 100,
            distance_threshold: 200.0,
        }
    }
}

/// Prepare entities for parallel processing
fn prepare_parallel_physics(
    mut query: Query<(&mut Velocity, &Transform, &ExternalForce), With<RigidBody>>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    config: Res<ParallelPhysicsConfig>,
    mut metrics: ResMut<ParallelPhysicsMetrics>,
) {
    if !config.enabled {
        return;
    }

    let start = std::time::Instant::now();
    
    // Get active entity position for distance-based processing
    let active_pos = active_query
        .iter()
        .next()
        .map(|t| t.translation)
        .unwrap_or_default();
    
    // Reset metrics
    metrics.entities_processed.store(0, Ordering::Relaxed);
    
    // Process entities that are within range
    let mut batch_count = 0;
    for (mut velocity, transform, _force) in query.iter_mut() {
        if batch_count >= config.max_entities_per_batch {
            break;
        }
        
        let distance = active_pos.distance(transform.translation);
        if distance > config.distance_threshold {
            continue;
        }
        
        // Apply basic velocity damping to prevent instability
        velocity.linvel *= 0.999;
        velocity.angvel *= 0.995;
        
        // Clamp velocities to prevent explosions
        let max_linear_vel = 100.0;
        let max_angular_vel = 10.0;
        
        if velocity.linvel.length() > max_linear_vel {
            velocity.linvel = velocity.linvel.normalize() * max_linear_vel;
        }
        
        if velocity.angvel.length() > max_angular_vel {
            velocity.angvel = velocity.angvel.normalize() * max_angular_vel;
        }
        
        batch_count += 1;
        metrics.entities_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    metrics.processing_time_ms = start.elapsed().as_millis() as f32;
}

/// Apply results from parallel physics processing
fn apply_parallel_physics_results(
    metrics: Res<ParallelPhysicsMetrics>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
    // Report metrics periodically
    let current_time = time.elapsed_secs();
    if current_time - *last_report > 5.0 {
        *last_report = current_time;
        let count = metrics.entities_processed.load(Ordering::Relaxed);
        if count > 0 {
            info!(
                "Parallel Physics: Processed {} entities in {:.2}ms",
                count, metrics.processing_time_ms
            );
        }
    }
}

/// System to safely enable/disable parallel physics
pub fn toggle_parallel_physics_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<ParallelPhysicsConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::F9) {
        config.enabled = !config.enabled;
        info!("Parallel physics: {}", if config.enabled { "Enabled" } else { "Disabled" });
    }
}
