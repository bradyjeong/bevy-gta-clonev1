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
