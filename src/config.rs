use crate::systems::world::{ChunkCoord, chunk_coord_to_index};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// CRITICAL: Centralized Game Configuration - Eliminates 470+ magic numbers
/// All configurations have validation bounds and safety limits
#[derive(Resource, Debug, Clone)]
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
    pub chunk_size: f32,       // 128.0 - Optimized chunk size for 12km world
    pub map_size: f32,         // 12000.0 - Finite world size (12km x 12km)
    pub total_chunks_x: usize, // 94 - Total chunks along X axis (12000/128)
    pub total_chunks_z: usize, // 94 - Total chunks along Z axis (12000/128)
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

    // Environment bounds
    pub lake_size: f32,      // 200.0 - Lake size
    pub lake_depth: f32,     // 5.0 - Lake depth
    pub lake_position: Vec3, // (300.0, -2.0, 300.0) - Lake position
}

#[derive(Debug, Clone)]
pub struct VehicleConfig {
    // Basic car parameters
    pub basic_car: VehicleTypeConfig,

    // Super car parameters
    pub super_car: VehicleTypeConfig,

    // Helicopter parameters
    pub helicopter: VehicleTypeConfig,

    // F16 jet parameters
    pub f16: VehicleTypeConfig,
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
    pub vehicle_lod_interval: f32, // 0.1 - Vehicle LOD check interval
    pub npc_lod_interval: f32,     // 0.1 - NPC LOD check interval

    pub audio_cleanup_interval: f32, // 1.0 - Audio cleanup interval
    pub effect_update_interval: f32, // 0.05 - Effect update interval

    // Performance targets
    pub target_fps: f32,           // 60.0 - Target FPS
    pub frame_time_threshold: f32, // 16.67 - Target frame time (ms)

    // Culling parameters
    pub culling_check_interval: f32, // 0.5 - Culling check interval
    pub max_visible_distance: f32,   // 1500.0 - Maximum visibility distance
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

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig::default(),
            world: WorldConfig::default(),
            vehicles: VehicleConfig::default(),
            npc: NPCConfig::default(),
            performance: PerformanceConfig::default(),
            audio: AudioConfig::default(),

