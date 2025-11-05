use bevy::prelude::*;

#[derive(Component)]
pub struct JetFlame {
    pub intensity: f32,       // 0.0 to 1.0 based on throttle/afterburner
    pub base_scale: f32,      // Base size of the flame
    pub max_scale: f32,       // Maximum scale when afterburner active
    pub flicker_speed: f32,   // Speed of flame animation
    pub color_intensity: f32, // Color brightness
}

impl Default for JetFlame {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            base_scale: 1.0,
            max_scale: 2.5,
            flicker_speed: 8.0,
            color_intensity: 1.0,
        }
    }
}

#[derive(Component)]
pub struct FlameEffect {
    pub parent_vehicle: Entity, // Entity this flame is attached to
    pub offset: Vec3,           // Local position offset from vehicle
}

impl Default for FlameEffect {
    fn default() -> Self {
        Self {
            parent_vehicle: Entity::PLACEHOLDER,
            offset: Vec3::ZERO,
        }
    }
}

#[derive(Component, Default)]
pub struct VehicleBeacon;

#[derive(Component, Default)]
pub struct ControlsText;

#[derive(Component, Default)]
pub struct ControlsDisplay;

#[derive(Component, Default)]
pub struct WaypointText;
