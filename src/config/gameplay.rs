use serde::{Deserialize, Serialize};

/// Camera configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraGameplay {
    pub distance: f32,
    pub height: f32,
    pub smoothing: f32,
}

/// Gameplay configuration values
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameplayConfig {
    pub vehicle: VehicleGameplay,
    pub npc: NPCGameplay,
    pub world: WorldGameplay,
    pub physics: PhysicsGameplay,
    pub camera: CameraGameplay,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VehicleGameplay {
    pub car_top_speed: f32,
    pub car_acceleration: f32,
    pub car_brake_force: f32,
    pub car_steering_angle: f32,
    pub supercar_top_speed: f32,
    pub supercar_acceleration: f32,
    pub helicopter_lift_force: f32,
    pub helicopter_max_pitch: f32,
    pub f16_max_speed: f32,
    pub f16_afterburner_multiplier: f32,
    pub yacht_max_speed: f32,
    pub yacht_acceleration: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NPCUpdateIntervals {
    pub close_interval: f32,
    pub medium_interval: f32,
    pub far_interval: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NPCGameplay {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub reaction_time: f32,
    pub vision_range: f32,
    pub hearing_range: f32,
    pub max_health: f32,
    pub capsule_height: f32,
    pub capsule_radius: f32,
    pub update_intervals: NPCUpdateIntervals,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldGameplay {
    pub gravity: f32,
    pub wind_strength: f32,
    pub time_scale: f32,
    pub day_night_cycle_duration: f32,
    pub weather_change_frequency: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsGameplay {
    pub friction_coefficient: f32,
    pub restitution: f32,
    pub air_resistance: f32,
    pub water_resistance: f32,
    pub collision_tolerance: f32,
    pub max_velocity: f32,
    pub min_mass: f32,
    pub max_mass: f32,
    pub min_world_coord: f32,
    pub max_world_coord: f32,
    pub vehicle_group: u32,
    pub character_group: u32,
    pub static_group: u32,
    pub min_collider_size: f32,
    pub max_collider_size: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub ground_friction: f32,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            vehicle: VehicleGameplay {
                car_top_speed: 120.0,
                car_acceleration: 30.0,
                car_brake_force: 50.0,
                car_steering_angle: 0.5,
                supercar_top_speed: 200.0,
                supercar_acceleration: 60.0,
                helicopter_lift_force: 100.0,
                helicopter_max_pitch: 0.7,
                f16_max_speed: 600.0,
                f16_afterburner_multiplier: 1.5,
                yacht_max_speed: 40.0,
                yacht_acceleration: 10.0,
            },
            npc: NPCGameplay {
                walk_speed: 2.0,
                run_speed: 5.0,
                reaction_time: 0.5,
                vision_range: 50.0,
                hearing_range: 30.0,
                max_health: 100.0,
                capsule_height: 1.0,
                capsule_radius: 0.5,
                update_intervals: NPCUpdateIntervals {
                    close_interval: 0.1,
                    medium_interval: 0.3,
                    far_interval: 0.5,
                },
            },
            world: WorldGameplay {
                gravity: 9.81,
                wind_strength: 5.0,
                time_scale: 1.0,
                day_night_cycle_duration: 1440.0, // 24 minutes real-time
                weather_change_frequency: 300.0,   // Every 5 minutes
            },
            physics: PhysicsGameplay {
                friction_coefficient: 0.6,
                restitution: 0.3,
                air_resistance: 0.1,
                water_resistance: 0.8,
                collision_tolerance: 0.01,
                max_velocity: 500.0,
                min_mass: 0.1,
                max_mass: 10000.0,
                min_world_coord: -10000.0,
                max_world_coord: 10000.0,
                vehicle_group: 1,
                character_group: 2,
                static_group: 3,
                min_collider_size: 0.1,
                max_collider_size: 100.0,
                linear_damping: 0.1,
                angular_damping: 0.1,
                ground_friction: 0.6,
            },
            camera: CameraGameplay {
                distance: 10.0,
                height: 5.0,
                smoothing: 0.8,
            },
        }
    }
}