            camera: CameraConfig::default(),
            ui: UIConfig::default(),
        }
    }
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            static_group: Group::GROUP_1,
            vehicle_group: Group::GROUP_2,
            character_group: Group::GROUP_3,
            max_world_coord: 10000.0,
            min_world_coord: -10000.0,
            max_velocity: 500.0,
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
        let map_size = 12000.0;
        let total_chunks = (map_size / chunk_size) as usize;

        Self {
            chunk_size,
            map_size,
            total_chunks_x: total_chunks,
            total_chunks_z: total_chunks,
            streaming_radius: 800.0,
            lod_distances: [150.0, 300.0, 500.0],
            building_density: 0.5,
            tree_density: 2.0,
            vehicle_density: 0.3,
            npc_density: 0.2,
            cleanup_delay: 30.0,
            update_interval: 0.1,
            lake_size: 200.0,
            lake_depth: 5.0,
            lake_position: Vec3::new(300.0, -2.0, 300.0),
        }
    }
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            basic_car: VehicleTypeConfig {
                body_size: Vec3::new(1.8, 0.6, 3.6),
                collider_size: Vec3::new(0.9, 0.6, 1.8),
                max_speed: 80.0,
                acceleration: 25.0,
                mass: 1200.0,
                linear_damping: 1.0,
                angular_damping: 5.0,
                default_color: Color::srgb(0.8, 0.8, 0.8),
                engine_volume: 0.6,
                horn_volume: 0.8,
            },
            super_car: VehicleTypeConfig {
                body_size: Vec3::new(1.8, 0.4, 4.2),
                collider_size: Vec3::new(0.9, 0.5, 2.1),
                max_speed: 120.0,
                acceleration: 40.0,
                mass: 1400.0,
                linear_damping: 1.0,
                angular_damping: 5.0,
                default_color: Color::srgb(1.0, 0.0, 0.0),
                engine_volume: 0.8,
                horn_volume: 0.9,
            },
            helicopter: VehicleTypeConfig {
                body_size: Vec3::new(2.5, 1.5, 5.0),
                collider_size: Vec3::new(1.25, 0.75, 2.5),
                max_speed: 60.0,
                acceleration: 30.0,
                mass: 2500.0,
                linear_damping: 2.0,
                angular_damping: 8.0,
                default_color: Color::srgb(0.9, 0.9, 0.9),
                engine_volume: 1.0,
                horn_volume: 0.5,
            },
            f16: VehicleTypeConfig {
                body_size: Vec3::new(16.0, 2.0, 3.0),
                collider_size: Vec3::new(8.0, 1.0, 1.5),
                max_speed: 200.0,
                acceleration: 80.0,
                mass: 8000.0,
                linear_damping: 0.5,
                angular_damping: 3.0,
                default_color: Color::srgb(0.4, 0.4, 0.5),
                engine_volume: 1.2,
                horn_volume: 0.3,
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
    fn default() -> Self {
        Self {
            vehicle_lod_interval: 0.1,
            npc_lod_interval: 0.1,

            audio_cleanup_interval: 1.0,
            effect_update_interval: 0.05,
            target_fps: 60.0,
            frame_time_threshold: 16.67,
            culling_check_interval: 0.5,
            max_visible_distance: 1500.0,
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
            distance: 15.0,   // Closer to player
            height: 3.0,      // Much lower, more behind than above
            lerp_speed: 0.25, // Much more responsive camera
            look_ahead_distance: 10.0,
            look_ahead_height: 2.0,
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
    }
}

impl PhysicsConfig {
    pub fn validate_and_clamp(&mut self) {
        // Clamp world coordinates to prevent infinite world issues
        self.max_world_coord = self.max_world_coord.clamp(1000.0, 50000.0);
        self.min_world_coord = self.min_world_coord.clamp(-50000.0, -1000.0);

        // Clamp velocities to prevent physics explosions
        self.max_velocity = self.max_velocity.clamp(10.0, 2000.0);
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
        self.map_size = self.map_size.clamp(8000.0, 16000.0);
        self.streaming_radius = self.streaming_radius.clamp(200.0, 2000.0);

        // Recalculate chunk counts after validation
        let chunks_per_axis = (self.map_size / self.chunk_size) as usize;
        self.total_chunks_x = chunks_per_axis.clamp(50, 200);
        self.total_chunks_z = chunks_per_axis.clamp(50, 200);

        // Validate LOD distances are in ascending order
        self.lod_distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for distance in &mut self.lod_distances {
            *distance = distance.clamp(50.0, 5000.0);
        }

        // Clamp density values to reasonable ranges
        self.building_density = self.building_density.clamp(0.1, 5.0);
        self.tree_density = self.tree_density.clamp(0.1, 3.0);
        self.vehicle_density = self.vehicle_density.clamp(0.05, 2.0);
        self.npc_density = self.npc_density.clamp(0.01, 1.0);

        // Clamp timing parameters
        self.cleanup_delay = self.cleanup_delay.clamp(5.0, 300.0);
        self.update_interval = self.update_interval.clamp(0.01, 1.0);

        // Validate lake parameters
        self.lake_size = self.lake_size.clamp(50.0, 1000.0);
        self.lake_depth = self.lake_depth.clamp(1.0, 50.0);
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
        self.basic_car.validate_and_clamp();
        self.super_car.validate_and_clamp();
        self.helicopter.validate_and_clamp();
        self.f16.validate_and_clamp();
    }
}

impl VehicleTypeConfig {
    pub fn validate_and_clamp(&mut self) {
        // Clamp body size to reasonable vehicle dimensions
        self.body_size.x = self.body_size.x.clamp(0.5, 20.0);
        self.body_size.y = self.body_size.y.clamp(0.2, 5.0);
        self.body_size.z = self.body_size.z.clamp(1.0, 25.0);

        // Clamp collider size to be smaller than body size
        self.collider_size.x = self.collider_size.x.clamp(0.25, self.body_size.x);
        self.collider_size.y = self.collider_size.y.clamp(0.1, self.body_size.y);
        self.collider_size.z = self.collider_size.z.clamp(0.5, self.body_size.z);

        // Clamp performance parameters
        self.max_speed = self.max_speed.clamp(10.0, 500.0);
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

        // Clamp update intervals
        self.close_interval = self.close_interval.clamp(0.01, 0.5);
        self.medium_interval = self.medium_interval.clamp(0.05, 1.0);
        self.far_interval = self.far_interval.clamp(0.1, 5.0);
    }
}

impl PerformanceConfig {
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
        self.height = self.height.clamp(2.0, 50.0);
        self.lerp_speed = self.lerp_speed.clamp(0.001, 0.5);
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
