use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleStats {
    pub engine_power: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub braking_force: f32,
    pub turning_radius: f32,
    pub mass: f32,
    pub fuel_capacity: f32,
    pub fuel_consumption: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct VehicleStatsConfig {
    pub vehicle_configs: HashMap<String, VehicleStats>,
}

impl Default for VehicleStatsConfig {
    fn default() -> Self {
        let mut configs = HashMap::new();
        
        configs.insert("Car".to_string(), VehicleStats {
            engine_power: 150.0,
            max_speed: 160.0,
            acceleration: 5.0,
            braking_force: 8.0,
            turning_radius: 6.0,
            mass: 1600.0,
            fuel_capacity: 55.0,
            fuel_consumption: 0.08,
        });
        
        configs.insert("SuperCar".to_string(), VehicleStats {
            engine_power: 450.0,
            max_speed: 280.0,
            acceleration: 8.5,
            braking_force: 12.0,
            turning_radius: 8.5,
            mass: 1400.0,
            fuel_capacity: 60.0,
            fuel_consumption: 0.12,
        });
        
        Self {
            vehicle_configs: configs,
        }
    }
}

impl VehicleStatsConfig {
    #[must_use] pub fn get_stats(&self, vehicle_type: &str) -> Option<&VehicleStats> {
        self.vehicle_configs.get(vehicle_type)
    }
}
