//! Ground detection coordination events
//! 
//! These events replace direct ground detection service calls with proper event-driven
//! communication, maintaining plugin boundaries while providing ground height data.

use bevy::prelude::*;

/// Request ID for matching ground detection requests with responses (4 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GroundRequestId(pub u32);

impl GroundRequestId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Request ground height at a specific position (20 bytes)
/// Sent by: spawn systems, setup systems, NPC systems
/// Handled by: ground detection service
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestGroundHeight {
    pub id: GroundRequestId,
    pub position: Vec3,
    pub entity_height: f32,
}

impl RequestGroundHeight {
    pub fn new(id: GroundRequestId, position: Vec3, entity_height: f32) -> Self {
        Self { id, position, entity_height }
    }
    
    pub fn simple(id: GroundRequestId, position: Vec3) -> Self {
        Self::new(id, position, 0.0)
    }
}

/// Ground height result with validation (24 bytes)
/// Sent by: ground detection service
/// Handled by: spawn systems, setup systems, NPC systems
#[derive(Event, Debug, Clone, Copy)]
pub struct GroundHeightResult {
    pub id: GroundRequestId,
    pub position: Vec3,
    pub ground_height: f32,
    pub valid: bool,
    pub surface_type: SurfaceType,
}

impl GroundHeightResult {
    pub fn new(
        id: GroundRequestId,
        position: Vec3,
        ground_height: f32,
        valid: bool,
        surface_type: SurfaceType,
    ) -> Self {
        Self {
            id,
            position,
            ground_height,
            valid,
            surface_type,
        }
    }
    
    pub fn valid_ground(id: GroundRequestId, position: Vec3, ground_height: f32) -> Self {
        Self::new(id, position, ground_height, true, SurfaceType::Ground)
    }
    
    pub fn water_surface(id: GroundRequestId, position: Vec3, water_height: f32) -> Self {
        Self::new(id, position, water_height, true, SurfaceType::Water)
    }
    
    pub fn invalid(id: GroundRequestId, position: Vec3) -> Self {
        Self::new(id, position, 0.0, false, SurfaceType::Unknown)
    }
}

/// Request spawn position validation (20 bytes)
/// Sent by: spawn systems, factories
/// Handled by: ground detection service
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestSpawnPositionValidation {
    pub id: GroundRequestId,
    pub position: Vec3,
    pub entity_height: f32,
}

impl RequestSpawnPositionValidation {
    pub fn new(id: GroundRequestId, position: Vec3, entity_height: f32) -> Self {
        Self { id, position, entity_height }
    }
}

/// Spawn position validation result (21 bytes)
/// Sent by: ground detection service
/// Handled by: spawn systems, factories
#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnPositionValidationResult {
    pub id: GroundRequestId,
    pub position: Vec3,
    pub adjusted_position: Vec3,
    pub valid: bool,
}

impl SpawnPositionValidationResult {
    pub fn new(
        id: GroundRequestId,
        position: Vec3,
        adjusted_position: Vec3,
        valid: bool,
    ) -> Self {
        Self {
            id,
            position,
            adjusted_position,
            valid,
        }
    }
    
    pub fn valid_spawn(id: GroundRequestId, position: Vec3, adjusted_position: Vec3) -> Self {
        Self::new(id, position, adjusted_position, true)
    }
    
    pub fn invalid_spawn(id: GroundRequestId, position: Vec3) -> Self {
        Self::new(id, position, position, false)
    }
}

/// Surface type for ground detection (1 byte)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SurfaceType {
    Ground = 0,
    Water = 1,
    Road = 2,
    Building = 3,
    Vegetation = 4,
    Unknown = 255,
}

impl Default for SurfaceType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl SurfaceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ground => "ground",
            Self::Water => "water",
            Self::Road => "road",
            Self::Building => "building",
            Self::Vegetation => "vegetation",
            Self::Unknown => "unknown",
        }
    }
    
    pub fn is_solid(&self) -> bool {
        matches!(self, Self::Ground | Self::Road | Self::Building)
    }
    
    pub fn allows_spawn(&self) -> bool {
        matches!(self, Self::Ground | Self::Vegetation)
    }
}

// Compile-time size verification (≤128 bytes requirement)
const _: () = {
    assert!(std::mem::size_of::<RequestGroundHeight>() <= 128);
    assert!(std::mem::size_of::<GroundHeightResult>() <= 128);
    assert!(std::mem::size_of::<RequestSpawnPositionValidation>() <= 128);
    assert!(std::mem::size_of::<SpawnPositionValidationResult>() <= 128);
};
