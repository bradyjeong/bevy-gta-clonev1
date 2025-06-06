use bevy::prelude::*;

#[derive(Component)]
pub struct NPC {
    pub target_position: Vec3,
    pub speed: f32,
    pub last_update: f32,
    pub update_interval: f32,
}

#[derive(Component)]
pub struct Cullable {
    pub max_distance: f32,
    pub is_culled: bool,
}

impl Cullable {
    pub fn new(max_distance: f32) -> Self {
        Self {
            max_distance,
            is_culled: false,
        }
    }
}

// Road system components
#[derive(Component)]
pub struct RoadEntity {
    pub road_id: u32,
}

#[derive(Component)]
pub struct IntersectionEntity {
    pub intersection_id: u32,
}

#[derive(Component)]
pub struct DynamicTerrain;

#[derive(Component)]
pub struct DynamicContent {
    pub content_type: ContentType,
}

#[derive(Clone, PartialEq)]
pub enum ContentType {
    Road,
    Building,
    Tree,
    Vehicle,
    NPC,
}

#[derive(Component)]
pub struct PerformanceCritical;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Landmark;

#[derive(Component)]
pub struct Buildable;

#[derive(Component)]
pub struct SunLight;

#[derive(Component)]
pub struct SkyDome;

#[derive(Component)]
pub struct Clouds;

#[derive(Component)]
pub struct MainCamera;

// Resources
#[derive(Resource)]
pub struct CullingSettings {
    pub _npc_cull_distance: f32,
    pub _car_cull_distance: f32,
    pub _building_cull_distance: f32,
    pub _tree_cull_distance: f32,
}

impl Default for CullingSettings {
    fn default() -> Self {
        Self {
            _npc_cull_distance: 200.0,
            _car_cull_distance: 300.0,
            _building_cull_distance: 800.0,
            _tree_cull_distance: 400.0,
        }
    }
}

#[derive(Resource)]
pub struct PerformanceStats {
    pub entity_count: usize,
    pub culled_entities: usize,
    pub frame_time: f32,
    pub last_report: f32,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            entity_count: 0,
            culled_entities: 0,
            frame_time: 0.0,
            last_report: 0.0,
        }
    }
}
