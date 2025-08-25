use bevy::prelude::*;

/// Unified vehicle specifications - merges duplicate data from SimpleF16Specs and VehicleState
#[derive(Component, Clone, Debug)]
pub struct UnifiedVehicleSpecs {
    // Physics properties
    pub mass: f32,
    pub max_thrust: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,

    // Performance (was duplicated between SimpleF16Specs and VehicleState)
    pub max_speed: f32,
    pub acceleration: f32,

    // Flight-specific (only for aircraft)
    pub roll_rate_max: Option<f32>,
    pub pitch_rate_max: Option<f32>,
    pub yaw_rate_max: Option<f32>,
    pub min_altitude: Option<f32>,

    // Visual properties
    pub color: Color,
}

impl UnifiedVehicleSpecs {
    /// Create F-16 specifications
    pub fn f16() -> Self {
        Self {
            mass: 8000.0,
            max_thrust: 75000.0,
            linear_damping: 0.98,
            angular_damping: 0.95,
            max_speed: 300.0,
            acceleration: 100.0,
            roll_rate_max: Some(3.0),
            pitch_rate_max: Some(2.0),
            yaw_rate_max: Some(1.5),
            min_altitude: Some(1.0),
            color: Color::srgb(0.35, 0.37, 0.40), // F-16 Falcon Gray
        }
    }

    /// Create car specifications
    pub fn car() -> Self {
        Self {
            mass: 1500.0,
            max_thrust: 2000.0,
            linear_damping: 0.9,
            angular_damping: 0.8,
            max_speed: 60.0,
            acceleration: 20.0,
            roll_rate_max: None,
            pitch_rate_max: None,
            yaw_rate_max: None,
            min_altitude: None,
            color: Color::srgb(0.8, 0.0, 0.0),
        }
    }
}

/// Improved logging targets - replaces emoji spam with structured logging
pub mod log_targets {
    pub const VEHICLE_SPAWN: &str = "vehicle::spawn";
    pub const VEHICLE_PHYSICS: &str = "vehicle::physics";
    pub const VEHICLE_BOUNDS: &str = "vehicle::bounds";
    pub const VEHICLE_INTERACTION: &str = "vehicle::interaction";
    pub const FLAME_EFFECTS: &str = "effects::flames";
}
