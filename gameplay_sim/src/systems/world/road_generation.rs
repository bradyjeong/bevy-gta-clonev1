//! ───────────────────────────────────────────────
//! System:   Road Generation
//! Purpose:  Procedurally generates roads based on player position
//! Schedule: Update (throttled)
//! Reads:    ActiveEntity, Transform, RoadEntity
//! Writes:   RoadNetwork, Commands
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashSet;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use game_core::prelude::*;
use crate::systems::world::road_network::{RoadNetwork, RoadSpline, RoadType};
use game_core::bundles::VisibleBundle;

const CHUNK_SIZE: f32 = 200.0;

#[derive(Resource, Default)]
pub struct RoadGenerationCache {
    pub generated_chunks: HashSet<(i32, i32)>,
    pub last_player_chunk: Option<(i32, i32)>,
}

/// Trait for extending RoadNetwork functionality
pub trait RoadNetworkExtensions {
    fn clear_cache(&mut self);
    fn generate_chunk_roads(&mut self, chunk_x: i32, chunk_z: i32) -> Vec<u32>;
}

impl RoadNetworkExtensions for RoadNetwork {
    fn clear_cache(&mut self) {
        self.roads.clear();
        self.intersections.clear();
        self.next_road_id = 1;
        self.next_intersection_id = 1;
    }
    
    fn generate_chunk_roads(&mut self, chunk_x: i32, chunk_z: i32) -> Vec<u32> {
        let mut road_ids = Vec::new();
        
        // Generate deterministic roads for this chunk
        let mut rng = StdRng::seed_from_u64(
            ((chunk_x as u64) << 32) | ((chunk_z as u64) & 0xFFFFFFFF)
        );
        
        let chunk_center = Vec3::new(
            chunk_x as f32 * CHUNK_SIZE,
            0.0,
            chunk_z as f32 * CHUNK_SIZE,
        );
        
        // Main roads (horizontal and vertical through chunk)
        if rng.gen_bool(0.7) {
            let start_point = chunk_center + Vec3::new(-CHUNK_SIZE * 0.5, 0.0, 0.0);
            let end_point = chunk_center + Vec3::new(CHUNK_SIZE * 0.5, 0.0, 0.0);
            
            let road = RoadSpline {
                points: vec![start_point, end_point],
                road_type: RoadType::MainStreet,
                width: 8.0,
                connections: Vec::new(),
            };
            
            self.roads.insert(self.next_road_id, road);
            road_ids.push(self.next_road_id);
            self.next_road_id += 1;
        }
        
        if rng.gen_bool(0.7) {
            let start_point = chunk_center + Vec3::new(0.0, 0.0, -CHUNK_SIZE * 0.5);
            let end_point = chunk_center + Vec3::new(0.0, 0.0, CHUNK_SIZE * 0.5);
            
            let road = RoadSpline {
                points: vec![start_point, end_point],
                road_type: RoadType::MainStreet,
                width: 8.0,
                connections: Vec::new(),
            };
            
            self.roads.insert(self.next_road_id, road);
            road_ids.push(self.next_road_id);
            self.next_road_id += 1;
        }
        
        // Secondary roads
        for _ in 0..rng.gen_range(1..4) {
            let start = chunk_center + Vec3::new(
                rng.gen_range(-CHUNK_SIZE * 0.4..CHUNK_SIZE * 0.4),
                0.0,
                rng.gen_range(-CHUNK_SIZE * 0.4..CHUNK_SIZE * 0.4),
            );
            
            let end = start + Vec3::new(
                rng.gen_range(-50.0..50.0),
                0.0,
                rng.gen_range(-50.0..50.0),
            );
            
            let road_type = if rng.gen_bool(0.3) {
                RoadType::SideStreet
            } else {
                RoadType::Alley
            };
            
            let road = RoadSpline {
                points: vec![start, end],
                road_type,
                width: 6.0,
                connections: Vec::new(),
            };
            
            self.roads.insert(self.next_road_id, road);
            road_ids.push(self.next_road_id);
            self.next_road_id += 1;
        }
        
        road_ids
    }
}

