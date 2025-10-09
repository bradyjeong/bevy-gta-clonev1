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
            gender: if rng.gen_bool(0.5) {
                NPCGender::Male
            } else {
                NPCGender::Female
            },
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
pub struct NPCLeftFoot;

#[derive(Component)]
pub struct NPCRightFoot;

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

// DEPRECATED: Use VisibilityRange from Bevy instead
// This is kept for legacy compatibility during migration
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

// Helper function to convert Cullable distances to VisibilityRange
pub fn cullable_to_visibility_range(
    max_distance: f32,
) -> bevy::render::view::visibility::VisibilityRange {
    use bevy::render::view::visibility::VisibilityRange;
    VisibilityRange::abrupt(0.0, max_distance)
}

// Road system components
#[derive(Component)]
pub struct RoadEntity {
    pub road_id: u64,
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
#[derive(Resource, Default)]
pub struct MeshCache {
    pub road_meshes: std::collections::HashMap<String, Handle<Mesh>>,
    pub npc_body_meshes: std::collections::HashMap<String, Handle<Mesh>>,
    pub intersection_meshes: std::collections::HashMap<String, Handle<Mesh>>,
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
            max_buildings: 800, // Reduced from unlimited
            max_vehicles: 200,  // Reduced from unlimited
            max_npcs: 150,      // Reduced from unlimited
            max_trees: 400,     // Reduced from unlimited
            building_entities: Vec::new(),
            vehicle_entities: Vec::new(),
            npc_entities: Vec::new(),
            tree_entities: Vec::new(),
        }
    }
}

/// WorldBounds Resource - Finite world with context-aware boundaries
#[derive(Resource, Debug, Clone, Default)]
pub struct WorldBounds {
    // Core finite world bounds
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,

    // Context-aware boundary zones (GTA-style)
    pub warning_zone_size: f32,  // Distance from edge where warnings start
    pub critical_zone_size: f32, // Distance from edge where effects trigger
    pub boundary_enforcement: BoundaryEnforcement,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BoundaryEnforcement {
    /// Natural barriers - mountains, water, terrain that blocks movement
    Natural,
    /// Progressive deterrent - increasing hostility/danger near edges
    #[default]
    Progressive,
    /// Hard teleport - instant return to safe zone (fallback only)
    Teleport,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryVehicleType {
    OnFoot,
    GroundVehicle,
    Aircraft,
    Boat,
    Submarine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryZone {
    Safe,        // Normal gameplay area
    Warning,     // Visual/audio warnings start
    Critical,    // Context-specific effects trigger
    OutOfBounds, // Beyond world limits
}

impl WorldBounds {
    /// Create WorldBounds from GameConfig
    pub fn from_config(world_config: &crate::config::WorldConfig) -> Self {
        let (min_x, max_x, min_z, max_z) = world_config.world_bounds();

        Self {
            min_x,
            max_x,
            min_z,
            max_z,
            warning_zone_size: 500.0,  // 500m warning zone
            critical_zone_size: 200.0, // 200m critical zone
            boundary_enforcement: BoundaryEnforcement::Progressive,
        }
    }

    /// Get boundary zone for a position and vehicle type
    pub fn get_boundary_zone(
        &self,
        position: Vec3,
        _vehicle_type: BoundaryVehicleType,
    ) -> BoundaryZone {
        let distance_to_edge = self.distance_to_nearest_edge(position);

        if distance_to_edge < 0.0 {
            BoundaryZone::OutOfBounds
        } else if distance_to_edge < self.critical_zone_size {
            BoundaryZone::Critical
        } else if distance_to_edge < self.warning_zone_size {
            BoundaryZone::Warning
        } else {
            BoundaryZone::Safe
        }
    }

    /// Get distance to nearest world edge (negative if outside bounds)
    pub fn distance_to_nearest_edge(&self, position: Vec3) -> f32 {
        let x_distance = (position.x - self.min_x).min(self.max_x - position.x);
        let z_distance = (position.z - self.min_z).min(self.max_z - position.z);

        x_distance.min(z_distance)
    }

    /// Check if position is within world bounds
    pub fn is_in_bounds(&self, position: Vec3) -> bool {
        position.x >= self.min_x
            && position.x <= self.max_x
            && position.z >= self.min_z
            && position.z <= self.max_z
    }

    /// Get pushback force for position near boundaries (for gentle boundary enforcement)
    pub fn get_pushback_force(&self, position: Vec3, pushback_strength: f32) -> Vec3 {
        let mut force = Vec3::ZERO;
        let pushback_zone = self.critical_zone_size; // Use critical zone for pushback

        // X-axis pushback
        if position.x < self.min_x + pushback_zone {
            let distance = position.x - self.min_x;
            if distance < pushback_zone {
                let strength = (1.0 - distance / pushback_zone).max(0.0);
                force.x = strength * pushback_strength;
            }
        } else if position.x > self.max_x - pushback_zone {
            let distance = self.max_x - position.x;
            if distance < pushback_zone {
                let strength = (1.0 - distance / pushback_zone).max(0.0);
                force.x = -strength * pushback_strength;
            }
        }

        // Z-axis pushback
        if position.z < self.min_z + pushback_zone {
            let distance = position.z - self.min_z;
            if distance < pushback_zone {
                let strength = (1.0 - distance / pushback_zone).max(0.0);
                force.z = strength * pushback_strength;
            }
        } else if position.z > self.max_z - pushback_zone {
            let distance = self.max_z - position.z;
            if distance < pushback_zone {
                let strength = (1.0 - distance / pushback_zone).max(0.0);
                force.z = -strength * pushback_strength;
            }
        }

        force
    }

    /// Clamp position to world bounds
    pub fn clamp_to_bounds(&self, position: Vec3) -> Vec3 {
        Vec3::new(
            position.x.clamp(self.min_x, self.max_x),
            position.y, // Don't clamp Y - allow flying/underground
            position.z.clamp(self.min_z, self.max_z),
        )
    }

    /// Get safe respawn position near world center
    pub fn safe_respawn_position(&self) -> Vec3 {
        let center_x = (self.min_x + self.max_x) / 2.0;
        let center_z = (self.min_z + self.max_z) / 2.0;

        // Spawn slightly offset from exact center
        Vec3::new(center_x + 10.0, 2.0, center_z + 10.0)
    }
}

/// Boundary effect component - tracks what boundary effects are active
#[derive(Component, Debug)]
pub struct BoundaryEffects {
    pub current_zone: BoundaryZone,
    pub vehicle_type: BoundaryVehicleType,
    pub warning_active: bool,
    pub effect_intensity: f32, // 0.0 = none, 1.0 = maximum
}

impl Default for BoundaryEffects {
    fn default() -> Self {
        Self {
            current_zone: BoundaryZone::Safe,
            vehicle_type: BoundaryVehicleType::OnFoot,
            warning_active: false,
            effect_intensity: 0.0,
        }
    }
}
