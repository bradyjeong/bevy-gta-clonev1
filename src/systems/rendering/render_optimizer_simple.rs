use bevy::prelude::*;
use std::time::Instant;

use crate::components::{ActiveEntity, Cullable};
use crate::config::GameConfig;

/// Custom render optimization system that replaces Bevy's built-in culling with time-budgeted processing
///
/// PERFORMANCE TARGETS (measured as wall-clock time via Instant::elapsed):
/// - Time budget: 3ms max processing time per frame (render_optimization_system)
/// - Time budget: 2ms max processing time per frame (batch_rendering_system)  
/// - Time budget: 1ms max processing time per frame (render_queue_manager_system)
/// - Update frequency: 0.5s for primary system, 0.3s for batch system
/// - Operation limits: 20 operations per frame (primary), 20 batches per frame (batch)
/// - Distance cutoff: 300m for visibility, uses config.world.streaming_radius for batch system
///
/// MIGRATION PATH: This system may become obsolete once Bevy implements region visibility & world streaming.
/// Track these RFCs for upstream alternatives:
/// - Region visibility RFC: https://github.com/bevyengine/rfcs/pull/89
/// - World streaming RFC: https://github.com/bevyengine/rfcs/pull/90
/// When available, consider migrating to upstream implementation to reduce maintenance burden.
///
/// Simplified render optimization system with batching limits and view frustum culling
#[cfg(feature = "simple_render_culler")]
pub fn render_optimization_system(
    read_query: Query<(Entity, &Transform), With<Cullable>>,
    mut visibility_query: Query<&mut Visibility, With<Cullable>>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    config: Res<GameConfig>,
    _time: Res<Time>,
) {
    let start_time = Instant::now();

    let Ok(active_transform) = active_query.single() else {
        return;
    };
    let active_pos = active_transform.translation;

    let max_processing_time = 3.0; // 3ms time budget
    let max_render_operations = 20; // Limit render operations per frame
    let mut render_operations = 0;

    // Pass 1: collect (Entity, distance) for sorting (read-only for thread safety)
    let mut entities: Vec<(Entity, f32)> = read_query
        .iter()
        .map(|(entity, transform)| {
            let distance = active_pos.distance(transform.translation);
            (entity, distance)
        })
        .collect();

    // Use partial sort for better performance with large entity counts
    if entities.len() > max_render_operations {
        entities.select_nth_unstable_by(max_render_operations, |a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        entities.truncate(max_render_operations);
    } else {
        entities.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    // Pass 2: fresh borrow for each entity
    for (entity, distance) in entities {
        let Ok(mut visibility) = visibility_query.get_mut(entity) else {
            continue;
        };
        let Ok((_, transform)) = read_query.get(entity) else {
            continue;
        };
        // Check time budget
        if start_time.elapsed().as_secs_f32() * 1000.0 > max_processing_time {
            break;
        }

        // Check operation limit
        if render_operations >= max_render_operations {
            break;
        }

        // Skip LOD updates for very distant entities (beyond 300m)
        if distance > 300.0 {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
                render_operations += 1;
            }
            continue;
        }

        // View frustum culling (simplified)
        if !is_in_view_frustum(transform, &active_transform, 90.0, 500.0) {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
                render_operations += 1;
            }
            continue;
        }

        // Update visibility based on distance
        let lod_distance = config.world.lod_distances.get(2).copied().unwrap_or(500.0);
        let should_be_visible = distance < lod_distance;
        let new_visibility = if should_be_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if *visibility != new_visibility {
            *visibility = new_visibility;
            render_operations += 1;
        }
    }

    // Performance monitoring
    let processing_time = start_time.elapsed().as_secs_f32() * 1000.0;
    if processing_time > 3.0 {
        warn!(
            "Render optimization took {:.2}ms (> 3ms budget), {} operations",
            processing_time, render_operations
        );
    }
}

