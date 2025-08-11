//! Spawn validation events - replaces direct is_on_road_spline calls
//! 
//! These events decouple spawn validation logic from content placement,
//! allowing the roads plugin to provide validation services without direct coupling.

use bevy::prelude::*;
use super::content_events::ContentType;

/// Request ID for matching validation requests with responses (4 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ValidationId(pub u32);

impl ValidationId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Request spawn validation for a specific position and content type (20 bytes)
/// Sent by: dynamic content system, entity factories
/// Handled by: validation systems (roads, terrain, buildings)
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestSpawnValidation {
    pub id: ValidationId,
    pub pos: Vec3,
    pub content_type: ContentType,
}

impl RequestSpawnValidation {
    pub fn new(id: ValidationId, pos: Vec3, content_type: ContentType) -> Self {
        Self { id, pos, content_type }
    }
}

/// Validation reason enum to replace String for better performance (1 byte)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ValidationReason {
    Valid = 0,
    OnRoad = 1,
    TooClose = 2,
    OutOfBounds = 3,
    InWater = 4,
    BlockedByTerrain = 5,
}

impl ValidationReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::OnRoad => "on_road",
            Self::TooClose => "too_close",
            Self::OutOfBounds => "out_of_bounds",
            Self::InWater => "in_water",
            Self::BlockedByTerrain => "blocked_by_terrain",
        }
    }
}

/// Validation result with reason for invalid positions (25 bytes)
/// Sent by: validation systems (roads, terrain, buildings)
/// Handled by: content spawning system, factories
#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnValidationResult {
    pub id: ValidationId,
    pub position: Vec3,
    pub content_type: ContentType,
    pub valid: bool,
    pub reason: ValidationReason,
}

impl SpawnValidationResult {
    pub fn new(id: ValidationId, position: Vec3, content_type: ContentType, valid: bool, reason: ValidationReason) -> Self {
        Self {
            id,
            position,
            content_type,
            valid,
            reason,
        }
    }
    
    pub fn valid(id: ValidationId, position: Vec3, content_type: ContentType) -> Self {
        Self::new(id, position, content_type, true, ValidationReason::Valid)
    }
    
    pub fn invalid(id: ValidationId, position: Vec3, content_type: ContentType, reason: ValidationReason) -> Self {
        Self::new(id, position, content_type, false, reason)
    }
    
    pub fn on_road(id: ValidationId, position: Vec3, content_type: ContentType) -> Self {
        Self::invalid(id, position, content_type, ValidationReason::OnRoad)
    }
    
    pub fn too_close(id: ValidationId, position: Vec3, content_type: ContentType) -> Self {
        Self::invalid(id, position, content_type, ValidationReason::TooClose)
    }
    
    pub fn out_of_bounds(id: ValidationId, position: Vec3, content_type: ContentType) -> Self {
        Self::invalid(id, position, content_type, ValidationReason::OutOfBounds)
    }
}

/// Request road spline validation specifically (replaces is_on_road_spline calls) (16 bytes)
/// Sent by: validation coordinator
/// Handled by: roads plugin validation system
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestRoadValidation {
    pub id: ValidationId,
    pub pos: Vec3,
}

impl RequestRoadValidation {
    pub fn new(id: ValidationId, pos: Vec3) -> Self {
        Self { id, pos }
    }
}

/// Road validation result with distance information (20 bytes)
/// Sent by: roads plugin validation system
/// Handled by: validation coordinator
#[derive(Event, Debug, Clone, Copy)]
pub struct RoadValidationResult {
    pub id: ValidationId,
    pub pos: Vec3,
    pub on_road: bool,
    pub distance_to_road: f32,
}

impl RoadValidationResult {
    pub fn new(id: ValidationId, pos: Vec3, on_road: bool, distance_to_road: f32) -> Self {
        Self {
            id,
            pos,
            on_road,
            distance_to_road,
        }
    }
}

// Compile-time size verification (≤128 bytes requirement)
const _: () = {
    assert!(std::mem::size_of::<RequestSpawnValidation>() <= 128);
    assert!(std::mem::size_of::<SpawnValidationResult>() <= 128);
    assert!(std::mem::size_of::<RequestRoadValidation>() <= 128);
    assert!(std::mem::size_of::<RoadValidationResult>() <= 128);
};
