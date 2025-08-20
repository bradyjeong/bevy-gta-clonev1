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
use crate::util::safe_math::{is_valid_position, validate_transform};

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

/// Component for entities that need special handling during world origin shifts
/// Used for entities with position data separate from Transform component
#[derive(Component)]
pub struct FollowsWorldOffset;

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
            shift_threshold: 5_000.0,  // 5km (conservative for testing)
            nearby_radius: 10_000.0,   // 10km (shift everything nearby)
            min_shift_interval: 1.0,   // 1s (prevent rapid shifts)
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
    mut all_transforms_query: Query<(&mut Transform, Option<&RigidBody>), (Without<ActiveEntity>, Without<IgnoreWorldShift>)>,
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
    
    let total_entities = all_transforms_query.iter().count();
    info!("Performing world origin shift: moving {} entities by {:?}", total_entities, shift_amount);
    
    let mut nearby_shifted = 0;
    let mut physics_shifted = 0;
    
    // Process all entities in a single pass
    for (mut transform, rigidbody) in all_transforms_query.iter_mut() {
        let distance_to_active = transform.translation.distance(active_pos);
        let is_physics_body = rigidbody.is_some();
        
        // CRITICAL: Always shift physics bodies to prevent orphaned bodies causing Rapier panics
        // For non-physics entities, only shift if nearby (performance optimization)
        let should_shift = is_physics_body || distance_to_active <= config.nearby_radius;
        
        if should_shift {
            transform.translation += shift_amount;
            
            // Validate result
            if validate_transform(&mut transform) {
                warn!("Transform became invalid during world shift, sanitized");
            }
            
            if is_physics_body {
                physics_shifted += 1;
            } else {
                nearby_shifted += 1;
            }
        }
    }
    
    // Update world offset for deterministic generation
    world_offset.offset -= shift_amount; // Subtract because we moved entities toward origin
    world_offset.last_shift_time = current_time;
    
    // Fire event for other systems that need to know about the shift
    origin_events.write(WorldOriginShifted {
        shift_amount,
        new_world_offset: world_offset.offset,
    });
    
    info!("World origin shift complete: moved {} nearby + {} physics entities, new world offset: {:?}", 
          nearby_shifted, physics_shifted, world_offset.offset);
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

/// Safety system that regularly checks for orphaned entities beyond safe bounds
pub fn world_sanity_check_system(
    mut commands: Commands,
    rigidbody_query: Query<(Entity, &Transform, Option<&Name>), With<RigidBody>>,
    time: Res<Time>,
) {
    // Run sanity check every 5 seconds
    if (time.elapsed_secs() % 5.0) < time.delta_secs() {
        const MAX_SAFE_DISTANCE: f32 = 100_000.0; // 100km safety limit
        let mut culled_count = 0;
        
        for (entity, transform, name) in rigidbody_query.iter() {
            let distance = transform.translation.length();
            
            if !is_valid_position(transform.translation) || distance > MAX_SAFE_DISTANCE {
                let entity_name = name.map(|n| n.as_str()).unwrap_or("Unknown");
                warn!("Culling orphaned physics entity '{}' at distance {:.1}km", entity_name, distance / 1000.0);
                
                commands.entity(entity).despawn();
                culled_count += 1;
            }
        }
        
        if culled_count > 0 {
            info!("World sanity check: culled {} orphaned physics entities", culled_count);
        }
    }
}

/// System that handles special cases during world origin shifts
/// Listens for WorldOriginShifted events and updates entities with separate position data
pub fn world_shift_special_cases_system(
    mut shift_events: EventReader<WorldOriginShifted>,
    mut npc_query: Query<&mut crate::components::world::NPCState, With<FollowsWorldOffset>>,
) {
    for event in shift_events.read() {
        let shift_amount = event.shift_amount;
        let mut updated_count = 0;
        
        // Update NPC target positions (world coordinates)
        for mut npc_state in npc_query.iter_mut() {
            npc_state.target_position += shift_amount;
            
            // Validate the result
            if !npc_state.target_position.is_finite() {
                warn!("NPC target position became invalid during world shift, resetting");
                npc_state.target_position = Vec3::ZERO;
            }
            
            updated_count += 1;
        }
        
        if updated_count > 0 {
            info!("Updated {} NPC target positions for world shift: shift_amount {:?}", updated_count, shift_amount);
        }
    }
}
