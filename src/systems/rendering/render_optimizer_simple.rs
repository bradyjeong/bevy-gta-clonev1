use bevy::prelude::*;
use std::time::Instant;

use crate::components::ActiveEntity;
use crate::systems::world::unified_distance_culling::UnifiedCullable;
use crate::config::GameConfig;

/// Simplified render optimization system with batching limits and view frustum culling
pub fn render_optimization_system(
    mut render_query: Query<(Entity, &mut Visibility, &Transform), With<UnifiedCullable>>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    config: Res<GameConfig>,
    time: Res<Time>,
    mut last_update: Local<f32>,
) {
    let start_time = Instant::now();
    let current_time = time.elapsed_secs();
    
    // Update less frequently to reduce overhead
    if current_time - *last_update < 0.5 {
        return;
    }
    *last_update = current_time;
    
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    let max_processing_time = 3.0; // 3ms time budget
    let max_render_operations = 20; // Limit render operations per frame
    let mut render_operations = 0;
    
    // Sort entities by distance for priority processing
    let mut entities_with_distance: Vec<_> = render_query.iter_mut()
        .map(|(entity, visibility, transform)| {
            let distance = active_pos.distance(transform.translation);
            (distance, entity, visibility, transform)
        })
        .collect();
    
    entities_with_distance.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    
    for (distance, _entity, mut visibility, transform) in entities_with_distance {
        // Check time budget
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
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
        let should_be_visible = distance < config.world.lod_distances[2]; // Use config lod distance
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
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > 2.0 {
        warn!("Render optimization took {:.2}ms (> 2ms budget), {} operations", processing_time, render_operations);
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
pub fn batch_rendering_system(
    mut renderable_query: Query<(Entity, &mut Visibility, &Transform), With<UnifiedCullable>>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    config: Res<GameConfig>,
    time: Res<Time>,
    mut last_update: Local<f32>,
) {
    let start_time = Instant::now();
    let current_time = time.elapsed_secs();
    
    // Update every 0.3 seconds to reduce overhead
    if current_time - *last_update < 0.3 {
        return;
    }
    *last_update = current_time;
    
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    let max_processing_time = 2.0; // 2ms time budget
    let max_render_batches = 20; // Limit render batches per frame
    let mut render_batches = 0;
    
    // Process entities in batches based on distance
    let mut entities_to_process: Vec<_> = renderable_query.iter_mut().collect();
    entities_to_process.sort_by(|a, b| {
        let dist_a = active_pos.distance(a.2.translation);
        let dist_b = active_pos.distance(b.2.translation);
        dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    let batch_size = 10; // Process 10 entities per batch
    
    for batch in entities_to_process.chunks_mut(batch_size) {
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        if render_batches >= max_render_batches {
            break;
        }
        
        // Process this batch
        for (_entity, visibility, transform) in batch {
            let distance = active_pos.distance(transform.translation);
            
            // Simple distance-based visibility using world streaming radius
            let should_be_visible = distance < config.world.streaming_radius;
            
            let new_visibility = if should_be_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            
            if **visibility != new_visibility {
                **visibility = new_visibility;
            }
        }
        
        render_batches += 1;
    }
    
    // Performance monitoring
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > 1.5 {
        warn!("Batch rendering took {:.2}ms (> 1.5ms budget), {} batches", processing_time, render_batches);
    }
}

/// System to manage render queue and prevent frame drops
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
        if start_time.elapsed().as_millis() as f32 > 1.0 {
            break; // Don't spend more than 1ms on render queue management
        }
        
        if render_updates >= max_render_updates {
            break;
        }
        
        render_updates += 1;
    }
    
    // Update frame budget
    *frame_budget -= start_time.elapsed().as_millis() as f32;
    
    if *frame_budget < 5.0 {
        warn!("Frame budget low: {:.2}ms remaining", *frame_budget);
    }
}
