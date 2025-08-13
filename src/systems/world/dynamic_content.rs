use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{DynamicTerrain, ActiveEntity, DynamicContent, Car};
use crate::GlobalRng;
use crate::components::world::EntityLimits;
use crate::events::world::validation_events::{RequestSpawnValidation, ValidationId};
use crate::events::world::content_events::{ContentType as EventContentType};
use crate::events::world::chunk_events::{ChunkLoaded, ChunkCoord};
use crate::components::world::ContentType;
use std::collections::HashMap;

pub fn dynamic_terrain_system(
    mut terrain_query: Query<&mut Transform, (With<DynamicTerrain>, Without<ActiveEntity>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicTerrain>)>,
) {
    if let Ok(active_transform) = active_query.single() {
        if let Ok(mut terrain_transform) = terrain_query.single_mut() {
            // Only move terrain if player has moved significantly to prevent sliding
            let distance_moved = (active_transform.translation.xz() - terrain_transform.translation.xz()).length();
            
            if distance_moved > 50.0 {  // Only follow when player moves 50+ units
                terrain_transform.translation.x = active_transform.translation.x;
                terrain_transform.translation.z = active_transform.translation.z;
                terrain_transform.translation.y = -0.1; // 10cm below road surface to prevent z-fighting
            }
        }
    }
}

// Observer-based chunk tracking (replaces timer-based polling)
#[derive(Resource, Default)]
pub struct ChunkContentTracker {
    loaded_chunks: std::collections::HashSet<ChunkCoord>,
    last_player_chunk: Option<ChunkCoord>,
}

impl ChunkContentTracker {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn is_chunk_loaded(&self, coord: ChunkCoord) -> bool {
        self.loaded_chunks.contains(&coord)
    }
    
    pub fn get_loaded_chunks(&self) -> &std::collections::HashSet<ChunkCoord> {
        &self.loaded_chunks
    }
}

// Track validation requests for dynamic content
#[derive(Default)]
pub struct DynamicValidationTracker {
    pending_validations: HashMap<u32, (Vec3, EventContentType)>,
    next_validation_id: u32,
}

impl DynamicValidationTracker {
    pub fn new_id(&mut self) -> u32 {
        let id = self.next_validation_id;
        self.next_validation_id = self.next_validation_id.wrapping_add(1);
        id
    }
}

/// Observer-based dynamic content spawning (replaces timer-based polling)
/// Responds to ChunkLoaded events to spawn content reactively
pub fn on_chunk_loaded(
    trigger: Trigger<ChunkLoaded>,
    _commands: Commands,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicContent>)>,
    content_query: Query<(Entity, &Transform, &DynamicContent)>,
    existing_vehicles_query: Query<&Transform, (With<Car>, Without<DynamicContent>)>,
    _entity_limits: ResMut<EntityLimits>,
    mut validation_tracker: Local<DynamicValidationTracker>,
    mut validation_writer: EventWriter<RequestSpawnValidation>,
    mut chunk_tracker: ResMut<ChunkContentTracker>,
    mut rng: ResMut<GlobalRng>,
) {
    let chunk_loaded = trigger.event();
    let chunk_coord = chunk_loaded.coord;
    
    // Mark chunk as loaded for tracking
    chunk_tracker.loaded_chunks.insert(chunk_coord);
    
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        let current_chunk = ChunkCoord::from_world_pos(active_pos, 200.0); // 200m chunk size
        
        // Only spawn content if player is in or near this chunk
        let chunk_distance = ((chunk_coord.x - current_chunk.x).abs() + (chunk_coord.z - current_chunk.z).abs()) as f32;
        if chunk_distance > 2.0 { // Only spawn in adjacent chunks
            return;
        }
        
        chunk_tracker.last_player_chunk = Some(current_chunk);
        
        // PERFORMANCE: Frame time budgeting - max 2ms per chunk
        let frame_start_time = std::time::Instant::now();
        
        // Calculate chunk center for spawning
        let chunk_center = Vec3::new(
            (chunk_coord.x as f32) * 200.0 + 100.0,  // Chunk size 200m + half offset
            0.0,
            (chunk_coord.z as f32) * 200.0 + 100.0,
        );
        
        // Reduced spawn parameters for observer-based system
        let spawn_radius = 150.0;    // Radius per chunk
        let spawn_density = 80.0;    // Spacing between entities
        let max_spawn_attempts = 10; // Limited per chunk
        
        // Collect existing content for collision avoidance
        let mut existing_content: Vec<(Vec3, ContentType, f32)> = content_query.iter()
            .map(|(_, transform, dynamic_content)| {
                let radius = match dynamic_content.content_type {
                    ContentType::Building => 20.0,
                    ContentType::Road => 15.0,
                    ContentType::Tree => 8.0,
                    ContentType::Vehicle => 25.0,
                    ContentType::NPC => 3.0,
                };
                (transform.translation, dynamic_content.content_type.clone(), radius)
            })
            .collect();
            
        // Add existing vehicles to collision avoidance
        for vehicle_transform in existing_vehicles_query.iter() {
            existing_content.push((vehicle_transform.translation, ContentType::Vehicle, 25.0));
        }
        
        // Spawn content in circular pattern around chunk center
        let mut spawn_attempts = 0;
        for radius_step in (spawn_density as i32..spawn_radius as i32).step_by(spawn_density as usize) {
            let radius = radius_step as f32;
            let circumference = 2.0 * std::f32::consts::PI * radius;
            let points_on_circle = (circumference / spawn_density).max(6.0) as i32;
            
            for i in 0..points_on_circle {
                spawn_attempts += 1;
                if spawn_attempts > max_spawn_attempts { break; }
                
                // PERFORMANCE: Check frame time budget
                if frame_start_time.elapsed().as_millis() > 2 {
                    break; // Exit early to maintain frame rate
                }
                
                let angle = (i as f32 / points_on_circle as f32) * 2.0 * std::f32::consts::PI;
                let spawn_x = chunk_center.x + radius * angle.cos();
                let spawn_z = chunk_center.z + radius * angle.sin();
                let spawn_pos = Vec3::new(spawn_x, 0.0, spawn_z);
                
                // Only spawn if no content exists nearby
                if !has_content_at_position(spawn_pos, &existing_content, spawn_density * 0.8) {
                    request_spawn_validation(spawn_pos, &mut validation_tracker, &mut validation_writer, &mut rng);
                }
            }
            if spawn_attempts > max_spawn_attempts { break; }
        }
        
        trace!("Observer spawned content for chunk {:?}, attempts: {}", chunk_coord, spawn_attempts);
    }
}

