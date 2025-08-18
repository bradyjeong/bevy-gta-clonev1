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

// Realistic aircraft flight dynamics component
#[derive(Component, Clone)]
pub struct AircraftFlight {
    // Flight control surfaces (normalized -1.0 to 1.0)
    pub pitch: f32,    // Elevator control (nose up/down)
    pub roll: f32,     // Aileron control (bank left/right)
    pub yaw: f32,      // Rudder control (nose left/right)
    pub throttle: f32, // Engine power (0.0 to 1.0)

    // Flight state
    pub airspeed: f32,
    pub angle_of_attack: f32,
    pub current_thrust: f32,

    // Engine state
    pub afterburner: bool,
    pub afterburner_active: bool, // Actual afterburner status (with delay)
    pub afterburner_timer: f32,   // Time since afterburner request
    pub engine_spool_time: f32,
}

// F16-specific flight specifications (data-only following AGENT.md)
#[derive(Component, Clone)]
pub struct F16Specs {
    // Physical properties
    pub mass: f32,               // kg
    pub wing_area: f32,          // m²
    pub max_thrust: f32,         // Newtons
    pub afterburner_thrust: f32, // Newtons

    // Flight envelope
    pub stall_speed: f32,         // m/s
    pub max_speed: f32,           // m/s
    pub max_angle_of_attack: f32, // radians

    // Aerodynamic coefficients
    pub lift_coefficient_0: f32,     // CL0 (base lift)
    pub lift_coefficient_alpha: f32, // CLα (lift per AoA)
    pub drag_coefficient: f32,       // CD

    // Control characteristics
    pub control_sensitivity: f32,
    pub yaw_scale: f32,         // Yaw sensitivity multiplier
    pub afterburner_delay: f32, // Seconds before afterburner VFX activates
    pub spool_rate_normal: f32,
    pub spool_rate_afterburner: f32,
    
    // Realistic control rates per axis
    pub roll_rate_max: f32,     // Maximum roll rate (rad/s)
    pub pitch_rate_max: f32,    // Maximum pitch rate (rad/s)
    pub yaw_rate_max: f32,      // Maximum yaw rate (rad/s)

    // Inertia tensor components (kg⋅m²)
    pub inertia_roll: f32,  // Ixx - roll axis
    pub inertia_pitch: f32, // Iyy - pitch axis
    pub inertia_yaw: f32,   // Izz - yaw axis
}

impl Default for AircraftFlight {
    fn default() -> Self {
        Self {
            // Control inputs start at neutral
            pitch: 0.0,
            roll: 0.0,
            yaw: 0.0,
            throttle: 0.0,

            // Flight state
            airspeed: 0.0,
            angle_of_attack: 0.0,
            current_thrust: 0.0,

            // Engine starts cold
            afterburner: false,
            afterburner_active: false,
            afterburner_timer: 0.0,
            engine_spool_time: 0.0,
        }
    }
}

impl Default for F16Specs {
    fn default() -> Self {
        Self {
            // F-16C Fighting Falcon realistic specifications
            mass: 12000.0,        // kg (empty weight ~8,500 kg + fuel/equipment)
            wing_area: 27.87,     // m² (300 sq ft)
            max_thrust: 130000.0, // Newtons (~29,000 lbf F100-PW-229)
            afterburner_thrust: 176000.0, // Newtons (~39,500 lbf with afterburner)

            // Flight envelope
            stall_speed: 40.0,          // m/s (~80 knots clean config)
            max_speed: 616.0,           // m/s (Mach 2.0 at altitude)
            max_angle_of_attack: 0.436, // radians (25 degrees)

            // Aerodynamic coefficients (simplified but realistic)
            lift_coefficient_0: 0.2,     // Base lift coefficient
            lift_coefficient_alpha: 5.0, // Lift curve slope (per radian)
            drag_coefficient: 0.03,      // Clean configuration drag

            // Control characteristics - realistic F-16 rates
            control_sensitivity: 3.5,    // Pitch rate (rad/s per control input)
            yaw_scale: 0.3,              // Reduced yaw for stability (1.05 rad/s)
            afterburner_delay: 0.2,      // Seconds before VFX activates
            spool_rate_normal: 2.5,      // Engine spool-up rate
            spool_rate_afterburner: 1.5, // Faster spool with afterburner
            
            // Separate control rates for each axis
            roll_rate_max: 6.3,          // Real F-16 roll rate (rad/s)
            pitch_rate_max: 3.5,         // Real F-16 pitch rate (rad/s)
            yaw_rate_max: 1.05,          // Real F-16 yaw rate (rad/s)

            // Inertia tensor (realistic F-16 values)
            inertia_roll: 9000.0,    // kg⋅m² - roll axis (slender body)
            inertia_pitch: 165000.0, // kg⋅m² - pitch axis (longer moment arm)
            inertia_yaw: 175000.0,   // kg⋅m² - yaw axis (similar to pitch)
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
