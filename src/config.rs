use crate::systems::world::unified_world::{ChunkCoord, chunk_coord_to_index};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

/// CRITICAL: Centralized Game Configuration - Eliminates 470+ magic numbers
/// All configurations have validation bounds and safety limits
#[derive(Resource, Debug, Clone, Default)]
pub struct GameConfig {
    // Physics Configuration
    pub physics: PhysicsConfig,

    // World Configuration
    pub world: WorldConfig,

    // Vehicle Configuration
    pub vehicles: VehicleConfig,

    // NPC Configuration
    pub npc: NPCConfig,

    // Performance Configuration
    pub performance: PerformanceConfig,

    // Audio Configuration
    pub audio: AudioConfig,

    // Camera Configuration
    pub camera: CameraConfig,

    // UI Configuration
    pub ui: UIConfig,

    // World Streaming Configuration
    pub world_streaming: WorldStreamingConfig,

    // World Physics Configuration
    pub world_physics: WorldPhysicsConfig,

    // Character Dimensions Configuration
    pub character_dimensions: CharacterDimensionsConfig,

    // World Bounds Configuration
    pub world_bounds: WorldBoundsConfig,

    // World Object Configuration
    pub world_objects: WorldObjectsConfig,

    // World Environment Configuration (from world_config.ron)
    pub world_env: crate::constants::WorldEnvConfig,
}

#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    // Collision groups
    pub static_group: Group,
    pub vehicle_group: Group,
    pub character_group: Group,

    // World bounds with safety limits
    pub max_world_coord: f32, // 10000.0 - Critical boundary
    pub min_world_coord: f32, // -10000.0 - Critical boundary

    // Velocity limits with safety caps
    pub max_velocity: f32,         // 500.0 - Prevents physics explosions
    pub max_angular_velocity: f32, // 50.0 - Prevents spin chaos

    // Collider bounds with validation
    pub max_collider_size: f32, // 1000.0 - Prevents oversized colliders
    pub min_collider_size: f32, // 0.01 - Prevents degenerate colliders

    // Mass limits with safety ranges
    pub max_mass: f32, // 100000.0 - Prevents infinite mass
    pub min_mass: f32, // 0.1 - Prevents zero mass issues

    // Damping constants
    pub linear_damping: f32,  // 1.0 - Standard linear damping
    pub angular_damping: f32, // 5.0 - Standard angular damping

    // Ground friction
    pub ground_friction: f32, // 0.3 - Standard ground friction
}

#[derive(Debug, Clone)]
pub struct WorldConfig {
    // Finite world parameters (12km x 12km like GTA V)
    pub chunk_size: f32,       // 128.0 - Optimized chunk size for 4km world
    pub map_size: f32,         // 4000.0 - Finite world size (4km x 4km)
    pub total_chunks_x: usize, // 31 - Total chunks along X axis (4000/128)
    pub total_chunks_z: usize, // 31 - Total chunks along Z axis (4000/128)
    pub streaming_radius: f32, // 800.0 - Object streaming radius

    // LOD distances with performance optimization
    pub lod_distances: [f32; 3], // [300.0, 600.0, 1000.0] - LOD transitions

    // Content generation parameters
    pub building_density: f32, // 1.0 - Building spawn density
    pub tree_density: f32,     // 2.0 - Tree spawn density
    pub vehicle_density: f32,  // 0.3 - Vehicle spawn density
    pub npc_density: f32,      // 0.2 - NPC spawn density

    // Performance parameters
    pub cleanup_delay: f32,   // 30.0 - Entity cleanup delay
    pub update_interval: f32, // 0.1 - Standard update interval
}

#[derive(Debug, Clone)]
pub struct VehicleConfig {
    // Super car parameters
    pub super_car: VehicleTypeConfig,

    // Helicopter parameters
    pub helicopter: VehicleTypeConfig,

    // F16 jet parameters
    pub f16: VehicleTypeConfig,

    // Yacht parameters
    pub yacht: VehicleTypeConfig,
}

#[derive(Debug, Clone)]
pub struct VehicleTypeConfig {
    // Physical dimensions
    pub body_size: Vec3,     // Body dimensions
    pub collider_size: Vec3, // Collider dimensions

    // Performance stats
    pub max_speed: f32,    // Maximum speed
    pub acceleration: f32, // Acceleration rate

    // Physics properties
    pub mass: f32,            // Vehicle mass
    pub linear_damping: f32,  // Linear damping
    pub angular_damping: f32, // Angular damping

    // Visual properties
    pub default_color: Color, // Default vehicle color

    // Audio properties
    pub engine_volume: f32, // Engine volume
    pub horn_volume: f32,   // Horn volume
}

