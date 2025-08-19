//! # Vehicle Components - Simplified Architecture
//!
//! This module defines all vehicle-related components following AGENT.md principles.
//! The architecture was completely refactored from a monolithic SuperCar struct
//! to focused, single-responsibility components.
//!
//! ## Component-Based Architecture
//!
//! ### Core Design Principles:
//! - **Single Responsibility**: Each component handles one specific aspect
//! - **Clear Boundaries**: No tangled interdependencies between components
//! - **Data-Only**: Components contain no logic, only data
//! - **Composable**: Components can be mixed and matched as needed
//!
//! ### SuperCar System Components:
//! - [`SuperCarSpecs`] - Performance specifications (max speed, acceleration)
//! - [`EngineState`] - Engine data (RPM, power band, torque)
//! - [`TurboSystem`] - Turbo-specific data and staging
//! - [`SuperCarSuspension`] - Suspension and weight distribution
//! - [`TractionControl`] - Stability and traction systems
//! - [`DrivingModes`] - Driving mode selection and launch control
//! - [`ExhaustSystem`] - Exhaust effects and audio
//! - [`Transmission`] - Gear ratios and transmission data
//!
//! ### Bundle Usage:
//! ```rust
//! // Create a complete SuperCar entity
//! let supercar_entity = commands.spawn((
//!     Car,
//!     SuperCarBundle::default(),
//!     Transform::default(),
//!     // ... other standard components
//! )).id();
//! ```
//!
//! ## Migration Notes:
//!
//! The old monolithic `SuperCar` struct (36 fields) has been replaced with
//! focused components. This improves:
//! - **Maintainability**: Each component can be modified independently
//! - **Performance**: Bevy ECS optimizations work better with smaller components
//! - **Testing**: Individual components can be unit tested in isolation
//! - **Understanding**: Clear separation of concerns makes code easier to follow

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum DrivingMode {
    Comfort, // Reduced power, softer suspension
    #[default]
    Sport, // Enhanced response, firmer suspension
    Track,   // Maximum performance, no limits
    Custom,  // User-defined settings
}

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum ExhaustMode {
    Quiet,  // Minimal exhaust noise
    Normal, // Standard exhaust note
    #[default]
    Sport, // Enhanced exhaust sounds
    Track,  // Maximum exhaust intensity
}

// Legacy marker components (kept for compatibility)
#[derive(Component)]
pub struct Car;

// Turbo system state
#[derive(Component, Clone)]
pub struct TurboSystem {
    pub turbo_boost: bool,
    pub pressure: f32,
    pub lag: f32,
    pub cooldown: f32,
    pub max_time: f32,
    pub current_time: f32,
    pub stage: u8,
    pub pressure_buildup: f32,
}

// Engine state and characteristics
#[derive(Component, Clone)]
pub struct EngineState {
    pub rpm: f32,
    pub max_rpm: f32,
    pub idle_rpm: f32,
    pub power_band_start: f32,
    pub power_band_end: f32,
    pub temperature: f32,
    pub oil_pressure: f32,
    pub fuel_consumption_rate: f32,
    pub rev_limiter_active: bool,
}

// Transmission and gear management
#[derive(Component, Clone)]
pub struct Transmission {
    pub gear: u8,
    pub gear_ratios: Vec<f32>,
    pub shift_rpm: f32,
    pub downshift_rpm: f32,
}

// Driving modes and launch control
#[derive(Component, Clone)]
pub struct DrivingModes {
    pub mode: DrivingMode,
    pub launch_control: bool,
    pub launch_control_engaged: bool,
    pub launch_rpm_limit: f32,
    pub sport_mode_active: bool,
    pub track_mode_active: bool,
}

// Aerodynamics system
#[derive(Component, Clone)]
pub struct AerodynamicsSystem {
    pub downforce: f32,
    pub active_aero: bool,
    pub rear_wing_angle: f32,
    pub front_splitter_level: f32,
}

// Performance metrics and telemetry
#[derive(Component, Clone)]
pub struct PerformanceMetrics {
    pub g_force_lateral: f32,
    pub g_force_longitudinal: f32,
    pub performance_timer: f32,
    pub zero_to_sixty_time: f32,
    pub is_timing_launch: bool,
}