/// Simplified view frustum culling
fn is_in_view_frustum(
    object_transform: &Transform,
    camera_transform: &Transform,
    fov_degrees: f32,
    max_distance: f32,
) -> bool {
    let to_object = object_transform.translation - camera_transform.translation;
    let distance = to_object.length();

    // Distance check
    if distance > max_distance {
        return false;
    }

    // Simple FOV check (not perfect but fast)
    let forward = camera_transform.forward();
    let to_object_normalized = to_object.normalize();
    let dot_product = forward.dot(to_object_normalized);

    // Convert FOV to radians and check if object is within view
    let fov_radians = fov_degrees.to_radians();
    let cos_half_fov = (fov_radians * 0.5).cos();

    dot_product > cos_half_fov
}

/// Batch rendering system with operation limits
#[cfg(feature = "simple_render_culler")]
pub fn batch_rendering_system(
    read_query: Query<(Entity, &Transform), With<Cullable>>,
    mut visibility_query: Query<&mut Visibility, With<Cullable>>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    config: Res<GameConfig>,
    _time: Res<Time>,
) {
    let start_time = Instant::now();

    let Ok(active_transform) = active_query.single() else {
        return;
    };
    let active_pos = active_transform.translation;

    let max_processing_time = 2.0; // 2ms time budget
    let max_render_batches = 20; // Limit render batches per frame
    let mut render_batches = 0;

    // Pass 1: collect (Entity, distance) for sorting (read-only for thread safety)
    let mut entities: Vec<(Entity, f32)> = read_query
        .iter()
        .map(|(entity, transform)| {
            let distance = active_pos.distance(transform.translation);
            (entity, distance)
        })
        .collect();

    entities.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let batch_size = 10; // Process 10 entities per batch

    for batch in entities.chunks(batch_size) {
        if start_time.elapsed().as_secs_f32() * 1000.0 > max_processing_time {
            break;
        }

        if render_batches >= max_render_batches {
            break;
        }

        // Process this batch
        for &(entity, distance) in batch {
            let Ok(mut visibility) = visibility_query.get_mut(entity) else {
                continue;
            };

            // Simple distance-based visibility using world streaming radius
            let should_be_visible = distance < config.world.streaming_radius;

            let new_visibility = if should_be_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            if *visibility != new_visibility {
                *visibility = new_visibility;
            }
        }

        render_batches += 1;
    }

    // Performance monitoring
    let processing_time = start_time.elapsed().as_secs_f32() * 1000.0;
    if processing_time > 2.0 {
        warn!(
            "Batch rendering took {:.2}ms (> 2ms budget), {} batches",
            processing_time, render_batches
        );
    }
}

/// System to manage render queue and prevent frame drops
#[cfg(feature = "simple_render_culler")]
pub fn render_queue_manager_system(
    pending_render_query: Query<Entity, (With<Visibility>, Changed<Transform>)>,
    _time: Res<Time>,
    mut frame_budget: Local<f32>,
) {
    let start_time = Instant::now();
    let frame_time_budget = 16.67; // Target 60 FPS (16.67ms per frame)

    // Reset frame budget every frame
    *frame_budget = frame_time_budget;

    let max_render_updates = 25; // Limit render updates per frame
    let mut render_updates = 0;

    for _entity in pending_render_query.iter() {
        if start_time.elapsed().as_secs_f32() * 1000.0 > 1.0 {
            break; // Don't spend more than 1ms on render queue management
        }

        if render_updates >= max_render_updates {
            break;
        }

        render_updates += 1;
    }

    // Update frame budget
    *frame_budget -= start_time.elapsed().as_secs_f32() * 1000.0;

    if *frame_budget < 5.0 {
        warn!("Frame budget low: {:.2}ms remaining", *frame_budget);
    }
}

#[cfg(feature = "simple_render_culler")]
pub struct SimpleRenderCullerPlugin;

#[cfg(feature = "simple_render_culler")]
impl Plugin for SimpleRenderCullerPlugin {
    fn build(&self, app: &mut App) {
        use bevy::time::common_conditions::on_timer;
        use std::time::Duration;

        app.add_systems(
            FixedUpdate,
            render_optimization_system.run_if(on_timer(Duration::from_millis(500))),
        )
        .add_systems(
            FixedUpdate,
            batch_rendering_system.run_if(on_timer(Duration::from_millis(300))),
        )
        .add_systems(Update, render_queue_manager_system);
    }
}
