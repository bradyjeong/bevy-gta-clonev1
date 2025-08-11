//! Spawn validation event handler V2 - Using decomposed resources
//! 
//! Handles RequestSpawnValidation events using the new PlacementGrid and RoadNetwork resources

use bevy::prelude::*;
use crate::events::world::validation_events::{
    RequestSpawnValidation, SpawnValidationResult, ValidationReason,
    RequestRoadValidation, RoadValidationResult,
};
use crate::events::world::content_events::ContentType;
use crate::world::{PlacementGrid, RoadNetwork};

use std::collections::HashMap;

/// Track pending validation requests
#[derive(Default)]
pub struct ValidationTracker {
    pending_requests: HashMap<u32, RequestSpawnValidation>,
    next_id: u32,
}

impl ValidationTracker {
    pub fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }
}

/// V2 Handle spawn validation requests by coordinating with road validation
#[cfg(feature = "world_v2")]
pub fn handle_spawn_validation_request_v2(
    mut validation_reader: EventReader<RequestSpawnValidation>,
    mut road_validation_writer: EventWriter<RequestRoadValidation>,
    mut tracker: Local<ValidationTracker>,
) {
    for request in validation_reader.read() {
        // Store the original request for later response
        tracker.pending_requests.insert(request.id.0, *request);
        
        // Request road validation from roads plugin
        road_validation_writer.write(RequestRoadValidation::new(request.id, request.pos));
    }
}

/// V2 Handle road validation results and emit final spawn validation results
#[cfg(feature = "world_v2")]
pub fn handle_road_validation_result_v2(
    mut road_validation_reader: EventReader<RoadValidationResult>,
    mut spawn_validation_writer: EventWriter<SpawnValidationResult>,
    mut tracker: Local<ValidationTracker>,
    placement_grid: Res<PlacementGrid>,
) {
    for road_result in road_validation_reader.read() {
        if let Some(original_request) = tracker.pending_requests.remove(&road_result.id.0) {
            let validation_result = determine_spawn_validity_v2(
                original_request,
                *road_result,
                &placement_grid,
            );
            spawn_validation_writer.write(validation_result);
        }
    }
}

/// V2 Handle road validation requests using RoadNetwork resource
#[cfg(feature = "world_v2")]
pub fn handle_road_validation_request_v2(
    mut road_request_reader: EventReader<RequestRoadValidation>,
    mut road_result_writer: EventWriter<RoadValidationResult>,
    road_network: Res<RoadNetwork>,
) {
    for request in road_request_reader.read() {
        // Use RoadNetwork methods directly
        let on_road = road_network.is_near_road(request.pos, 25.0);
        let distance_to_road = if on_road { 
            0.0 
        } else {
            road_network.get_nearest_road_point(request.pos)
                .map(|nearest| request.pos.distance(nearest))
                .unwrap_or(f32::MAX)
        };
        
        road_result_writer.write(RoadValidationResult::new(
            request.id,
            request.pos,
            on_road,
            distance_to_road,
        ));
    }
}

/// V2 Determine if spawn is valid based on content type, road validation, and placement grid
fn determine_spawn_validity_v2(
    request: RequestSpawnValidation,
    road_result: RoadValidationResult,
    placement_grid: &PlacementGrid,
) -> SpawnValidationResult {
    let on_road = road_result.on_road;
    
    // First check placement grid for collisions
    let radius = get_content_radius(request.content_type);
    if placement_grid.check_collision(request.pos, radius) {
        return SpawnValidationResult::invalid(
            request.id,
            request.pos,
            request.content_type,
            ValidationReason::Collision,
        );
    }
    
    // Apply content-specific validation rules
    match request.content_type {
        ContentType::Building => {
            if on_road {
                SpawnValidationResult::invalid(request.id, request.pos, request.content_type, ValidationReason::OnRoad)
            } else if is_in_water_area(request.pos) {
                SpawnValidationResult::invalid(request.id, request.pos, request.content_type, ValidationReason::InWater)
            } else {
                SpawnValidationResult::valid(request.id, request.pos, request.content_type)
            }
        }
        ContentType::Vehicle => {
            if !on_road {
                SpawnValidationResult::invalid(request.id, request.pos, request.content_type, ValidationReason::OutOfBounds)
            } else {
                SpawnValidationResult::valid(request.id, request.pos, request.content_type)
            }
        }
        ContentType::Tree => {
            if on_road {
                SpawnValidationResult::invalid(request.id, request.pos, request.content_type, ValidationReason::OnRoad)
            } else if is_in_water_area(request.pos) {
                SpawnValidationResult::invalid(request.id, request.pos, request.content_type, ValidationReason::InWater)
            } else {
                SpawnValidationResult::valid(request.id, request.pos, request.content_type)
            }
        }
        ContentType::NPC => {
            // NPCs can spawn anywhere except water
            if is_in_water_area(request.pos) {
                SpawnValidationResult::invalid(request.id, request.pos, request.content_type, ValidationReason::InWater)
            } else {
                SpawnValidationResult::valid(request.id, request.pos, request.content_type)
            }
        }
        ContentType::Road => {
            // Roads managed by separate system, always valid for this check
            SpawnValidationResult::valid(request.id, request.pos, request.content_type)
        }
    }
}

/// Get default radius for content type
fn get_content_radius(content_type: ContentType) -> f32 {
    match content_type {
        ContentType::Building => 15.0,
        ContentType::Vehicle => 3.0,
        ContentType::Tree => 2.0,
        ContentType::NPC => 1.0,
        ContentType::Road => 10.0,
    }
}

/// Check if position is in water area (replaces direct water area checks)
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
