//! Spawn validation event handler
//! 
//! Handles RequestSpawnValidation events by checking road splines and other constraints,
//! then emits SpawnValidationResult events. This replaces direct is_on_road_spline calls.

use bevy::prelude::*;
use crate::events::world::validation_events::{
    RequestSpawnValidation, SpawnValidationResult, ValidationReason,
    RequestRoadValidation, RoadValidationResult,
};
use crate::events::world::content_events::ContentType;
use crate::world::RoadNetwork;
use crate::systems::world::road_generation::is_on_road_spline;

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

/// Handle spawn validation requests by coordinating with road validation
/// Named: handle_spawn_validation_request (per Oracle requirements)
pub fn handle_spawn_validation_request(
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

/// Handle road validation results and emit final spawn validation results
/// Named: handle_road_validation_result (per Oracle requirements)
pub fn handle_road_validation_result(
    mut road_validation_reader: EventReader<RoadValidationResult>,
    mut spawn_validation_writer: EventWriter<SpawnValidationResult>,
    mut tracker: Local<ValidationTracker>,
) {
    for road_result in road_validation_reader.read() {
        if let Some(original_request) = tracker.pending_requests.remove(&road_result.id.0) {
            let validation_result = determine_spawn_validity(original_request, *road_result);
            spawn_validation_writer.write(validation_result);
        }
    }
}

/// Handle road validation requests from roads plugin
/// Named: handle_road_validation_request (per Oracle requirements)
pub fn handle_road_validation_request(
    mut road_request_reader: EventReader<RequestRoadValidation>,
    mut road_result_writer: EventWriter<RoadValidationResult>,
    road_network: Res<RoadNetwork>,
) {
    for request in road_request_reader.read() {
        let on_road = is_on_road_spline(request.pos, &*road_network, 25.0);
        let distance_to_road = if on_road { 0.0 } else { 25.0 }; // Simplified distance
        
        road_result_writer.write(RoadValidationResult::new(
            request.id,
            request.pos,
            on_road,
            distance_to_road,
        ));
    }
}

/// Determine if spawn is valid based on content type and road validation
fn determine_spawn_validity(
    request: RequestSpawnValidation,
    road_result: RoadValidationResult,
) -> SpawnValidationResult {
    let on_road = road_result.on_road;
    
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
