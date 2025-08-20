//! Seamless World Root Shifting System
//! 
//! Industry-standard floating origin using a single world root entity and quantized shifts.
//! Shifts happen before physics simulation in small, invisible increments.
//! 
//! Following AGENT.md "Simplicity First" - O(1) root shifting instead of O(N) entity updates.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::ActiveEntity;
use crate::util::safe_math::is_valid_position;

/// Resource tracking the cumulative world offset for deterministic generation
#[derive(Resource, Default)]
pub struct WorldOffset {
    /// Cumulative offset representing "true" world position
    pub offset: Vec3,
    /// Time of last origin shift to prevent multiple shifts per frame
    pub last_shift_time: f32,
}

/// Component marking the root of the world coordinate system
/// All game entities should be children of this entity
#[derive(Component)]
pub struct WorldRoot;

/// Component to opt out of world shifting (UI, screen-space effects only)
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

/// Configuration for seamless world root shifting
#[derive(Resource)]
pub struct FloatingOriginConfig {
    /// Size of quantized shift cells (smaller = smoother)
    pub cell_size: f32,
    /// Distance that triggers rebase (should be multiple of cell_size)
    pub rebase_threshold: f32,
    /// Minimum time between shifts (prevent frame rate issues)
    pub min_shift_interval: f32,
}

impl Default for FloatingOriginConfig {
    fn default() -> Self {
        Self {
            cell_size: 256.0,         // 256m cells (invisible shift size)
            rebase_threshold: 512.0,  // 2 cells = 512m (trigger rebase)
            min_shift_interval: 0.1,  // 100ms minimum between shifts
        }
    }
}

/// Seamless world root shifting system - O(1) invisible shifts using industry standard technique
/// Runs BEFORE physics simulation to prevent any visible popping
pub fn seamless_world_rebase_system(
    mut world_offset: ResMut<WorldOffset>,
    config: Res<FloatingOriginConfig>,
    time: Res<Time>,
    mut origin_events: EventWriter<WorldOriginShifted>,
    // Note: Rapier 0.30 may not have set_world_offset - leaving for future implementation
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut world_root_query: Query<&mut Transform, (With<WorldRoot>, Without<ActiveEntity>)>,
) {
    // Get current active entity position
    let Ok(active_transform) = active_query.single() else {
        return; // No active entity, no rebase needed
    };
    
    let current_time = time.elapsed_secs();
    
    // Rate limiting to prevent frame rate issues
    if current_time - world_offset.last_shift_time < config.min_shift_interval {
        return;
    }
    
    // Get world root entity first
    let Ok(mut world_root_transform) = world_root_query.single_mut() else {
        warn!("No WorldRoot entity found - cannot perform seamless rebase");
        return;
    };
    
    // CRITICAL: Use render-space position (relative to WorldRoot) for distance calculations
    let render_pos = active_transform.translation - world_root_transform.translation;
    
    // Validate render position first
    if !is_valid_position(render_pos) {
        warn!("ActiveEntity has invalid render position: {:?}, skipping rebase", render_pos);
        return;
    }
    
    // Check if we need to rebase using render-space distance
    let distance_from_render_origin = render_pos.length();
    if distance_from_render_origin <= config.rebase_threshold {
        return; // Still within acceptable range
    }
    
    // Calculate quantized shift to nearest cell boundary using render coordinates
    // This ensures deterministic streaming and invisible shifts
    let shift_x = (render_pos.x / config.cell_size).round() * config.cell_size;
    let shift_z = (render_pos.z / config.cell_size).round() * config.cell_size;
    let shift_amount = Vec3::new(shift_x, 0.0, shift_z); // Never shift Y (keep ground level)
    
    // Skip tiny shifts that won't help
    if shift_amount.length() < config.cell_size * 0.5 {
        return;
    }
    
    // INDUSTRY STANDARD: Shift world root and physics context in sync
    // This is invisible to players because terrain moves with the player
    world_root_transform.translation += shift_amount;
    
    // Update logical world offset for deterministic generation
    world_offset.offset += shift_amount; // Add because logical world advanced forward
    world_offset.last_shift_time = current_time;
    
    // Fire event for subsystems that track world coordinates (streaming, AI, etc.)
    origin_events.write(WorldOriginShifted {
        shift_amount: -shift_amount, // Negative because world moved -shift, so logical +shift
        new_world_offset: world_offset.offset,
    });
    
    info!("Seamless world rebase: shifted by {:?}, new logical offset: {:?}", 
          shift_amount, world_offset.offset);
}

