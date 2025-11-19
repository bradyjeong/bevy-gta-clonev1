use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficState {
    Moving,
    Stopping, // For obstacle or red light
    WaitingForLight,
}

#[derive(Component)]
pub struct TrafficAI {
    pub current_road_id: u64,
    pub current_lane: i32,
    pub spline_t: f32, // 0.0 to 1.0 progress along current road
    pub speed: f32,
    pub state: TrafficState,
    pub target_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightState {
    GreenNS, // North-South green
    YellowNS,
    GreenEW, // East-West green
    YellowEW,
    RedAll,
}

#[derive(Component)]
pub struct TrafficLight {
    pub intersection_id: u32,
    pub state: LightState,
    pub timer: f32,
}

// Marker for traffic vehicles
#[derive(Component)]
pub struct TrafficVehicle;
