use bevy::prelude::*;



/// NPC Behavior Component - replaces old `NPCBehavior`
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
    // Compatibility fields for spawners
    pub health: Option<f32>,
    pub max_health: Option<f32>,
    pub behavior_state: Option<NPCBehaviorState>,
    pub spawn_time: Option<f32>,
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
pub enum NPCBehaviorState {
    Idle,
    Walking,
    Running,
    Interacting,
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
    #[must_use] pub fn new(npc_type: NPCType) -> Self {
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
    #[must_use] pub fn random() -> Self {
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

#[derive(Component)]
pub struct Cullable {
    pub max_distance: f32,
    pub is_culled: bool,
}

impl Cullable {
    #[must_use] pub fn new(max_distance: f32) -> Self {
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

// Road network types
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

impl RoadType {
    #[must_use] pub fn width(&self) -> f32 {
        match self {
            RoadType::Highway => 12.0,
            RoadType::MainStreet => 8.0,
            RoadType::SideStreet => 6.0,
            RoadType::Alley => 4.0,
        }
    }
    
    #[must_use] pub fn priority(&self) -> u32 {
        match self {
            RoadType::Highway => 4,
            RoadType::MainStreet => 3,
            RoadType::SideStreet => 2,
            RoadType::Alley => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoadSpline {
    pub points: Vec<Vec3>,
    pub road_type: RoadType,
    pub width: f32,
    pub connections: Vec<u32>,
}

impl Default for RoadSpline {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            road_type: RoadType::default(),
            width: 4.0,
            connections: Vec::new(),
        }
    }
}

impl RoadSpline {
    #[must_use] pub fn closest_point(&self, position: Vec3) -> (f32, Vec3) {
        if self.points.is_empty() {
            return (0.0, position);
        }
        
        let mut min_distance = f32::MAX;
        let mut closest_point = self.points[0];
        
        for point in &self.points {
            let distance = position.distance(*point);
            if distance < min_distance {
                min_distance = distance;
                closest_point = *point;
            }
        }
        
        (min_distance, closest_point)
    }
    
    #[must_use] pub fn evaluate(&self, t: f32) -> Vec3 {
        if self.points.is_empty() {
            return Vec3::ZERO;
        }
        
        if self.points.len() == 1 {
            return self.points[0];
        }
        
        let t = t.clamp(0.0, 1.0);
        let segment_count = self.points.len() - 1;
        let segment_t = t * segment_count as f32;
        let segment_index = (segment_t as usize).min(segment_count - 1);
        let local_t = segment_t - segment_index as f32;
        
        let start = self.points[segment_index];
        let end = self.points[segment_index + 1];
        
        start.lerp(end, local_t)
    }
    
    #[must_use] pub fn direction_at(&self, t: f32) -> Vec3 {
        if self.points.len() < 2 {
            return Vec3::X; // Default direction
        }
        
        let t = t.clamp(0.0, 1.0);
        let segment_count = self.points.len() - 1;
        let segment_t = t * segment_count as f32;
        let segment_index = (segment_t as usize).min(segment_count - 1);
        
        let start = self.points[segment_index];
        let end = if segment_index + 1 < self.points.len() {
            self.points[segment_index + 1]
        } else {
            self.points[segment_index]
        };
        
        (end - start).normalize_or_zero()
    }
    
    #[must_use] pub fn length(&self) -> f32 {
        if self.points.len() < 2 {
            return 0.0;
        }
        
        let mut total_length = 0.0;
        for i in 0..self.points.len() - 1 {
            total_length += self.points[i].distance(self.points[i + 1]);
        }
        total_length
    }
}

#[derive(Debug, Clone)]
pub enum IntersectionType {
    TJunction,
    CrossRoads,
    Roundabout,
    Cross,
    Curve,
    HighwayOnramp,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ContentLayer {
    Roads,
    Buildings,
    Vehicles,
    Vegetation,
    Landmarks,
    NPCs,
}

#[derive(Debug, Clone, Default, Resource)]
pub struct RoadNetwork {
    pub roads: std::collections::HashMap<u32, RoadSpline>,
    pub intersections: std::collections::HashMap<u32, RoadIntersection>,
    pub next_road_id: u32,
    pub next_intersection_id: u32,
    pub generated_chunks: std::collections::HashSet<(i32, i32)>,
}

#[derive(Debug, Clone)]
pub struct RoadIntersection {
    pub position: Vec3,
    pub connected_roads: Vec<u32>,
    pub intersection_type: IntersectionType,
    pub radius: f32,
}

impl RoadNetwork {
    #[must_use] pub fn new() -> Self {
        Self {
            roads: std::collections::HashMap::new(),
            intersections: std::collections::HashMap::new(),
            next_road_id: 1,
            next_intersection_id: 1,
            generated_chunks: std::collections::HashSet::new(),
        }
    }
    