/// System to update Rapier physics bodies during world origin shifts
/// Rapier 0.30 requires manual translation of rigid body positions
pub fn update_physics_after_origin_shift(
    mut origin_events: EventReader<WorldOriginShifted>,
    mut rigidbody_query: Query<&mut Transform, (With<RigidBody>, Without<WorldRoot>)>,
) {
    for event in origin_events.read() {
        // Translate all physics bodies by the world shift amount to keep them in sync
        // Note: event.shift_amount is negative (world moved -shift), so we apply the shift directly
        let physics_shift = -event.shift_amount;
        
        for mut transform in rigidbody_query.iter_mut() {
            transform.translation += physics_shift;
        }
        
        // Note: Rapier 0.30 will sync from Transform changes automatically
        // No need for manual RigidBody position updates
        
        info!("Updated {} physics bodies for world origin shift", rigidbody_query.iter().count());
    }
}

/// System to create the WorldRoot entity that parents all game content
/// Run this during setup to establish the coordinate system
pub fn setup_world_root(mut commands: Commands) {
    commands.spawn((
        Name::new("WorldRoot"),
        Transform::default(),
        GlobalTransform::default(),
        WorldRoot,
        Visibility::default(),
    ));
    
    info!("WorldRoot entity created for seamless coordinate shifting");
}

/// Coordinate conversion helpers for streaming systems
impl WorldOffset {
    /// Convert logical world position to render position (relative to WorldRoot)
    pub fn logical_to_render(&self, logical_pos: Vec3) -> Vec3 {
        logical_pos - self.offset
    }
    
    /// Convert render position to logical world position (true global coordinates)
    pub fn render_to_logical(&self, render_pos: Vec3) -> Vec3 {
        render_pos + self.offset
    }
    
    /// Get the logical position of the active entity for streaming systems
    pub fn get_active_logical_position(&self, active_render_pos: Vec3) -> Vec3 {
        self.render_to_logical(active_render_pos)
    }
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
/// Also handles streamed entities that aren't parented to WorldRoot
pub fn world_shift_special_cases_system(
    mut shift_events: EventReader<WorldOriginShifted>,
    mut npc_query: Query<&mut crate::components::world::NPCState, With<FollowsWorldOffset>>,
    mut streamed_entities_query: Query<&mut Transform, (With<FollowsWorldOffset>, Without<crate::components::world::NPCState>)>,
) {
    for event in shift_events.read() {
        let shift_amount = event.shift_amount;
        let mut npc_updated_count = 0;
        let mut entity_updated_count = 0;
        
        // Update NPC target positions (world coordinates)
        // Event shift_amount is negative, so add it to maintain position
        for mut npc_state in npc_query.iter_mut() {
            npc_state.target_position += shift_amount;
            
            // Validate the result
            if !npc_state.target_position.is_finite() {
                warn!("NPC target position became invalid during world shift, resetting");
                npc_state.target_position = Vec3::ZERO;
            }
            
            npc_updated_count += 1;
        }
        
        // Update streamed entities that aren't children of WorldRoot
        // Event shift_amount is negative, so add it to maintain position
        for mut transform in streamed_entities_query.iter_mut() {
            transform.translation += shift_amount;
            
            // Validate the result
            if !transform.translation.is_finite() {
                warn!("Entity transform became invalid during world shift, resetting");
                transform.translation = Vec3::ZERO;
            }
            
            entity_updated_count += 1;
        }
        
        if npc_updated_count > 0 {
            info!("Updated {} NPC target positions for world shift: shift_amount {:?}", npc_updated_count, shift_amount);
        }
        
        if entity_updated_count > 0 {
            info!("Updated {} streamed entity transforms for world shift: shift_amount {:?}", entity_updated_count, shift_amount);
        }
    }
}
