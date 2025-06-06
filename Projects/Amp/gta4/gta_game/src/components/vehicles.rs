use bevy::prelude::*;

// Legacy marker components (kept for compatibility)
#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct SuperCar {
    pub max_speed: f32,
    pub acceleration: f32,
    pub turbo_boost: bool,
    pub exhaust_timer: f32,
}

#[derive(Component)]
pub struct Helicopter;

#[derive(Component)]
pub struct F16;

#[derive(Component)]
pub struct MainRotor;

#[derive(Component)]
pub struct TailRotor;

// NEW LOD SYSTEM

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VehicleType {
    BasicCar,
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
#[derive(Component)]
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
            VehicleType::SuperCar => (120.0, 40.0),
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

// LOD distances
pub const LOD_FULL_DISTANCE: f32 = 100.0;
pub const LOD_MEDIUM_DISTANCE: f32 = 200.0;
pub const LOD_LOW_DISTANCE: f32 = 300.0;
pub const LOD_CULL_DISTANCE: f32 = 400.0;