/// Legacy cleanup system for distant content (maintained for compatibility)
/// Spawning is now handled by the on_chunk_loaded observer
pub fn cleanup_distant_content(
    mut commands: Commands,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicContent>)>,
    content_query: Query<(Entity, &Transform, &DynamicContent)>,
    mut chunk_tracker: ResMut<ChunkContentTracker>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        let cleanup_radius = 2500.0;
        
        // Only cleanup - spawning is now handled by observers
        for (entity, content_transform, _) in content_query.iter() {
            let distance = active_pos.distance(content_transform.translation);
            if distance > cleanup_radius {
                commands.entity(entity).despawn();
            }
        }
        
        // Update current chunk tracking
        let current_chunk = ChunkCoord::from_world_pos(active_pos, 200.0);
        chunk_tracker.last_player_chunk = Some(current_chunk);
    }
}

// Export the cleanup system for plugin registration
pub use cleanup_distant_content as query_dynamic_content;

fn has_content_at_position(position: Vec3, existing_content: &[(Vec3, ContentType, f32)], min_distance: f32) -> bool {
    existing_content.iter().any(|(existing_pos, _, radius)| {
        // Fixed: Use sum of distances plus buffer instead of max
        let required_distance = min_distance + radius + 2.0; // 2.0 buffer
        position.distance(*existing_pos) < required_distance
    })
}

// Dead function removed - spawn_dynamic_content_safe was never used







pub fn vehicle_separation_system(
    mut vehicle_query: Query<(&mut Transform, &mut Velocity), (With<Car>, With<DynamicContent>)>,
) {
    let vehicles: Vec<(Vec3, Entity)> = vehicle_query.iter()
        .enumerate()
        .map(|(i, (transform, _))| (transform.translation, Entity::from_raw(i as u32)))
        .collect();
    
    for (mut transform, mut velocity) in vehicle_query.iter_mut() {
        let current_pos = transform.translation;
        
        for (other_pos, _) in &vehicles {
            if *other_pos == current_pos { continue; }
            
            let distance = current_pos.distance(*other_pos);
            if distance < 15.0 && distance > 0.1 { // Too close
                let separation_force = (current_pos - *other_pos).normalize() * (15.0 - distance) * 2.0;
                velocity.linvel += separation_force;
                
                // Also adjust position slightly to prevent exact overlap
                transform.translation += separation_force * 0.1;
            }
        }
    }
}



// REMOVED: Dead code functions replaced by UnifiedEntityFactory
// - spawn_building() -> use UnifiedEntityFactory::spawn_building_consolidated()
// - spawn_vehicle() -> use UnifiedEntityFactory::spawn_vehicle_consolidated()  
// - spawn_dynamic_tree() -> use UnifiedEntityFactory::spawn_tree_consolidated()
// - spawn_dynamic_npc() -> use UnifiedEntityFactory::spawn_npc_consolidated()
// 
// These functions were marked with #[allow(dead_code)] and have been
// consolidated into the unified spawning pipeline for Phase 3.

/// Request spawn validation for dynamic content (Phase 3 - Event-driven)
/// This replaces spawn_dynamic_content_safe_unified using events
fn request_spawn_validation(
    position: Vec3,
    validation_tracker: &mut DynamicValidationTracker,
    validation_writer: &mut EventWriter<RequestSpawnValidation>,
    rng: &mut GlobalRng,
) {
    // Ultra-reduced spawn rates from AGENT.md (buildings 8%, vehicles 4%, trees 5%, NPCs 1%)
    let content_type = if rng.gen_range(0.0..1.0) < 0.08 {
        Some(EventContentType::Building)
    } else if rng.gen_range(0.0..1.0) < 0.04 {
        Some(EventContentType::Vehicle)
    } else if rng.gen_range(0.0..1.0) < 0.05 {
        Some(EventContentType::Tree)
    } else if rng.gen_range(0.0..1.0) < 0.01 {
        Some(EventContentType::NPC)
    } else {
        None
    };
    
    if let Some(content_type) = content_type {
        let validation_id = validation_tracker.new_id();
        validation_tracker.pending_validations.insert(validation_id, (position, content_type));
        
        validation_writer.write(RequestSpawnValidation::new(
            ValidationId::new(validation_id),
            position,
            content_type,
        ));
    }
}