pub fn road_generation_system(
    mut commands: Commands,
    active_query: Query<&Transform, With<ActiveEntity>>,
    road_query: Query<(Entity, &Transform), With<RoadEntity>>,
    mut road_network: ResMut<RoadNetwork>,
    mut cache: ResMut<RoadGenerationCache>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        let chunk_size = CHUNK_SIZE;
        
        // Calculate current chunk
        let current_chunk = (
            (active_pos.x / chunk_size).floor() as i32,
            (active_pos.z / chunk_size).floor() as i32,
        );
        
        // Check if we've moved to a new chunk
        let chunk_changed = cache.last_player_chunk != Some(current_chunk);
        if chunk_changed {
            cache.last_player_chunk = Some(current_chunk);
        }
        
        // Clear cache if no roads exist but cache exists (optimization)
        if road_network.roads.is_empty() && !cache.generated_chunks.is_empty() {
            cache.generated_chunks.clear();
        }
        
        let active_radius = 800.0;   // Increased for better road coverage
        let cleanup_radius = 2000.0;  // Very large cleanup radius to prevent premature despawning
        
        // Clean up distant road entities (very conservative cleanup)
        if chunk_changed {
            for (entity, transform) in road_query.iter() {
                // Simple distance check - only remove roads that are extremely far away
                let distance = active_pos.distance(transform.translation);
                if distance > cleanup_radius {
                    println!("DEBUG: Cleaning up road entity at distance {}", distance);
                    commands.entity(entity).despawn();
                }
            }
        }
        
        // Determine which chunks need roads
        let (chunk_x, chunk_z) = current_chunk;
        let chunk_radius = ((active_radius as f32 / chunk_size).ceil() as i32).max(3); // Ensure at least 3x3 chunk coverage
        
        // Generate roads for nearby chunks
        for dx in -chunk_radius..=chunk_radius {
            for dz in -chunk_radius..=chunk_radius {
                let check_chunk_x = chunk_x + dx;
                let check_chunk_z = chunk_z + dz;
                
                let chunk_center = Vec3::new(
                    check_chunk_x as f32 * chunk_size,
                    0.0,  // At ground level
                    check_chunk_z as f32 * chunk_size
                );
                let distance = active_pos.distance(chunk_center);
                if distance <= active_radius {
                    // Check if chunk already generated
                    if !cache.generated_chunks.contains(&(check_chunk_x, check_chunk_z)) {
                        cache.generated_chunks.insert((check_chunk_x, check_chunk_z));
                        
                        // Generate roads for this chunk
                        generate_roads_for_chunk(&mut commands, check_chunk_x, check_chunk_z, &mut road_network);
                    }
                }
            }
        }
    }
}

fn generate_roads_for_chunk(
    commands: &mut Commands,
    chunk_x: i32,
    chunk_z: i32,
    road_network: &mut RoadNetwork,
) {
    let mut rng = StdRng::seed_from_u64(
        ((chunk_x as u64) << 32) | ((chunk_z as u64) & 0xFFFFFFFF)
    );
    
    let chunk_center = Vec3::new(
        chunk_x as f32 * CHUNK_SIZE,
        0.0,
        chunk_z as f32 * CHUNK_SIZE,
    );
    
    // Main roads (horizontal and vertical through chunk)
    if rng.gen_bool(0.7) {
        let start = chunk_center + Vec3::new(-CHUNK_SIZE * 0.5, 0.0, 0.0);
        let end = chunk_center + Vec3::new(CHUNK_SIZE * 0.5, 0.0, 0.0);
        let road_id = road_network.add_road(start, end, RoadType::MainStreet);
        
        // Spawn road entity
        spawn_road_entity(commands, road_id, start, end);
    }
    
    if rng.gen_bool(0.7) {
        let start = chunk_center + Vec3::new(0.0, 0.0, -CHUNK_SIZE * 0.5);
        let end = chunk_center + Vec3::new(0.0, 0.0, CHUNK_SIZE * 0.5);
        let road_id = road_network.add_road(start, end, RoadType::MainStreet);
        
        // Spawn road entity
        spawn_road_entity(commands, road_id, start, end);
    }
    
    // Secondary roads
    for _ in 0..rng.gen_range(1..4) {
        let start = chunk_center + Vec3::new(
            rng.gen_range(-CHUNK_SIZE * 0.4..CHUNK_SIZE * 0.4),
            0.0,
            rng.gen_range(-CHUNK_SIZE * 0.4..CHUNK_SIZE * 0.4),
        );
        
        let end = start + Vec3::new(
            rng.gen_range(-50.0..50.0),
            0.0,
            rng.gen_range(-50.0..50.0),
        );
        
        let road_type = if rng.gen_bool(0.3) {
            RoadType::SideStreet
        } else {
            RoadType::Alley
        };
        
        let road_id = road_network.add_road(start, end, road_type);
        spawn_road_entity(commands, road_id, start, end);
    }
}

fn spawn_road_entity(commands: &mut Commands, road_id: u32, start: Vec3, end: Vec3) {
    let center = (start + end) * 0.5;
    
    commands.spawn((
        RoadEntity { road_id },
        Transform::from_translation(center),
        VisibleBundle::default(),
    ));
}

pub fn is_on_road_spline(position: Vec3, road_network: &RoadNetwork, tolerance: f32) -> bool {
    road_network.get_road_at_position(position, tolerance).is_some()
}

pub fn find_nearest_road_position(position: Vec3, road_network: &RoadNetwork) -> Vec3 {
    if let Some((_, _, nearest_pos)) = road_network.find_nearest_road(position) {
        nearest_pos
    } else {
        position // Return original position if no roads found
    }
}
