use bevy::prelude::*;

// Legacy marker components (kept for compatibility)
#[derive(Component)]
pub struct Car;

#[derive(Component, Clone)]
pub struct SuperCar {
    pub max_speed: f32,
    pub acceleration: f32,
    pub turbo_boost: bool,
    pub exhaust_timer: f32,
    
    // Advanced hypercar physics
    pub weight: f32,              // kg - affects acceleration and handling
    pub power: f32,               // hp - determines acceleration capability
    pub torque: f32,              // nm - low-end acceleration power
    pub drag_coefficient: f32,    // aerodynamic efficiency
    
    // Suspension and handling
    pub suspension_stiffness: f32,
    pub suspension_damping: f32,
    pub front_weight_bias: f32,   // 0.0-1.0, affects handling balance
    
    // Traction control system
    pub traction_control: bool,
    pub stability_control: bool,
    pub wheel_spin_threshold: f32,
    pub current_traction: f32,    // 0.0-1.0, current grip level
    
    // Turbo system
    pub turbo_pressure: f32,      // 0.0-1.0, builds up over time
    pub turbo_lag: f32,           // seconds to full boost
    pub turbo_cooldown: f32,      // prevents overheating
    pub max_turbo_time: f32,      // maximum continuous boost
    pub current_turbo_time: f32,  // current boost duration
    
    // Engine characteristics
    pub rpm: f32,                 // current engine RPM
    pub max_rpm: f32,             // redline RPM
    pub idle_rpm: f32,            // idle RPM
    pub power_band_start: f32,    // RPM where peak power begins
    pub power_band_end: f32,      // RPM where peak power ends
}

impl Default for SuperCar {
    fn default() -> Self {
        Self {
            // Bugatti Chiron specifications
            max_speed: 261.0,            // mph - actual Chiron top speed
            acceleration: 150.0,         // enhanced for 0-60 in 2.4s feel
            turbo_boost: false,
            exhaust_timer: 0.0,
            
            // Advanced hypercar physics (Chiron specs)
            weight: 1995.0,              // kg - actual Chiron weight
            power: 1500.0,               // hp - quad-turbo W16 engine
            torque: 1180.0,              // lb-ft converted to approximate scale
            drag_coefficient: 0.35,      // excellent aerodynamics
            
            // Premium suspension and handling
            suspension_stiffness: 8.5,   // adaptive suspension
            suspension_damping: 4.2,     // excellent damping control
            front_weight_bias: 0.43,     // rear-mid engine layout
            
            // Advanced traction systems
            traction_control: true,
            stability_control: true,
            wheel_spin_threshold: 0.15,  // very low for maximum grip
            current_traction: 1.0,       // start with perfect grip
            
            // Quad-turbo system
            turbo_pressure: 0.0,
            turbo_lag: 0.8,              // minimal lag for modern turbos
            turbo_cooldown: 0.0,
            max_turbo_time: 15.0,        // 15 seconds of boost before cooldown
            current_turbo_time: 0.0,
            
            // W16 engine characteristics
            rpm: 800.0,                  // idle RPM
            max_rpm: 6700.0,             // actual Chiron redline
            idle_rpm: 800.0,
            power_band_start: 2000.0,    // turbos kick in early
            power_band_end: 6000.0,      // peak power range
        }
    }
}

#[derive(Component)]
pub struct Helicopter;

#[derive(Component)]
pub struct F16;

// Realistic aircraft flight dynamics component
#[derive(Component, Clone)]
pub struct AircraftFlight {
    // Flight control surfaces (normalized -1.0 to 1.0)
    pub pitch: f32,      // Elevator control (nose up/down)
    pub roll: f32,       // Aileron control (bank left/right)
    pub yaw: f32,        // Rudder control (nose left/right)
    pub throttle: f32,   // Engine power (0.0 to 1.0)
    
    // Flight state
    pub airspeed: f32,
    pub angle_of_attack: f32,
    pub stall_speed: f32,
    pub max_speed: f32,
    
    // Aerodynamic properties
    pub lift_coefficient: f32,
    pub drag_coefficient: f32,
    pub thrust_power: f32,
    pub control_sensitivity: f32,
    
    // Engine state
    pub afterburner: bool,
    pub engine_spool_time: f32,
    pub current_thrust: f32,
}

impl Default for AircraftFlight {
    fn default() -> Self {
        Self {
            // Control inputs start at neutral
            pitch: 0.0,
            roll: 0.0,
            yaw: 0.0,
            throttle: 0.0,
            
            // Flight state
            airspeed: 0.0,
            angle_of_attack: 0.0,
            stall_speed: 40.0,     // Minimum speed to maintain lift
            max_speed: 300.0,      // Maximum airspeed
            
            // F16-specific aerodynamic properties (realistic values)
            lift_coefficient: 1.4,      // F16 has excellent lift characteristics
            drag_coefficient: 0.03,     // Low drag design for high performance
            thrust_power: 200.0,        // Powerful F100 engine with afterburner
            control_sensitivity: 3.0,   // Highly maneuverable fighter jet
            
            // Engine starts cold
            afterburner: false,
            engine_spool_time: 0.0,
            current_thrust: 0.0,
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
            VehicleType::BasicCar => (60.0, 20.0),
            VehicleType::SuperCar => (261.0, 150.0),
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