#[derive(Debug, Clone)]
pub struct NPCConfig {
    // Spawn parameters
    pub max_npcs: usize,     // 100 - Maximum NPC count
    pub spawn_interval: f32, // 5.0 - Spawn check interval
    pub spawn_radius: f32,   // 900.0 - Spawn area radius

    // Behavior parameters
    pub update_intervals: NPCUpdateIntervals,

    // Physical properties
    pub default_height: f32, // 1.8 - Standard NPC height
    pub default_build: f32,  // 1.0 - Standard NPC build
    pub capsule_radius: f32, // 0.4 - NPC collision capsule radius
    pub capsule_height: f32, // 0.8 - NPC collision capsule height

    // Movement properties
    pub walk_speed: f32,         // 1.5 - Walking speed
    pub run_speed: f32,          // 3.0 - Running speed
    pub avoidance_distance: f32, // 5.0 - Player avoidance distance
}

#[derive(Debug, Clone)]
pub struct NPCUpdateIntervals {
    pub close_distance: f32,  // 50.0 - Close NPC distance
    pub far_distance: f32,    // 100.0 - Far NPC distance
    pub close_interval: f32,  // 0.05 - Update interval for close NPCs
    pub medium_interval: f32, // 0.2 - Update interval for medium NPCs
    pub far_interval: f32,    // 0.5 - Update interval for far NPCs
}

#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    // Timing intervals
    #[deprecated(
        since = "0.1.0",
        note = "TimingService removed - LOD now handled by VisibilityRange. Use Local<Timer> in individual systems if throttling needed."
    )]
    pub vehicle_lod_interval: f32, // 0.1 - Vehicle LOD check interval
    #[deprecated(
        since = "0.1.0",
        note = "TimingService removed - LOD now handled by VisibilityRange. Use Local<Timer> in individual systems if throttling needed."
    )]
    pub npc_lod_interval: f32, // 0.1 - NPC LOD check interval

    #[deprecated(
        since = "0.1.0",
        note = "TimingService removed - audio cleanup no longer needed. Audio entities self-manage lifecycle."
    )]
    pub audio_cleanup_interval: f32, // 1.0 - Audio cleanup interval
    #[deprecated(
        since = "0.1.0",
        note = "TimingService removed - effects update every frame. Add Local<Timer> to effect systems if throttling needed."
    )]
    pub effect_update_interval: f32, // 0.05 - Effect update interval

    // Performance targets
    pub target_fps: f32,           // 60.0 - Target FPS
    pub frame_time_threshold: f32, // 16.67 - Target frame time (ms)

    // Culling parameters
    pub culling_check_interval: f32, // 0.5 - Culling check interval
    pub max_visible_distance: f32, // 1000.0 - Maximum visibility distance (reduced for performance)

    // VisibilityRange distances per entity type
    pub npc_visibility_distance: f32, // 125.0 - NPCs visible range
    pub vehicle_visibility_distance: f32, // 250.0 - Vehicles visible range
    pub tree_visibility_distance: f32, // 300.0 - Trees visible range
    pub building_visibility_distance: f32, // 1500.0 - High for Manhattan skyline visibility
    pub road_visibility_distance: f32, // 400.0 - Roads visible range
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    // Volume levels
    pub master_volume: f32,   // 1.0 - Master volume
    pub engine_volume: f32,   // 0.8 - Engine volume
    pub turbo_volume: f32,    // 0.9 - Turbo whistle volume
    pub exhaust_volume: f32,  // 0.85 - Exhaust volume
    pub backfire_volume: f32, // 0.7 - Backfire volume

    pub footstep_volume: f32, // 0.5 - Footstep volume

    // Audio timing
    pub footstep_intervals: FootstepConfig,

    // Spatial audio
    pub fade_distance: f32,      // 100.0 - Audio fade distance
    pub max_audio_distance: f32, // 250.0 - Maximum audio distance
}

#[derive(Debug, Clone)]
pub struct FootstepConfig {
    pub base_interval: f32,   // 0.5 - Base footstep interval
    pub walk_multiplier: f32, // 1.0 - Walking speed multiplier
    pub run_multiplier: f32,  // 0.6 - Running speed multiplier
    pub variation: f32,       // 0.1 - Timing variation (+/- 0.1)
}

#[derive(Debug, Clone)]
pub struct CameraConfig {
    // Chase camera parameters
    pub distance: f32,            // 20.0 - Camera distance from target
    pub height: f32,              // 12.0 - Camera height above target
    pub lerp_speed: f32,          // 0.05 - Camera smoothing speed
    pub look_ahead_distance: f32, // 10.0 - Look ahead distance
    pub look_ahead_height: f32,   // 2.0 - Look ahead height

