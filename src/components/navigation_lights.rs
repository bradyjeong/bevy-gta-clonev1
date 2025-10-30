use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct NavigationLight {
    pub light_type: NavigationLightType,
    pub blink_timer: Timer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavigationLightType {
    RedPort,
    GreenStarboard,
    WhiteTail,
    RedBeacon,
}

#[derive(Component)]
pub struct LandingLight {
    pub activation_altitude: f32,
}

impl Default for LandingLight {
    fn default() -> Self {
        Self {
            activation_altitude: 50.0,
        }
    }
}

impl NavigationLight {
    pub fn new(light_type: NavigationLightType) -> Self {
        let blink_interval = match light_type {
            NavigationLightType::RedBeacon => 0.8,
            NavigationLightType::WhiteTail => 1.2,
            _ => 0.0,
        };

        Self {
            light_type,
            blink_timer: Timer::from_seconds(blink_interval, TimerMode::Repeating),
        }
    }
}
