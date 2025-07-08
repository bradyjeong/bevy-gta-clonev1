// TEMP_PHASE_6_BRIDGE - Temporary compatibility layer for Phase 6a

// Re-export Bevy types that are missing
pub use bevy::prelude::{
    // Bundle types that exist in Bevy 0.16
    Transform, Visibility, InheritedVisibility, ViewVisibility, GlobalTransform,
    // Query types
    Query,
    // Other common types
    StandardMaterial, Mesh, Handle, Assets, Commands, Entity, Component, Resource, Bundle,
};

// Define SpatialBundle temporarily since it's not available in Bevy 0.16 prelude
#[derive(Bundle, Clone, Debug, Default)]
pub struct SpatialBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl SpatialBundle {
    pub fn from_transform(transform: Transform) -> Self {
        Self {
            transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

// Temporary compatibility types for Phase 6
pub type TransformBundle = SpatialBundle;
pub type VisibilityBundle = SpatialBundle;
// In Bevy 0.16, use MeshMaterial3d and Mesh3d
use bevy::prelude::{Mesh3d, MeshMaterial3d};

#[derive(Bundle)]
pub struct MaterialMeshBundle<M: bevy::prelude::Material> {
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<M>,
    pub transform: Transform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub global_transform: GlobalTransform,
}

// Re-export components from game_core
pub use game_core::prelude::*;

// Temporary re-exports of collision groups to fix ambiguity
pub use game_core::constants::{
    STATIC_GROUP as GAME_CORE_STATIC_GROUP,
    VEHICLE_GROUP as GAME_CORE_VEHICLE_GROUP,
    CHARACTER_GROUP as GAME_CORE_CHARACTER_GROUP,
};

// Define missing types temporarily
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    Oak,
    Pine,
    Birch,
}

impl Default for TreeType {
    fn default() -> Self {
        Self::Oak
    }
}

// Define missing enum for NPC behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NPCBehaviorState {
    Idle,
    Walking,
    Running,
    Driving,
}

impl Default for NPCBehaviorState {
    fn default() -> Self {
        Self::Idle
    }
}

// Define missing bundle error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    NotImplemented,
    InvalidConfiguration,
    MissingComponent,
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::NotImplemented => write!(f, "Bundle type not implemented"),
            BundleError::InvalidConfiguration => write!(f, "Invalid bundle configuration"),
            BundleError::MissingComponent => write!(f, "Missing required component"),
        }
    }
}

impl std::error::Error for BundleError {}

// Define missing performance types temporarily  
#[derive(Debug, Clone, Default, Resource)]
pub struct PerformanceCounters {
    pub fps: f32,
    pub frame_time: f32,
    pub entity_count: usize,
    pub lod_updates: usize,
    pub culled_entities: usize,
}

impl PerformanceCounters {
    pub fn update_frame(&mut self, delta_time: f32) {
        self.frame_time = delta_time;
        self.fps = if delta_time > 0.0 { 1.0 / delta_time } else { 0.0 };
    }
    
    pub fn reset_per_frame_counters(&mut self) {
        self.lod_updates = 0;
        self.culled_entities = 0;
    }
}

// Define missing load state
#[derive(Debug, Clone, Default)]
pub struct LoadState {
    pub loading: bool,
    pub progress: f32,
}

// Define missing component types
#[derive(Debug, Clone, Default, bevy::prelude::Component)]
pub struct Tree {
    pub tree_type: TreeType,
    pub height: f32,
    pub age: f32,
}

#[derive(Debug, Clone, Default, bevy::prelude::Component)]
pub struct NPC {
    pub target_position: Vec3,
    pub speed: f32,
    pub last_update: f32,
    pub update_interval: f32,
    pub health: f32,
    pub max_health: f32,
    pub behavior_state: NPCBehaviorState,
    pub spawn_time: f32,
}

// Define missing enum for road types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoadType {
    Highway,
    MainStreet, 
    SideStreet,
    Alley,
}

impl Default for RoadType {
    fn default() -> Self {
        Self::SideStreet
    }
}

// Define missing road network type
// Re-export canonical types from game_core
pub use game_core::world::RoadNetwork;

#[derive(Debug, Clone)]
pub struct RoadSpline {
    pub points: Vec<Vec3>,
    pub road_type: RoadType,
    pub width: f32,
}

impl Default for RoadSpline {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            road_type: RoadType::default(),
            width: 4.0,
        }
    }
}

// Define missing chunk state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkState {
    Loading,
    Loaded { entity_count: usize },
    Unloaded,
}

impl Default for ChunkState {
    fn default() -> Self {
        Self::Unloaded
    }
}

// Define missing unified performance tracker
pub type UnifiedPerformanceTracker = engine_bevy::services::performance_service::UnifiedPerformanceTracker;

// Temporary function stubs
pub fn save_game_system() {}
pub fn load_game_system() {}
pub fn water_wave_system() {}
pub fn yacht_buoyancy_system() {}
pub fn yacht_water_constraint_system() {}

// Re-export Vec3 for convenience
pub use bevy::prelude::Vec3;