// Audio and exhaust system
#[derive(Component, Clone)]
pub struct ExhaustSystem {
    pub note_mode: ExhaustMode,
    pub engine_note_intensity: f32,
    pub turbo_whistle_intensity: f32,
    pub backfire_timer: f32,
    pub pops_and_bangs: bool,
}

// Removed: Default implementation moved to #[derive(Default)] for marker struct

// Component-specific Default implementations

impl Default for TurboSystem {
    fn default() -> Self {
        Self {
            turbo_boost: false,
            pressure: 0.0,
            lag: 0.6,
            cooldown: 0.0,
            max_time: 18.0,
            current_time: 0.0,
            stage: 0,
            pressure_buildup: 1.2,
        }
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            rpm: 800.0,
            max_rpm: 6700.0,
            idle_rpm: 800.0,
            power_band_start: 2000.0,
            power_band_end: 6000.0,
            temperature: 0.7,
            oil_pressure: 0.8,
            fuel_consumption_rate: 22.5,
            rev_limiter_active: false,
        }
    }
}

impl Default for Transmission {
    fn default() -> Self {
        Self {
            gear: 1,
            gear_ratios: vec![3.6, 2.4, 1.8, 1.4, 1.1, 0.9, 0.75],
            shift_rpm: 6200.0,
            downshift_rpm: 3500.0,
        }
    }
}

impl Default for DrivingModes {
    fn default() -> Self {
        Self {
            mode: DrivingMode::Sport,
            launch_control: true,
            launch_control_engaged: false,
            launch_rpm_limit: 3500.0,
            sport_mode_active: true,
            track_mode_active: false,
        }
    }
}

impl Default for AerodynamicsSystem {
    fn default() -> Self {
        Self {
            downforce: 0.0,
            active_aero: true,
            rear_wing_angle: 0.0,
            front_splitter_level: 0.0,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            g_force_lateral: 0.0,
            g_force_longitudinal: 0.0,
            performance_timer: 0.0,
            zero_to_sixty_time: 0.0,
            is_timing_launch: false,
        }
    }
}

impl Default for ExhaustSystem {
    fn default() -> Self {
        Self {
            note_mode: ExhaustMode::Sport,
            engine_note_intensity: 0.8,
            turbo_whistle_intensity: 0.6,
            backfire_timer: 0.0,
            pops_and_bangs: true,
        }
    }
}

#[derive(Component)]
pub struct Helicopter;

#[derive(Component)]
pub struct F16;

// Ultra-simplified aircraft flight state - minimal necessary data
#[derive(Component, Clone)]
pub struct AircraftFlight {
    // Engine state only (eliminate derived data)
    pub throttle: f32,        // 0.0-1.0, processed from controls
    pub airspeed: f32,        // For UI/debugging only
    pub afterburner_active: bool,
}

// Simplified F16 specifications - all tuning constants data-driven
#[derive(Component, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimpleF16Specs {
    pub mass: f32,               // kg
    pub max_thrust: f32,         // Newtons  
    pub roll_rate_max: f32,      // Maximum roll rate (rad/s)
    pub pitch_rate_max: f32,     // Maximum pitch rate (rad/s)
    pub yaw_rate_max: f32,       // Maximum yaw rate (rad/s)
    pub throttle_increase_rate: f32,
    pub throttle_decrease_rate: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub lift_per_throttle: f32,
    pub min_altitude: f32,
    pub emergency_pullup_force: f32,
    
    // Previously magic numbers in code
    pub afterburner_multiplier: f32,  // Was 1.5
    pub angular_lerp_factor: f32,     // Was 8.0
    pub throttle_deadzone: f32,       // Was 0.1
}



impl Default for AircraftFlight {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            airspeed: 0.0,
            afterburner_active: false,
        }
    }
}

