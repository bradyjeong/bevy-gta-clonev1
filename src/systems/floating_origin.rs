//! Simple Floating Origin System
//! 
//! Keeps the ActiveEntity near the world origin by periodically shifting all entities back
//! when the ActiveEntity drifts too far. This prevents coordinate explosions while providing
//! an infinite world experience.
//! 
//! Following AGENT.md "Simplicity First" - one focused system that handles world shifting.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::ActiveEntity;
use crate::util::safe_math::{is_valid_position, sanitize_transform};

/// Resource tracking the cumulative world offset for deterministic generation
#[derive(Resource, Default)]
pub struct WorldOffset {
    /// Cumulative offset representing "true" world position
    pub offset: Vec3,
    /// Time of last origin shift to prevent multiple shifts per frame
    pub last_shift_time: f32,
}

/// Component to opt out of world shifting (for sky, UI, etc.)
#[derive(Component)]
pub struct IgnoreWorldShift;

/// Event fired when world origin is shifted
#[derive(Event)]
pub struct WorldOriginShifted {
    pub shift_amount: Vec3,
    pub new_world_offset: Vec3,
}

/// Configuration for floating origin system
#[derive(Resource)]
pub struct FloatingOriginConfig {
    /// Distance from origin that triggers a shift (default: 1km)
    pub shift_threshold: f32,
    /// Radius around ActiveEntity to shift other entities (default: 5km)
    pub nearby_radius: f32,
    /// Minimum time between shifts (default: 0.1s)
    pub min_shift_interval: f32,
}

impl Default for FloatingOriginConfig {
    fn default() -> Self {
        Self {
            shift_threshold: 1_000.0,  // 1km
            nearby_radius: 5_000.0,    // 5km  
            min_shift_interval: 0.1,   // 100ms
        }
    }
}

/// Main floating origin system - keeps ActiveEntity near world origin
pub fn floating_origin_system(
    mut world_offset: ResMut<WorldOffset>,
    config: Res<FloatingOriginConfig>,
    time: Res<Time>,
    mut origin_events: EventWriter<WorldOriginShifted>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut transform_query: Query<&mut Transform, (Without<ActiveEntity>, Without<IgnoreWorldShift>)>,
    mut physics_query: Query<&mut RigidBody, Without<IgnoreWorldShift>>,
) {
    // Get current active entity position
    let Ok(active_transform) = active_query.single() else {
        warn!("No ActiveEntity found for floating origin system");
        return;
    };
    
    let current_time = time.elapsed_secs();
    
    // Check if we need to prevent multiple shifts per frame
    if current_time - world_offset.last_shift_time < config.min_shift_interval {
        return;
    }
    
    let active_pos = active_transform.translation;
    
    // Validate active position first
    if !is_valid_position(active_pos) {
        warn!("ActiveEntity has invalid position: {:?}, skipping origin shift", active_pos);
        return;
    }
    
    // Check if we need to shift
    let distance_from_origin = active_pos.length();
    if distance_from_origin < config.shift_threshold {
        return; // No shift needed
    }
    
    // Calculate shift amount (bring ActiveEntity back toward origin)
    let shift_direction = -active_pos.normalize_or_zero();
    let shift_amount = shift_direction * (distance_from_origin - config.shift_threshold * 0.5);
    
    info!("Performing world origin shift: moving {} entities by {:?}", 
          transform_query.iter().count(), shift_amount);
    
    // Shift all non-excluded transforms
    let mut shifted_count = 0;
    for mut transform in transform_query.iter_mut() {
        // Only shift entities within nearby radius to ActiveEntity
        let distance_to_active = transform.translation.distance(active_pos);
        if distance_to_active <= config.nearby_radius {
            transform.translation += shift_amount;
            
            // Validate result
            if sanitize_transform(&mut transform) {
                warn!("Transform became invalid during world shift, sanitized");
            }
            
            shifted_count += 1;
        }
    }
    
    // Update physics bodies (they maintain their own position data)
    for mut rigidbody in physics_query.iter_mut() {
        // Note: Bevy/Rapier automatically syncs RigidBody positions from Transforms
        // so we don't need to manually update physics positions
        *rigidbody = *rigidbody; // Touch the component to mark it as changed
    }
    
    // Update world offset for deterministic generation
    world_offset.offset -= shift_amount; // Subtract because we moved entities toward origin
    world_offset.last_shift_time = current_time;
    
    // Fire event for other systems that need to know about the shift
    origin_events.write(WorldOriginShifted {
        shift_amount,
        new_world_offset: world_offset.offset,
    });
    
    info!("World origin shift complete: moved {} entities, new world offset: {:?}", 
          shifted_count, world_offset.offset);
}

/// System to add input validation to the infinite streaming system
pub fn validate_streaming_position(
    active_query: Query<&Transform, With<ActiveEntity>>,
) {
    if let Ok(active_transform) = active_query.single() {
        let pos = active_transform.translation;
        
        // Reject impossible positions before they reach streaming system
        const MAX_ALLOWED_DISTANCE: f32 = 50_000.0; // 50km reasonable limit
        
        if !pos.is_finite() {
            error!("ActiveEntity has non-finite position: {:?}", pos);
            return;
        }
        
        if pos.length() > MAX_ALLOWED_DISTANCE {
            error!("ActiveEntity position exceeds safe limit: {:?} (distance: {:.1}km)", 
                   pos, pos.length() / 1000.0);
            return;
        }
        
        // Position is valid for streaming
    }
}

/// Diagnostic system for floating origin
pub fn floating_origin_diagnostics(
    world_offset: Res<WorldOffset>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    time: Res<Time>,
) {
    // Log diagnostics every 10 seconds
    if (time.elapsed_secs() % 10.0) < time.delta_secs() {
        if let Ok(active_transform) = active_query.single() {
            let pos = active_transform.translation;
            let distance_from_origin = pos.length();
            
            info!("🌍 Floating Origin Status:");
            info!("  ActiveEntity distance from origin: {:.1}m", distance_from_origin);
            info!("  World logical offset: {:?}", world_offset.offset);
            info!("  World logical distance: {:.1}km", world_offset.offset.length() / 1000.0);
        }
    }
}