    // Swimming camera parameters
    pub swim_distance: f32,   // 2.5 - Distance behind swimmer
    pub swim_height: f32,     // 0.6 - Height above swimmer's back
    pub swim_look_ahead: f32, // 0.5 - Look ahead distance for targeting
}

#[derive(Debug, Clone)]
pub struct UIConfig {
    // Font sizes
    pub default_font_size: f32, // 16.0 - Default UI font size
    pub fps_font_size: f32,     // 20.0 - FPS counter font size

    // Padding and margins
    pub default_padding: f32, // 10.0 - Default UI padding
    pub panel_padding: f32,   // 8.0 - Panel padding

    // Colors
    pub panel_background: Color, // Semi-transparent black
    pub text_color: Color,       // White text

    // Border radius
    pub border_radius: f32, // 5.0 - Default border radius
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStreamingConfig {
    pub chunk_size: f32,
    pub streaming_radius: f32,
    pub lod_distances: LodDistancesConfig,
    pub vehicle_lod: LodConfig,
    pub npc_lod: LodConfig,
    pub vegetation_cull_distance: f32,
    pub road_cell_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodDistancesConfig {
    pub full: f32,
    pub medium: f32,
    pub far: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodConfig {
    pub full: f32,
    pub medium: f32,
    pub low: f32,
    pub cull: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPhysicsConfig {
    pub building_activation: ActivationRadiusConfig,
    pub dynamic_physics: DynamicPhysicsConfig,
    pub boundaries: BoundariesConfig,
    pub emergency_thresholds: EmergencyThresholdsConfig,
    pub water: WaterPhysicsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationRadiusConfig {
    pub activation_radius: f32,
    pub deactivation_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPhysicsConfig {
    pub full_physics_radius: f32,
    pub disable_radius: f32,
    pub hysteresis_buffer: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundariesConfig {
    pub pushback_strength: f32,
    pub aircraft_pushback_strength: f32,
    pub altitude_pushback: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyThresholdsConfig {
    pub max_coordinate: f32,
    pub max_velocity: f32,
    pub max_angular_velocity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterPhysicsConfig {
    pub buoyancy_water_density: f32,
    pub gravity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDimensionsConfig {
    pub player: CharacterDimensions,
    pub npc: CharacterDimensions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDimensions {
    pub foot_level: f32,
    pub capsule_radius: f32,
    pub upper_sphere_y: f32,
}

impl CharacterDimensions {
    pub fn lower_sphere_y(&self) -> f32 {
        self.foot_level + self.capsule_radius
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBoundsConfig {
    pub world_half_size: f32,
    pub terrain: TerrainBoundsConfig,
    pub edge_buffer: f32,
    pub vehicle_spawn_half_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainBoundsConfig {
    pub left_x: f32,
    pub right_x: f32,
    pub half_size: f32,
}

/// World object mesh and collider configurations
#[derive(Debug, Clone)]
pub struct WorldObjectsConfig {
    // Trees
    pub palm_tree: WorldObjectConfig,

    // Ocean
    pub ocean_floor: WorldObjectConfig,

    // Roads
    pub road_segment: WorldObjectConfig,

    // Buildings (generic sizes)
    pub small_building: WorldObjectConfig,
    pub medium_building: WorldObjectConfig,
    pub large_building: WorldObjectConfig,
}

#[derive(Debug, Clone)]
pub struct WorldObjectConfig {
    pub mesh_size: Vec3,
    pub collider_size: Vec3,
    pub collider_type: ColliderType,
}

#[derive(Debug, Clone, Copy)]
pub enum ColliderType {
    Cuboid,
    Cylinder { half_height: f32, radius: f32 },
    Capsule { half_height: f32, radius: f32 },
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            static_group: Group::GROUP_1,
            vehicle_group: Group::GROUP_2,
            character_group: Group::GROUP_3,
            max_world_coord: 10000.0,
            min_world_coord: -10000.0,
            max_velocity: 800.0,
            max_angular_velocity: 50.0,
            max_collider_size: 1000.0,
            min_collider_size: 0.01,
            max_mass: 100000.0,
            min_mass: 0.1,
            linear_damping: 1.0,
            angular_damping: 5.0,
            ground_friction: 0.3,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        let chunk_size = 128.0;
        let map_size = 6000.0; // Expanded from 4km to 6km for ocean zones
        let total_chunks = (map_size / chunk_size) as usize;

        Self {
            chunk_size,
            map_size,
            total_chunks_x: total_chunks,
            total_chunks_z: total_chunks,
            streaming_radius: 800.0, // Reduced from 1200 for 4km world
            lod_distances: [150.0, 300.0, 500.0],
            building_density: 0.5,
            tree_density: 2.0,
            vehicle_density: 0.3,
            npc_density: 0.2,
            cleanup_delay: 30.0,
            update_interval: 0.1,
        }
    }
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            super_car: VehicleTypeConfig {
                body_size: Vec3::new(1.9, 1.3, 4.7),        // Visual mesh size
                collider_size: Vec3::new(1.52, 1.04, 3.76), // 0.8x visual for GTA-style forgiving collision
                max_speed: 70.0,
                acceleration: 40.0,
                mass: 1400.0,
                linear_damping: 1.0,
                angular_damping: 5.0,
                default_color: Color::srgb(1.0, 0.0, 0.0),
                engine_volume: 0.8,
                horn_volume: 0.9,
            },
            helicopter: VehicleTypeConfig {
                body_size: Vec3::new(3.0, 3.0, 12.0),    // Visual mesh size
                collider_size: Vec3::new(2.4, 2.4, 9.6), // 0.8x visual for GTA-style forgiving collision
                max_speed: 83.0,
                acceleration: 30.0,
                mass: 2500.0,
                linear_damping: 2.0,
                angular_damping: 8.0,
                default_color: Color::srgb(0.9, 0.9, 0.9),
                engine_volume: 1.0,
                horn_volume: 0.5,
            },
            f16: VehicleTypeConfig {
                body_size: Vec3::new(15.0, 5.0, 10.0),    // Visual mesh size
                collider_size: Vec3::new(12.0, 4.0, 8.0), // 0.8x visual for GTA-style forgiving collision
                max_speed: 600.0,
                acceleration: 80.0,
                mass: 8000.0,
                linear_damping: 0.5,
                angular_damping: 3.0,
                default_color: Color::srgb(0.4, 0.4, 0.5),
                engine_volume: 1.2,
                horn_volume: 0.3,
            },
            yacht: VehicleTypeConfig {
                body_size: Vec3::new(20.0, 8.0, 60.0), // Visual mesh size (from vehicle_factory.rs)
                collider_size: Vec3::new(10.0, 3.0, 30.0), // 0.5x for boats (from vehicle_factory.rs:715)
                max_speed: 30.0,
                acceleration: 10.0,
                mass: 50000.0,
                linear_damping: 3.0,
                angular_damping: 10.0,
                default_color: Color::srgb(0.95, 0.95, 1.0),
                engine_volume: 0.6,
                horn_volume: 0.8,
            },
        }
    }
}

impl Default for NPCConfig {
    fn default() -> Self {
        Self {
            max_npcs: 100,
            spawn_interval: 5.0,
            spawn_radius: 900.0,
            update_intervals: NPCUpdateIntervals::default(),
            default_height: 1.8,
            default_build: 1.0,
            capsule_radius: 0.4,
            capsule_height: 0.8,
            walk_speed: 2.5, // Increased from 1.5 to match faster player walking
            run_speed: 5.0,  // Increased from 3.0 to match faster player running
            avoidance_distance: 5.0,
        }
    }
}

impl Default for NPCUpdateIntervals {
    fn default() -> Self {
        Self {
            close_distance: 50.0,
            far_distance: 100.0,
            close_interval: 0.05,
            medium_interval: 0.2,
            far_interval: 0.5,
        }
    }
}

impl Default for PerformanceConfig {
    #[allow(deprecated)]
    fn default() -> Self {
        Self {
            vehicle_lod_interval: 0.1,
            npc_lod_interval: 0.1,

            audio_cleanup_interval: 1.0,
            effect_update_interval: 0.05,
            target_fps: 60.0,
            frame_time_threshold: 16.67,
            culling_check_interval: 0.5,
            max_visible_distance: 1000.0,
            npc_visibility_distance: 125.0,
            vehicle_visibility_distance: 250.0,
            tree_visibility_distance: 300.0,
            building_visibility_distance: 1500.0, // High for Manhattan skyline visibility
            road_visibility_distance: 400.0,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            engine_volume: 0.8,
            turbo_volume: 0.9,
            exhaust_volume: 0.85,
            backfire_volume: 0.7,

            footstep_volume: 0.5,
            footstep_intervals: FootstepConfig::default(),
            fade_distance: 100.0,
            max_audio_distance: 250.0,
        }
    }
}

impl Default for FootstepConfig {
    fn default() -> Self {
        Self {
            base_interval: 0.5,
            walk_multiplier: 1.0,
            run_multiplier: 0.6,
            variation: 0.1,
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            distance: 8.0,   // Much closer to player
            height: 1.5,     // Lower for more parallel to ground
            lerp_speed: 2.5, // Near-instant camera response
            look_ahead_distance: 10.0,
            look_ahead_height: 2.0,

            // Swimming camera parameters
            swim_distance: 2.5,   // Behind swimmer
            swim_height: 0.6,     // Above swimmer's back
            swim_look_ahead: 0.5, // Look ahead targeting
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            default_font_size: 16.0,
            fps_font_size: 20.0,
            default_padding: 10.0,
            panel_padding: 8.0,
            panel_background: Color::srgba(0.0, 0.0, 0.0, 0.7),
            text_color: Color::srgb(1.0, 1.0, 1.0),
            border_radius: 5.0,
        }
    }
}

impl Default for WorldStreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 200.0,
            streaming_radius: 800.0,
            lod_distances: LodDistancesConfig {
                full: 150.0,
                medium: 300.0,
                far: 500.0,
            },
            vehicle_lod: LodConfig {
                full: 50.0,
                medium: 100.0,
                low: 125.0,
                cull: 150.0,
            },
            npc_lod: LodConfig {
                full: 25.0,
                medium: 50.0,
                low: 75.0,
                cull: 100.0,
            },
            vegetation_cull_distance: 500.0,
            road_cell_size: 400.0,
        }
    }
}

impl Default for WorldPhysicsConfig {
    fn default() -> Self {
        Self {
            building_activation: ActivationRadiusConfig {
                activation_radius: 200.0,
                deactivation_radius: 250.0,
            },
            dynamic_physics: DynamicPhysicsConfig {
                full_physics_radius: 100.0,
                disable_radius: 300.0,
                hysteresis_buffer: 50.0,
            },
            boundaries: BoundariesConfig {
                pushback_strength: 100.0,
                aircraft_pushback_strength: 150.0,
                altitude_pushback: 50.0,
            },
            emergency_thresholds: EmergencyThresholdsConfig {
                max_coordinate: 100000.0,
                max_velocity: 600.0,
                max_angular_velocity: 20.0,
            },
            water: WaterPhysicsConfig {
                buoyancy_water_density: 1000.0,
                gravity: 9.81,
            },
        }
    }
}

impl Default for CharacterDimensionsConfig {
    fn default() -> Self {
        Self {
            player: CharacterDimensions {
                foot_level: -0.45,
                capsule_radius: 0.25,
                upper_sphere_y: 1.45,
            },
            npc: CharacterDimensions {
                foot_level: -0.45,
                capsule_radius: 0.25,
                upper_sphere_y: 1.45,
            },
        }
    }
}

impl Default for WorldBoundsConfig {
    fn default() -> Self {
        Self {
            world_half_size: 3000.0,
            terrain: TerrainBoundsConfig {
                left_x: -1500.0,
                right_x: 1500.0,
                half_size: 600.0,
            },
            edge_buffer: 200.0,
            vehicle_spawn_half_size: 2000.0,
        }
    }
}

impl Default for WorldObjectsConfig {
    fn default() -> Self {
        Self {
            // Palm tree - from setup/environment.rs:109
            // Total capsule height = 2*half_height + 2*radius = 2*3.7 + 2*0.3 = 8.0m (matches mesh)
            palm_tree: WorldObjectConfig {
                mesh_size: Vec3::new(0.6, 8.0, 0.6), // Visual trunk size (8m tall)
                collider_size: Vec3::new(0.3, 3.7, 0.3), // Cylinder params (radius, half_height)
                collider_type: ColliderType::Cylinder {
                    half_height: 3.7, // Adjusted from 4.0 to match 8m visual height
                    radius: 0.3,
                },
            },

            // Ocean floor - from setup/world.rs:127
            // Original: ocean_size = 6400.0, Collider::cuboid(ocean_size / 2.0, 0.05, ocean_size / 2.0)
            ocean_floor: WorldObjectConfig {
                mesh_size: Vec3::new(10000.0, 0.1, 10000.0), // Visual ocean size
                collider_size: Vec3::new(3200.0, 0.05, 3200.0), // Half-extents (6400 / 2)
                collider_type: ColliderType::Cuboid,
            },

            // Road segment - from systems/world/road_generation.rs:122
            road_segment: WorldObjectConfig {
                mesh_size: Vec3::new(10.0, 0.04, 10.0), // Visual road size (varies)
                collider_size: Vec3::new(5.0, 0.02, 5.0), // Half-extents (varies)
                collider_type: ColliderType::Cuboid,
            },

            // Small building - from setup/world.rs:373
            small_building: WorldObjectConfig {
                mesh_size: Vec3::new(20.0, 20.0, 20.0),     // Visual size
                collider_size: Vec3::new(10.0, 10.0, 10.0), // Half-extents
                collider_type: ColliderType::Cuboid,
            },

            // Medium building
            medium_building: WorldObjectConfig {
                mesh_size: Vec3::new(40.0, 40.0, 40.0),
                collider_size: Vec3::new(20.0, 20.0, 20.0),
                collider_type: ColliderType::Cuboid,
            },

            // Large building
            large_building: WorldObjectConfig {
                mesh_size: Vec3::new(80.0, 80.0, 80.0),
                collider_size: Vec3::new(40.0, 40.0, 40.0),
                collider_type: ColliderType::Cuboid,
            },
        }
    }
}

/// CRITICAL VALIDATION FUNCTIONS - Prevent configuration errors
impl GameConfig {
    /// Validates all configuration values and clamps to safe ranges
    pub fn validate_and_clamp(&mut self) {
        self.physics.validate_and_clamp();
        self.world.validate_and_clamp();
        self.vehicles.validate_and_clamp();
        self.npc.validate_and_clamp();
        self.performance.validate_and_clamp();
        self.audio.validate_and_clamp();
        self.camera.validate_and_clamp();
        self.ui.validate_and_clamp();
        self.world_objects.validate_and_clamp();
        // Validate additional config sections
        // Note: world_bounds, world_physics, character_dimensions, world_streaming
        // don't have validate_and_clamp yet - add if needed
    }
}

impl PhysicsConfig {
    pub fn validate_and_clamp(&mut self) {
        // Clamp world coordinates to prevent infinite world issues
        self.max_world_coord = self.max_world_coord.clamp(1000.0, 50000.0);
        self.min_world_coord = self.min_world_coord.clamp(-50000.0, -1000.0);

        // Clamp velocities to prevent physics explosions
        self.max_velocity = self.max_velocity.clamp(10.0, 1000.0);
        self.max_angular_velocity = self.max_angular_velocity.clamp(1.0, 200.0);

        // Clamp collider sizes to prevent degenerate or oversized colliders
        self.max_collider_size = self.max_collider_size.clamp(100.0, 5000.0);
        self.min_collider_size = self.min_collider_size.clamp(0.001, 1.0);

        // Clamp mass to prevent physics instability
        self.max_mass = self.max_mass.clamp(1000.0, 1000000.0);
        self.min_mass = self.min_mass.clamp(0.01, 100.0);

        // Clamp damping values to reasonable ranges
        self.linear_damping = self.linear_damping.clamp(0.1, 10.0);
        self.angular_damping = self.angular_damping.clamp(0.1, 20.0);
        self.ground_friction = self.ground_friction.clamp(0.1, 2.0);
    }
}

impl WorldConfig {
    pub fn validate_and_clamp(&mut self) {
        // Clamp chunk parameters for finite world
        self.chunk_size = self.chunk_size.clamp(64.0, 256.0);
        self.map_size = self.map_size.clamp(2000.0, 10000.0); // Min 2km, max 10km
        self.streaming_radius = self.streaming_radius.clamp(200.0, 3000.0);

        // Recalculate chunk counts after validation
        let chunks_per_axis = (self.map_size / self.chunk_size) as usize;
        self.total_chunks_x = chunks_per_axis.max(1); // At least 1 chunk, no arbitrary minimum
        self.total_chunks_z = chunks_per_axis.max(1); // At least 1 chunk, no arbitrary minimum

        // Sanitize and validate LOD distances
        for distance in &mut self.lod_distances {
            if !distance.is_finite() {
                *distance = 50.0;
            }
            *distance = distance.clamp(50.0, 5000.0);
        }
        // Sort safely after sanitization
        self.lod_distances
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Clamp density values to reasonable ranges
        self.building_density = self.building_density.clamp(0.1, 5.0);
        self.tree_density = self.tree_density.clamp(0.1, 3.0);
        self.vehicle_density = self.vehicle_density.clamp(0.05, 2.0);
        self.npc_density = self.npc_density.clamp(0.01, 1.0);

        // Clamp timing parameters
        self.cleanup_delay = self.cleanup_delay.clamp(5.0, 300.0);
        self.update_interval = self.update_interval.clamp(0.01, 1.0);
    }

    /// Get total chunk count for the finite world
    pub fn total_chunk_count(&self) -> usize {
        self.total_chunks_x * self.total_chunks_z
    }

    /// Convert chunk coordinates to flat array index for Vec<Option<ChunkData>>
    pub fn chunk_coord_to_index(&self, x: i32, z: i32) -> Option<usize> {
        chunk_coord_to_index(
            ChunkCoord::new(x, z),
            self.total_chunks_x,
            self.total_chunks_z,
        )
    }

    /// Check if chunk coordinates are within finite world bounds
    pub fn is_chunk_in_bounds(&self, x: i32, z: i32) -> bool {
        self.chunk_coord_to_index(x, z).is_some()
    }

    /// Get world bounds in world coordinates
    pub fn world_bounds(&self) -> (f32, f32, f32, f32) {
        let half_size = self.map_size / 2.0;
        (-half_size, half_size, -half_size, half_size) // min_x, max_x, min_z, max_z
    }
}

impl VehicleConfig {
    pub fn validate_and_clamp(&mut self) {
        self.super_car.validate_and_clamp();
        self.helicopter.validate_and_clamp();
        self.f16.validate_and_clamp();
        self.yacht.validate_and_clamp();
    }
}

impl VehicleTypeConfig {
    pub fn validate_and_clamp(&mut self) {
        // Clamp body size to reasonable vehicle dimensions (expanded for boats and large aircraft)
        self.body_size.x = self.body_size.x.clamp(0.5, 20.0);
        self.body_size.y = self.body_size.y.clamp(0.2, 10.0); // Raised from 5.0 to accommodate yacht height 8.0
        self.body_size.z = self.body_size.z.clamp(1.0, 100.0); // Raised from 25.0 to accommodate yacht length 60.0

        // Clamp collider size to half-extents (collider_size represents HALF-EXTENTS for Rapier cuboids)
        let half_body = self.body_size * 0.5;
        self.collider_size.x = self.collider_size.x.clamp(0.25, half_body.x);
        self.collider_size.y = self.collider_size.y.clamp(0.1, half_body.y);
        self.collider_size.z = self.collider_size.z.clamp(0.5, half_body.z);

        // Clamp performance parameters
        self.max_speed = self.max_speed.clamp(10.0, 800.0);
        self.acceleration = self.acceleration.clamp(5.0, 200.0);
        self.mass = self.mass.clamp(100.0, 50000.0);

        // Clamp damping parameters
        self.linear_damping = self.linear_damping.clamp(0.1, 10.0);
        self.angular_damping = self.angular_damping.clamp(0.1, 20.0);

        // Clamp audio volumes
        self.engine_volume = self.engine_volume.clamp(0.0, 2.0);
        self.horn_volume = self.horn_volume.clamp(0.0, 2.0);
    }
}

impl NPCConfig {
    pub fn validate_and_clamp(&mut self) {
        // Clamp NPC limits to prevent performance issues
        self.max_npcs = self.max_npcs.clamp(10, 1000);
        self.spawn_interval = self.spawn_interval.clamp(0.5, 60.0);
        self.spawn_radius = self.spawn_radius.clamp(100.0, 5000.0);

        self.update_intervals.validate_and_clamp();

        // Clamp physical properties
        self.default_height = self.default_height.clamp(0.5, 3.0);
        self.default_build = self.default_build.clamp(0.3, 2.0);
        self.capsule_radius = self.capsule_radius.clamp(0.1, 1.0);
        self.capsule_height = self.capsule_height.clamp(0.2, 2.0);

        // Clamp movement speeds
        self.walk_speed = self.walk_speed.clamp(0.5, 10.0);
        self.run_speed = self.run_speed.clamp(1.0, 20.0);
        self.avoidance_distance = self.avoidance_distance.clamp(1.0, 50.0);
    }
}

impl NPCUpdateIntervals {
    pub fn validate_and_clamp(&mut self) {
        // Ensure distances are in ascending order
        if self.close_distance >= self.far_distance {
            self.far_distance = self.close_distance + 50.0;
        }

        self.close_distance = self.close_distance.clamp(10.0, 200.0);
        self.far_distance = self.far_distance.clamp(50.0, 500.0);

        // Enforce order after clamping
        if self.close_distance >= self.far_distance {
            self.far_distance = (self.close_distance + 1.0).min(500.0);
        }

        // Clamp update intervals
        self.close_interval = self.close_interval.clamp(0.01, 0.5);
        self.medium_interval = self.medium_interval.clamp(0.05, 1.0);
        self.far_interval = self.far_interval.clamp(0.1, 5.0);
    }
}

impl PerformanceConfig {
    #[allow(deprecated)]
    pub fn validate_and_clamp(&mut self) {
        // Clamp timing intervals to reasonable ranges
        self.vehicle_lod_interval = self.vehicle_lod_interval.clamp(0.01, 1.0);
        self.npc_lod_interval = self.npc_lod_interval.clamp(0.01, 1.0);

        self.audio_cleanup_interval = self.audio_cleanup_interval.clamp(0.1, 10.0);
        self.effect_update_interval = self.effect_update_interval.clamp(0.01, 0.5);

        // Clamp performance targets
        self.target_fps = self.target_fps.clamp(15.0, 240.0);
        self.frame_time_threshold = 1000.0 / self.target_fps;

        // Clamp culling parameters
        self.culling_check_interval = self.culling_check_interval.clamp(0.1, 5.0);
        self.max_visible_distance = self.max_visible_distance.clamp(500.0, 10000.0);
    }
}

impl AudioConfig {
    pub fn validate_and_clamp(&mut self) {
        // Clamp volume levels
        self.master_volume = self.master_volume.clamp(0.0, 2.0);
        self.engine_volume = self.engine_volume.clamp(0.0, 2.0);

        self.footstep_volume = self.footstep_volume.clamp(0.0, 2.0);

        self.footstep_intervals.validate_and_clamp();

        // Clamp spatial audio parameters
        self.fade_distance = self.fade_distance.clamp(10.0, 1000.0);
        self.max_audio_distance = self.max_audio_distance.clamp(50.0, 2000.0);
    }
}

impl FootstepConfig {
    pub fn validate_and_clamp(&mut self) {
        self.base_interval = self.base_interval.clamp(0.1, 2.0);
        self.walk_multiplier = self.walk_multiplier.clamp(0.5, 2.0);
        self.run_multiplier = self.run_multiplier.clamp(0.2, 1.5);
        self.variation = self.variation.clamp(0.0, 0.5);
    }
}

impl CameraConfig {
    pub fn validate_and_clamp(&mut self) {
        self.distance = self.distance.clamp(5.0, 100.0);
        self.height = self.height.clamp(1.0, 50.0); // Lowered min from 2.0 to accommodate default 1.5
        self.lerp_speed = self.lerp_speed.clamp(0.001, 5.0); // Raised max from 0.5 to accommodate default 2.5
        self.look_ahead_distance = self.look_ahead_distance.clamp(2.0, 50.0);
        self.look_ahead_height = self.look_ahead_height.clamp(0.5, 20.0);
    }
}

impl UIConfig {
    pub fn validate_and_clamp(&mut self) {
        self.default_font_size = self.default_font_size.clamp(8.0, 72.0);
        self.fps_font_size = self.fps_font_size.clamp(8.0, 72.0);
        self.default_padding = self.default_padding.clamp(0.0, 50.0);
        self.panel_padding = self.panel_padding.clamp(0.0, 50.0);
        self.border_radius = self.border_radius.clamp(0.0, 20.0);
    }
}

impl WorldObjectsConfig {
    pub fn validate_and_clamp(&mut self) {
        self.palm_tree.validate_and_clamp();
        self.ocean_floor.validate_and_clamp();
        self.road_segment.validate_and_clamp();
        self.small_building.validate_and_clamp();
        self.medium_building.validate_and_clamp();
        self.large_building.validate_and_clamp();
    }
}

impl WorldObjectConfig {
    pub fn validate_and_clamp(&mut self) {
        // Clamp mesh size to reasonable ranges
        self.mesh_size.x = self.mesh_size.x.clamp(0.1, 10000.0);
        self.mesh_size.y = self.mesh_size.y.clamp(0.01, 1000.0);
        self.mesh_size.z = self.mesh_size.z.clamp(0.1, 10000.0);

        // Validate collider parameters based on type
        match &mut self.collider_type {
            ColliderType::Cuboid => {
                // For cuboids, collider_size is half-extents, so clamp to mesh_size/2
                self.collider_size.x = self.collider_size.x.clamp(0.01, self.mesh_size.x / 2.0);
                self.collider_size.y = self.collider_size.y.clamp(0.001, self.mesh_size.y / 2.0);
                self.collider_size.z = self.collider_size.z.clamp(0.01, self.mesh_size.z / 2.0);
            }
            ColliderType::Cylinder {
                half_height,
                radius,
            } => {
                // Validate cylinder parameters
                *half_height = half_height.clamp(0.01, 1000.0);
                *radius = radius.clamp(0.01, 1000.0);
                // Keep collider_size in sync for reference
                self.collider_size = Vec3::new(*radius, *half_height, *radius);
            }
            ColliderType::Capsule {
                half_height,
                radius,
            } => {
                // Validate capsule parameters
                *half_height = half_height.clamp(0.01, 1000.0);
                *radius = radius.clamp(0.01, 1000.0);
                // Keep collider_size in sync for reference
                self.collider_size = Vec3::new(*radius, *half_height, *radius);
            }
        }
    }

    /// Helper to create a Collider from this config
    pub fn create_collider(&self) -> Collider {
        match self.collider_type {
            ColliderType::Cuboid => Collider::cuboid(
                self.collider_size.x,
                self.collider_size.y,
                self.collider_size.z,
            ),
            ColliderType::Cylinder {
                half_height,
                radius,
            } => Collider::cylinder(half_height, radius),
            ColliderType::Capsule {
                half_height,
                radius,
            } => Collider::capsule_y(half_height, radius),
        }
    }
}
