use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Serialize, Deserialize, Clone)]
pub struct YachtSpecs {
    pub mass: f32,
    pub max_thrust: f32,
    pub max_speed: f32,
    pub rudder_power: f32,
    pub throttle_ramp: f32,
    pub rudder_ramp: f32,
    pub drag_longitudinal: f32,
    pub drag_lateral: f32,
    pub drag_vertical: f32,
    pub yaw_damping: f32,
    pub buoyancy_damping: f32,
    pub target_submersion: f32,
    pub buoyancy_points: Vec<(f32, f32, f32)>,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

#[derive(Component, Default)]
pub struct YachtState {
    pub throttle: f32,
    pub rudder: f32,
    pub current_thrust: f32,
    pub current_rudder: f32,
    pub on_water: bool,        // Yacht is in water region
    pub thrust_in_water: bool, // Propeller is submerged (can generate thrust)
}

#[derive(Resource)]
pub struct WaterSurface {
    pub sea_level: f32,
}

impl Default for WaterSurface {
    fn default() -> Self {
        Self { sea_level: 0.0 }
    }
}

impl WaterSurface {
    pub fn sample(&self, _position: Vec3) -> (f32, Vec3) {
        (self.sea_level, Vec3::Y)
    }
}

#[derive(Component, Default)]
pub struct Yacht {
    pub speed: f32,
    pub max_speed: f32,
    pub turning_speed: f32,
    pub buoyancy: f32,
    pub wake_enabled: bool,
}

#[derive(Component)]
pub struct WaterBody;

#[derive(Component)]
pub struct WaterWave {
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
}

#[derive(Component)]
pub struct Boat;
