use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{DynamicTerrain, ActiveEntity, DynamicContent, Car};
use crate::GlobalRng;
use crate::components::world::EntityLimits;
use crate::events::world::validation_events::{RequestSpawnValidation, ValidationId};
use crate::events::world::content_events::{ContentType as EventContentType};
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

// Add timer to reduce frequency of dynamic content checks
#[derive(Default)]
pub struct DynamicContentTimer {
    timer: f32,
    last_player_pos: Option<Vec3>,
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

pub fn query_dynamic_content(
    mut commands: Commands,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicContent>)>,
    content_query: Query<(Entity, &Transform, &DynamicContent)>,
    existing_vehicles_query: Query<&Transform, (With<Car>, Without<DynamicContent>)>,
    _entity_limits: ResMut<EntityLimits>,
    time: Res<Time>,
    mut timer: Local<DynamicContentTimer>,
    mut validation_tracker: Local<DynamicValidationTracker>,
    mut validation_writer: EventWriter<RequestSpawnValidation>,
    mut rng: ResMut<GlobalRng>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        
        // Update timer
        timer.timer += time.delta_secs();
        
        // PERFORMANCE: Frame time budgeting - max 3ms per frame
        let frame_start_time = std::time::Instant::now();
        
        // CRITICAL PERFORMANCE OPTIMIZATION: Process every 8.0 seconds OR when player moves significantly
        let movement_threshold = 100.0;
        let player_moved = timer.last_player_pos
            .map(|last_pos| active_pos.distance(last_pos) > movement_threshold)
            .unwrap_or(true);
        
        let should_update = timer.timer >= 8.0 || player_moved;
        
        if !should_update {
            return;
        }
        
        timer.timer = 0.0;
        timer.last_player_pos = Some(active_pos);
        
        // EMERGENCY PERFORMANCE MODE - Drastically reduce entity spawning
        let active_radius = 100.0;   // REDUCED: Minimal spawn radius from 150.0 to 100.0
        let cleanup_radius = 2500.0;  // Match road cleanup radius to prevent premature despawning
        let spawn_density = 120.0;   // INCREASED: Much higher spacing between entities
        
        // Phase 1: Remove content outside cleanup radius (truly circular)
        for (entity, content_transform, _) in content_query.iter() {
            let distance = active_pos.distance(content_transform.translation);
            if distance > cleanup_radius {
                commands.entity(entity).despawn();
            }
        }
        
        // Phase 2: Collect existing content for collision avoidance
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
            
        // Add existing vehicles (non-dynamic) to the collision avoidance list with larger radius
        for vehicle_transform in existing_vehicles_query.iter() {
            existing_content.push((vehicle_transform.translation, ContentType::Vehicle, 25.0));
        }
        
        // Phase 3: TRUE CIRCULAR SPAWNING using polar coordinates
        // Generate content in concentric circles around the active entity
        let mut spawn_attempts = 0;
        let max_spawn_attempts = 15; // REDUCED: From 50 to 15 for better performance
        
        for radius_step in (spawn_density as i32..active_radius as i32).step_by(spawn_density as usize) {
            let radius = radius_step as f32;
            let circumference = 2.0 * std::f32::consts::PI * radius;
            let points_on_circle = (circumference / spawn_density).max(8.0) as i32;
            
            for i in 0..points_on_circle {
                spawn_attempts += 1;
                if spawn_attempts > max_spawn_attempts { break; }
                
                // PERFORMANCE: Check frame time budget
                if frame_start_time.elapsed().as_millis() > 3 {
                    break; // Exit early to maintain frame rate
                }
                
                let angle = (i as f32 / points_on_circle as f32) * 2.0 * std::f32::consts::PI;
                let spawn_x = active_pos.x + radius * angle.cos();
                let spawn_z = active_pos.z + radius * angle.sin();
                let spawn_pos = Vec3::new(spawn_x, 0.0, spawn_z);
                
                // Only spawn if no content exists nearby
                if !has_content_at_position(spawn_pos, &existing_content, spawn_density * 0.8) {
                    request_spawn_validation(spawn_pos, &mut validation_tracker, &mut validation_writer, &mut rng);
                }
            }
            if spawn_attempts > max_spawn_attempts { break; }
            
            // PERFORMANCE: Check frame time budget between radius loops
            if frame_start_time.elapsed().as_millis() > 3 {
                break; // Exit early to maintain frame rate
            }
        }
    }
}

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