impl Default for SimpleF16Specs {
    fn default() -> Self {
        Self {
            mass: 12000.0,        // kg
            max_thrust: 130000.0, // Newtons
            roll_rate_max: 6.3,    // rad/s
            pitch_rate_max: 3.5,   // rad/s
            yaw_rate_max: 1.05,    // rad/s
            throttle_increase_rate: 2.0,
            throttle_decrease_rate: 3.0,
            linear_damping: 0.15,
            angular_damping: 0.05,
            lift_per_throttle: 3.0,
            min_altitude: 0.5,
            emergency_pullup_force: 20.0,
            
            // Formerly magic numbers
            afterburner_multiplier: 1.5,
            angular_lerp_factor: 8.0,
            throttle_deadzone: 0.1,
        }
    }
}



#[derive(Component)]
pub struct MainRotor;

#[derive(Component)]
pub struct TailRotor;

// NEW LOD SYSTEM

#[derive(Component, Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VehicleType {
    BasicCar,
    Helicopter,
    F16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VehicleLOD {
    Full,      // 0-100m: All details (wheels, windows, etc)
    Medium,    // 100-200m: Simplified mesh (single body)
    Low,       // 200-300m: Basic box with texture
    StateOnly, // 300m+: No rendering, just state
}

// Lightweight state component - always in memory
#[derive(Component, Clone)]
pub struct VehicleState {
    pub vehicle_type: VehicleType,
    pub color: Color,
    pub max_speed: f32,
    pub acceleration: f32,
    pub damage: f32,
    pub fuel: f32,
    pub current_lod: VehicleLOD,
    pub last_lod_check: f32,
}

impl VehicleState {
    pub fn new(vehicle_type: VehicleType) -> Self {
        let (max_speed, acceleration) = match vehicle_type {
            VehicleType::BasicCar => (60.0, 20.0),
            VehicleType::Helicopter => (80.0, 25.0),
            VehicleType::F16 => (300.0, 100.0),
        };

        Self {
            vehicle_type,
            color: Color::srgb(0.8, 0.0, 0.0),
            max_speed,
            acceleration,
            damage: 0.0,
            fuel: 100.0,
            current_lod: VehicleLOD::StateOnly,
            last_lod_check: 0.0,
        }
    }
}

// Rendering components - only present when vehicle should be rendered
#[derive(Component)]
pub struct VehicleRendering {
    pub lod_level: VehicleLOD,
    pub mesh_entities: Vec<Entity>, // Child entities with meshes
}

// LOD distances - optimized for 60+ FPS target
pub const LOD_FULL_DISTANCE: f32 = 50.0;
pub const LOD_MEDIUM_DISTANCE: f32 = 100.0;
pub const LOD_LOW_DISTANCE: f32 = 125.0;
pub const LOD_CULL_DISTANCE: f32 = 150.0;

// Simple vehicle physics configurations (asset-driven)
#[derive(Component, Debug, Clone, serde::Deserialize)]
pub struct SimpleCarSpecs {
    pub base_speed: f32,
    pub rotation_speed: f32,
    pub emergency_brake_linear: f32,
    pub emergency_brake_angular: f32,
    pub min_height: f32,
    pub ground_bounce: f32,
    pub max_processing_time: f32,
}

impl Default for SimpleCarSpecs {
    fn default() -> Self {
        Self {
            base_speed: 25.0,
            rotation_speed: 2.0,
            emergency_brake_linear: 0.1,
            emergency_brake_angular: 0.5,
            min_height: 0.1,
            ground_bounce: 1.0,
            max_processing_time: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone, serde::Deserialize)]
pub struct SimpleHelicopterSpecs {
    pub lateral_speed: f32,
    pub vertical_speed: f32,
    pub forward_speed: f32,
    pub yaw_rate: f32,
    pub pitch_rate: f32,
    pub roll_rate: f32,
    pub angular_lerp_factor: f32,
    pub linear_lerp_factor: f32,
    pub min_height: f32,
    pub ground_bounce: f32,
    pub max_processing_time: f32,
}

impl Default for SimpleHelicopterSpecs {
    fn default() -> Self {
        Self {
            lateral_speed: 20.0,
            vertical_speed: 15.0,
            forward_speed: 25.0,
            yaw_rate: 1.5,
            pitch_rate: 1.0,
            roll_rate: 1.0,
            angular_lerp_factor: 4.0,
            linear_lerp_factor: 6.0,
            min_height: 1.0,
            ground_bounce: 5.0,
            max_processing_time: 1.0,
        }
    }
}
