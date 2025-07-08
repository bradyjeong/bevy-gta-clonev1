//! ───────────────────────────────────────────────
//! System:   Dynamic Content
//! Purpose:  Handles entity movement and physics
//! Schedule: Update (throttled)
//! Reads:    `ActiveEntity`, `EntityLimits`, Transform, Car, `GameConfig`
//! Writes:   `UnifiedEntityFactory`, `EntityLimits`, Transform, Velocity
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Only active entities can be controlled
//!   * Timing intervals are respected
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::cell::RefCell;
use game_core::prelude::*;
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::road_generation::is_on_road_spline;
use crate::config::GameConfig;


thread_local! {
    static CONTENT_RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

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

pub fn dynamic_content_system(
    mut commands: Commands,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicContent>)>,
    content_query: Query<(Entity, &Transform, &DynamicContent)>,
    existing_vehicles_query: Query<&Transform, (With<Car>, Without<DynamicContent>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _entity_limits: ResMut<EntityLimits>,
    mut unified_factory: ResMut<UnifiedEntityFactory>,
    road_network: Res<RoadNetwork>,
    time: Res<Time>,
    mut timer: Local<DynamicContentTimer>,
    game_config: Res<GameConfig>,
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
            .is_none_or(|last_pos| active_pos.distance(last_pos) > movement_threshold);
        
        let should_update = timer.timer >= 8.0 || player_moved;
        
        if !should_update {
            return;
        }
        
        timer.timer = 0.0;
        timer.last_player_pos = Some(active_pos);
        
        // Data-driven performance settings
        let active_radius = game_config.world.active_radius;
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
                (transform.translation, dynamic_content.content_type, radius)
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
                    spawn_dynamic_content_safe_unified(&mut commands, spawn_pos, &existing_content, &mut meshes, &mut materials, &mut unified_factory, &road_network, time.elapsed_secs(), &game_config);
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



fn is_in_water_area(position: Vec3) -> bool {
    // Lake position and size (must match water.rs setup)
    let lake_center = Vec3::new(300.0, -2.0, 300.0);
    let lake_size = 200.0;
    let buffer = 20.0; // Extra buffer around lake
    
    let distance = Vec2::new(
        position.x - lake_center.x,
        position.z - lake_center.z,
    ).length();
    
    distance < (lake_size / 2.0 + buffer)
}



pub fn vehicle_separation_system(
    mut vehicle_query: Query<(&mut Transform, &mut Velocity), (With<Car>, With<DynamicContent>)>,
) {
    let vehicles: Vec<(Vec3, Entity)> = vehicle_query.iter()
        .enumerate()
        .map(|(i, (transform, _))| (transform.translation, Entity::from_raw(i as u32)))
        .collect();
    
    for (mut transform, mut velocity) in &mut vehicle_query {
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

/// NEW UNIFIED SPAWN FUNCTION - Phase 2.1
/// This replaces `spawn_dynamic_content_safe` using the unified factory
fn spawn_dynamic_content_safe_unified(
    commands: &mut Commands,
    position: Vec3,
    existing_content: &[(Vec3, ContentType, f32)],
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    unified_factory: &mut ResMut<UnifiedEntityFactory>,
    road_network: &RoadNetwork,
    current_time: f32,
    game_config: &GameConfig,
) {
    // Data-driven spawn rates from game config
    let on_road = is_on_road_spline(position, road_network, 25.0);
    
    // Buildings - configurable spawn rate, not on roads
    if CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.0..1.0)) < game_config.spawn_rates.buildings {
        if !on_road && !is_in_water_area(position) {
            if let Ok(Some(_entity)) = unified_factory.spawn_entity_consolidated(
                commands,
                meshes,
                materials,
                ContentType::Building,
                position,
                Some(road_network),
                existing_content,
                current_time,
            ) {
                println!("DEBUG: Spawned building using unified factory at {position:?}");
            }
        }
    }
    
    // Vehicles - configurable spawn rate, only on roads  
    else if on_road && CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.0..1.0)) < game_config.spawn_rates.vehicles {
        if let Ok(Some(_entity)) = unified_factory.spawn_entity_consolidated(
            commands,
            meshes,
            materials,
            ContentType::Vehicle,
            position,
            Some(road_network),
            existing_content,
            current_time,
        ) {
            println!("DEBUG: Spawned vehicle using unified factory at {position:?}");
        }
    }
    
    // Trees - configurable spawn rate, not on roads, not in water
    else if !on_road && !is_in_water_area(position) && CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.0..1.0)) < game_config.spawn_rates.trees {
        if let Ok(Some(_entity)) = unified_factory.spawn_entity_consolidated(
            commands,
            meshes,
            materials,
            ContentType::Tree,
            position,
            Some(road_network),
            existing_content,
            current_time,
        ) {
            println!("DEBUG: Spawned tree using unified factory at {position:?}");
        }
    }
    
    // NPCs - configurable spawn rate, anywhere
    else if CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.0..1.0)) < game_config.spawn_rates.npcs {
        if let Ok(Some(_entity)) = unified_factory.spawn_entity_consolidated(
            commands,
            meshes,
            materials,
            ContentType::NPC,
            position,
            Some(road_network),
            existing_content,
            current_time,
        ) {
            println!("DEBUG: Spawned NPC using unified factory at {position:?}");
        }
    }
}
