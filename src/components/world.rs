use bevy::prelude::*;



/// NPC Behavior Component - replaces old NPCBehavior
#[derive(Component, Debug, Clone)]
pub struct NPCBehaviorComponent {
    pub speed: f32,
    pub last_update: f32,
    pub update_interval: f32,
}

/// Movement Controller Component
#[derive(Component)]
pub struct MovementController {
    pub current_speed: f32,
    pub max_speed: f32,
    pub stamina: f32,
}

/// Building Type Enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildingType {
    Residential,
    Commercial,
    Industrial,
    Skyscraper,
    Generic,
}

// Legacy NPC component (kept for compatibility during migration)
#[derive(Component)]
pub struct NPC {
    pub target_position: Vec3,
    pub speed: f32,
    pub last_update: f32,
    pub update_interval: f32,
}

// NEW NPC ARCHITECTURE - Following vehicle pattern

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NPCType {
    Civilian,
    Worker,
    Police,
    Emergency,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NPCLOD {
    Full,      // 0-50m: All body parts, animations, detailed appearance
    Medium,    // 50-100m: Simplified 3-part mesh (head, torso, legs)
    Low,       // 100-150m: Single human silhouette
    StateOnly, // 150m+: No rendering, just AI/movement state
}

// Lightweight state component - always in memory
#[derive(Component, Clone)]
pub struct NPCState {
    pub npc_type: NPCType,
    pub appearance: NPCAppearance,
    pub behavior: NPCBehaviorType,
    pub target_position: Vec3,
    pub speed: f32,
    pub current_lod: NPCLOD,
    pub last_lod_check: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct NPCAppearance {
    pub height: f32,
    pub build: f32,
    pub skin_tone: Color,
    pub hair_color: Color,
    pub shirt_color: Color,
    pub pants_color: Color,
    pub gender: NPCGender,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NPCGender {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NPCBehaviorType {
    Wandering,
    Commuting,
    Working,
    Socializing,
}

impl NPCState {
    pub fn new(npc_type: NPCType) -> Self {
        use rand::prelude::*;
        let mut rng = thread_rng();
        
        let speed = match npc_type {
            NPCType::Civilian => rng.gen_range(2.0..4.0),
            NPCType::Worker => rng.gen_range(3.0..5.0),
            NPCType::Police => rng.gen_range(4.0..6.0),
            NPCType::Emergency => rng.gen_range(5.0..8.0),
        };
        
        Self {
            npc_type,
            appearance: NPCAppearance::random(),
            behavior: NPCBehaviorType::Wandering,
            target_position: Vec3::new(
                rng.gen_range(-900.0..900.0),
                1.0,
                rng.gen_range(-900.0..900.0),
            ),
            speed,
            current_lod: NPCLOD::StateOnly,
            last_lod_check: 0.0,
        }
    }
}

impl NPCAppearance {
    pub fn random() -> Self {
        use rand::prelude::*;
        let mut rng = thread_rng();
        
        let skin_tones = [
            Color::srgb(0.8, 0.6, 0.4),
            Color::srgb(0.6, 0.4, 0.3),
            Color::srgb(0.9, 0.7, 0.5),
            Color::srgb(0.7, 0.5, 0.4),
            Color::srgb(0.5, 0.3, 0.2),
        ];
        
        let hair_colors = [
            Color::srgb(0.1, 0.1, 0.1), // Black
            Color::srgb(0.3, 0.2, 0.1), // Brown
            Color::srgb(0.6, 0.4, 0.2), // Blonde
            Color::srgb(0.4, 0.3, 0.2), // Dark brown
            Color::srgb(0.7, 0.7, 0.7), // Gray
        ];
        
        let clothing_colors = [
            Color::srgb(0.2, 0.2, 0.8), // Blue
            Color::srgb(0.8, 0.2, 0.2), // Red
            Color::srgb(0.2, 0.8, 0.2), // Green
            Color::srgb(0.1, 0.1, 0.1), // Black
            Color::srgb(0.9, 0.9, 0.9), // White
            Color::srgb(0.5, 0.3, 0.1), // Brown
        ];
        
        Self {
            height: rng.gen_range(1.6..1.9),
            build: rng.gen_range(0.8..1.2),
            skin_tone: skin_tones[rng.gen_range(0..skin_tones.len())],
            hair_color: hair_colors[rng.gen_range(0..hair_colors.len())],
            shirt_color: clothing_colors[rng.gen_range(0..clothing_colors.len())],
            pants_color: clothing_colors[rng.gen_range(0..clothing_colors.len())],
            gender: if rng.gen_bool(0.5) { NPCGender::Male } else { NPCGender::Female },
        }
    }
}

// Rendering components - only present when NPC should be rendered
#[derive(Component)]
pub struct NPCRendering {
    pub lod_level: NPCLOD,
    pub body_entities: Vec<Entity>, // Child entities with body part meshes
}

// NPC Body part components (following player architecture)
#[derive(Component)]
pub struct NPCHead;

#[derive(Component)]
pub struct NPCTorso;

#[derive(Component)]
pub struct NPCLeftArm;

#[derive(Component)]
pub struct NPCRightArm;

#[derive(Component)]
pub struct NPCLeftLeg;

#[derive(Component)]
pub struct NPCRightLeg;

#[derive(Component)]
pub struct NPCBodyPart {
    pub rest_position: Vec3,
    pub rest_rotation: Quat,
    pub animation_offset: Vec3,
    pub animation_rotation: Quat,
}

// LOD distances for NPCs - optimized for 60+ FPS target
pub const NPC_LOD_FULL_DISTANCE: f32 = 25.0;
pub const NPC_LOD_MEDIUM_DISTANCE: f32 = 50.0;
pub const NPC_LOD_LOW_DISTANCE: f32 = 75.0;
pub const NPC_LOD_CULL_DISTANCE: f32 = 100.0;

// LEGACY CULLABLE COMPONENT REMOVED - replaced by UnifiedCullable



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

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum ContentType {
    Road,
    Building,
    Tree,
    Vehicle,
    NPC,
}

#[derive(Component)]
pub struct PerformanceCritical;

#[derive(Component, Clone)]
pub struct Building {
    pub building_type: BuildingType,
    pub height: f32,
    pub scale: Vec3,
}

#[derive(Component)]
pub struct Landmark;

#[derive(Component)]
pub struct Buildable;



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
            _npc_cull_distance: 100.0,
            _car_cull_distance: 150.0,
            _building_cull_distance: 300.0,
            _tree_cull_distance: 250.0,
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





// Mesh caching components
#[derive(Resource)]
pub struct MeshCache {
    pub road_meshes: std::collections::HashMap<String, Handle<Mesh>>,
    pub npc_body_meshes: std::collections::HashMap<String, Handle<Mesh>>,
    pub intersection_meshes: std::collections::HashMap<String, Handle<Mesh>>,
}

impl Default for MeshCache {
    fn default() -> Self {
        Self {
            road_meshes: std::collections::HashMap::new(),
            npc_body_meshes: std::collections::HashMap::new(),
            intersection_meshes: std::collections::HashMap::new(),
        }
    }
}

// Entity limit tracking
#[derive(Resource)]
pub struct EntityLimits {
    pub max_buildings: usize,
    pub max_vehicles: usize,
    pub max_npcs: usize,
    pub max_trees: usize,
    pub building_entities: Vec<(Entity, f32)>, // (entity, spawn_time)
    pub vehicle_entities: Vec<(Entity, f32)>,
    pub npc_entities: Vec<(Entity, f32)>,
    pub tree_entities: Vec<(Entity, f32)>,
}

impl Default for EntityLimits {
    fn default() -> Self {
        Self {
            max_buildings: 800,    // Reduced from unlimited
            max_vehicles: 200,     // Reduced from unlimited
            max_npcs: 150,         // Reduced from unlimited
            max_trees: 400,        // Reduced from unlimited
            building_entities: Vec::new(),
            vehicle_entities: Vec::new(),
            npc_entities: Vec::new(),
            tree_entities: Vec::new(),
        }
    }
}