    pub fn add_road(&mut self, start: Vec3, end: Vec3, road_type: RoadType) -> u32 {
        let id = self.next_road_id;
        self.next_road_id += 1;
        
        let road = RoadSpline {
            points: vec![start, end],
            road_type,
            width: 8.0,
            connections: Vec::new(),
        };
        
        self.roads.insert(id, road);
        id
    }
    
    pub fn add_intersection(&mut self, position: Vec3, intersection_type: IntersectionType) -> u32 {
        let id = self.next_intersection_id;
        self.next_intersection_id += 1;
        
        let intersection = RoadIntersection {
            position,
            connected_roads: Vec::new(),
            intersection_type,
            radius: 10.0, // Default radius
        };
        
        self.intersections.insert(id, intersection);
        id
    }
    
    pub fn connect_roads(&mut self, road1_id: u32, road2_id: u32) {
        if let Some(road1) = self.roads.get_mut(&road1_id) {
            if !road1.connections.contains(&road2_id) {
                road1.connections.push(road2_id);
            }
        }
        
        if let Some(road2) = self.roads.get_mut(&road2_id) {
            if !road2.connections.contains(&road1_id) {
                road2.connections.push(road1_id);
            }
        }
    }
    
    pub fn get_all_roads(&self) -> impl Iterator<Item = (u32, &RoadSpline)> {
        self.roads.iter().map(|(id, road)| (*id, road))
    }
    
    #[must_use] pub fn get_road_at_position(&self, position: Vec3, tolerance: f32) -> Option<(u32, &RoadSpline)> {
        self.roads.iter()
            .find(|(_, road)| {
                road.points.iter().any(|&point| position.distance(point) < tolerance)
            })
            .map(|(id, road)| (*id, road))
    }
    
    #[must_use] pub fn find_nearest_road(&self, position: Vec3) -> Option<(u32, &RoadSpline, Vec3)> {
        self.roads.iter()
            .map(|(id, road)| {
                let nearest_point = road.points.iter()
                    .min_by(|a, b| {
                        position.distance(**a).partial_cmp(&position.distance(**b)).unwrap()
                    })
                    .copied()
                    .unwrap_or(Vec3::ZERO);
                (*id, road, nearest_point)
            })
            .min_by(|a, b| {
                position.distance(a.2).partial_cmp(&position.distance(b.2)).unwrap()
            })
    }
    
    pub fn get_generation_cache(&mut self) -> Result<&mut std::collections::HashMap<String, bool>, String> {
        // For now, return an error to indicate this needs proper implementation
        Err("Generation cache not implemented yet".to_string())
    }
    
    pub fn clear_cache(&mut self) {
        self.generated_chunks.clear();
    }
    
    pub fn generate_chunk_roads(&mut self, chunk_x: i32, chunk_z: i32) -> Vec<u32> {
        let chunk_coord = (chunk_x, chunk_z);
        
        if self.generated_chunks.contains(&chunk_coord) {
            return Vec::new();
        }
        
        self.generated_chunks.insert(chunk_coord);
        
        // Generate a simple grid of roads for the chunk
        let mut road_ids = Vec::new();
        let chunk_size = 100.0;
        let chunk_offset_x = chunk_x as f32 * chunk_size;
        let chunk_offset_z = chunk_z as f32 * chunk_size;
        
        // Create a few roads in the chunk
        for i in 0..2 {
            for j in 0..2 {
                let start = Vec3::new(
                    chunk_offset_x + i as f32 * chunk_size * 0.5,
                    0.0,
                    chunk_offset_z + j as f32 * chunk_size * 0.5,
                );
                let end = Vec3::new(
                    chunk_offset_x + (i + 1) as f32 * chunk_size * 0.5,
                    0.0,
                    chunk_offset_z + j as f32 * chunk_size * 0.5,
                );
                
                let road_id = self.add_road(start, end, RoadType::SideStreet);
                road_ids.push(road_id);
            }
        }
        
        road_ids
    }
}

// Chunk state for world streaming
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

#[derive(Component)]
pub struct DynamicTerrain;

#[derive(Component)]
pub struct DynamicContent {
    pub content_type: ContentType,
}

#[derive(Component, Clone, PartialEq, Debug, Copy, Hash, Eq)]
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
    // Compatibility fields for spawners
    pub max_occupants: Option<u32>,
    pub current_occupants: Option<u32>,
    pub spawn_time: Option<f32>,
}

#[derive(Component)]
pub struct Landmark;

#[derive(Component)]
pub struct Buildable;

// Tree component
#[derive(Component, Clone)]
pub struct Tree {
    pub tree_type: TreeType,
    pub height: f32,
    pub age: f32,
    pub spawn_time: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    Oak,
    Pine,
    Birch,
    Maple,
    Willow,
}

impl Default for TreeType {
    fn default() -> Self {
        Self::Oak
    }
}



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
#[derive(Default)]
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
