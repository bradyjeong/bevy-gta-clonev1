//! # Vehicle Components - Asset-Driven Architecture
//!
//! This module defines all vehicle-related components following AGENT.md "simplicity first" principles.
//! The architecture prioritizes clean, asset-driven configuration over complex runtime components.
//!
//! ## Essential Components
//!
//! ### Marker Components:
//! - [`Car`] - Basic car marker
//! - [`Helicopter`] - Helicopter marker  
//! - [`F16`] - F16 fighter jet marker
//!
//! ### Asset-Driven Specs:
//! - [`SimpleCarSpecs`] - Car physics loaded from RON files
//! - [`SimpleHelicopterSpecs`] - Helicopter physics loaded from RON files
//! - [`SimpleF16Specs`] - F16 physics loaded from RON files
//!
//! ### Core Vehicle System:
//! - [`VehicleType`] - Vehicle classification enum
//! - [`VehicleState`] - Lightweight runtime state
//! - [`VehicleRendering`] - LOD rendering management
//! - [`AircraftFlight`] - Minimal flight state for aircraft

use bevy::prelude::*;

// Essential marker components
#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct Helicopter;

#[derive(Component)]
pub struct F16;

// Vehicle health component for boundary effects and damage
#[derive(Component, Clone, Debug)]
pub struct VehicleHealth {
    pub current: f32,
    pub max: f32,
}

impl Default for VehicleHealth {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

impl VehicleHealth {
    pub fn new(max_health: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.current <= 0.0
    }

    pub fn health_percentage(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }
}

// Ultra-simplified aircraft flight state - minimal necessary data
#[derive(Component, Clone)]
pub struct AircraftFlight {
    // Engine state only (eliminate derived data)
    pub throttle: f32, // 0.0-1.0, processed from controls
    pub airspeed: f32, // For UI/debugging only
    pub afterburner_active: bool,
}

// Simplified F16 specifications - all tuning constants data-driven
#[derive(Component, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimpleF16Specs {
    pub max_forward_speed: f32, // Maximum forward velocity (m/s)
    pub roll_rate_max: f32,     // Maximum roll rate (rad/s)
    pub pitch_rate_max: f32,    // Maximum pitch rate (rad/s)
    pub yaw_rate_max: f32,      // Maximum yaw rate (rad/s)
    pub throttle_increase_rate: f32,
    pub throttle_decrease_rate: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub lift_per_throttle: f32,
    pub drag_factor: f32, // Momentum decay when engine off

    // Previously magic numbers in code
    pub afterburner_multiplier: f32, // Speed multiplier for afterburner
    pub linear_lerp_factor: f32,     // Linear velocity smoothing rate
    pub angular_lerp_factor: f32,    // Angular velocity smoothing rate
    pub throttle_deadzone: f32,      // Minimum throttle for lift activation
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
            max_forward_speed: 200.0_f32.clamp(50.0, 500.0), // m/s - realistic fighter jet speed
            roll_rate_max: 6.3_f32.clamp(0.1, 10.0),         // rad/s - prevent excessive rotation
            pitch_rate_max: 3.5_f32.clamp(0.1, 10.0),        // rad/s
            yaw_rate_max: 1.05_f32.clamp(0.1, 5.0),          // rad/s
            throttle_increase_rate: 2.0_f32.clamp(0.1, 10.0),
            throttle_decrease_rate: 3.0_f32.clamp(0.1, 10.0),
            linear_damping: 0.15_f32.clamp(0.01, 5.0),
            angular_damping: 0.05_f32.clamp(0.01, 5.0),
            lift_per_throttle: 3.0_f32.clamp(0.1, 50.0),
            drag_factor: 0.995_f32.clamp(0.9, 1.0), // Momentum decay per second when engine off

            // Formerly magic numbers - with safety limits
            afterburner_multiplier: 1.5_f32.clamp(1.0, 3.0), // Speed multiplier for afterburner
            linear_lerp_factor: 4.0_f32.clamp(1.0, 20.0),    // Linear velocity smoothing
            angular_lerp_factor: 8.0_f32.clamp(1.0, 20.0),   // Angular velocity smoothing
            throttle_deadzone: 0.1_f32.clamp(0.0, 0.5),      // Minimum throttle for lift
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
    SuperCar,
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
            VehicleType::SuperCar => (70.0, 40.0),
            VehicleType::Helicopter => (83.0, 30.0),
            VehicleType::F16 => (600.0, 80.0),
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
    pub linear_lerp_factor: f32,
    pub angular_lerp_factor: f32,
    pub emergency_brake_linear: f32,
    pub emergency_brake_angular: f32,
    pub drag_factor: f32, // Momentum decay when no input
}

impl Default for SimpleCarSpecs {
    fn default() -> Self {
        Self {
            base_speed: 70.0_f32.clamp(1.0, 100.0), // m/s - super car speeds
            rotation_speed: 3.0_f32.clamp(0.1, 10.0), // rad/s - super cars turn faster
            linear_lerp_factor: 4.0_f32.clamp(1.0, 20.0), // Smooth movement response
            angular_lerp_factor: 6.0_f32.clamp(1.0, 20.0), // Smooth rotation response
            emergency_brake_linear: 0.1_f32.clamp(0.01, 1.0), // Multiplier - keep some movement
            emergency_brake_angular: 0.5_f32.clamp(0.01, 1.0), // Multiplier
            drag_factor: 0.98_f32.clamp(0.9, 1.0), // Momentum decay per second when no input
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
    pub drag_factor: f32, // Momentum decay when no input
    pub main_rotor_rpm: f32,
    pub tail_rotor_rpm: f32,
}

impl Default for SimpleHelicopterSpecs {
    fn default() -> Self {
        Self {
            lateral_speed: 20.0_f32.clamp(1.0, 100.0), // m/s - reasonable helicopter speeds
            vertical_speed: 15.0_f32.clamp(1.0, 50.0), // m/s - vertical flight limits
            forward_speed: 25.0_f32.clamp(1.0, 100.0), // m/s
            yaw_rate: 1.5_f32.clamp(0.1, 5.0),         // rad/s - prevent excessive rotation
            pitch_rate: 1.0_f32.clamp(0.1, 5.0),       // rad/s
            roll_rate: 1.0_f32.clamp(0.1, 5.0),        // rad/s
            angular_lerp_factor: 4.0_f32.clamp(1.0, 20.0), // Smooth control response
            linear_lerp_factor: 6.0_f32.clamp(1.0, 20.0), // Smooth movement response
            drag_factor: 0.99_f32.clamp(0.9, 1.0), // Momentum decay per second when no input
            main_rotor_rpm: 20.0_f32.clamp(1.0, 100.0), // rad/s - main rotor speed
            tail_rotor_rpm: 35.0_f32.clamp(1.0, 100.0), // rad/s - tail rotor speed
        }
    }
}
