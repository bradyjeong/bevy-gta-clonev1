use bevy::prelude::*;
// use bevy_rapier3d::prelude::*;

/// CRITICAL: Realistic vehicle physics components with performance safeguards
/// All values are clamped and validated to prevent physics instability

#[derive(Component, Debug, Clone)]
pub struct VehicleSuspension {
    // Suspension parameters - validated ranges
    pub spring_strength: f32,      // 15000.0 - 50000.0 (N/m)
    pub damping_ratio: f32,        // 0.3 - 0.8 (critical damping ratio)
    pub max_compression: f32,      // 0.1 - 0.5 (meters)
    pub rest_length: f32,          // 0.3 - 0.8 (meters)
    
    // Current suspension state
    pub compression: f32,          // Current compression (0.0 - max_compression)
    pub velocity: f32,             // Compression velocity (m/s)
    pub force: f32,                // Current suspension force (N)
    
    // Wheel position relative to vehicle center
    pub wheel_position: Vec3,      // Local wheel position
    pub ground_contact: bool,      // Whether wheel is touching ground
    pub ground_normal: Vec3,       // Surface normal at contact point
}

#[derive(Component, Debug, Clone)]
pub struct TirePhysics {
    // Tire grip parameters - performance optimized
    pub dry_grip: f32,             // 1.0 - 1.4 (dry surface coefficient)
    pub wet_grip: f32,             // 0.6 - 0.9 (wet surface coefficient)
    pub lateral_grip: f32,         // 0.8 - 1.2 (cornering grip)
    pub rolling_resistance: f32,   // 0.01 - 0.03 (rolling friction)
    
    // Tire state
    pub slip_ratio: f32,           // Longitudinal slip (0.0 - 1.0)
    pub slip_angle: f32,           // Lateral slip angle (radians)
    pub tire_temperature: f32,     // Tire temperature (affects grip)
    pub wear_level: f32,           // Tire wear (1.0 = new, 0.0 = worn)
    
    // Forces
    pub longitudinal_force: f32,   // Forward/backward force
    pub lateral_force: f32,        // Side force
    pub normal_force: f32,         // Downward force from weight
}

#[derive(Component, Debug, Clone)]
pub struct VehicleDynamics {
    // Mass distribution - critical for stability
    pub total_mass: f32,           // Total vehicle mass (kg)
    pub front_weight_ratio: f32,   // 0.4 - 0.7 (front weight distribution)
    pub center_of_gravity: Vec3,   // Center of gravity offset
    pub inertia_tensor: Vec3,      // Rotational inertia (x, y, z)
    
    // Aerodynamics - simplified for performance
    pub drag_coefficient: f32,     // 0.2 - 0.5 (Cd)
    pub frontal_area: f32,         // 1.5 - 3.0 (m²)
    pub downforce_coefficient: f32, // 0.0 - 1.0 (additional downforce)
    
    // Current dynamics state
    pub weight_transfer: Vec3,     // Current weight transfer
    pub aerodynamic_force: Vec3,   // Current aero forces
    pub speed: f32,                // Current speed (m/s)
}

#[derive(Component, Debug, Clone)]
pub struct EnginePhysics {
    // Engine characteristics
    pub max_torque: f32,           // Maximum engine torque (Nm)
    pub power_curve: [f32; 8],     // Power curve samples (0-100% RPM)
    pub idle_rpm: f32,             // Idle RPM
    pub max_rpm: f32,              // Maximum RPM
    pub current_rpm: f32,          // Current RPM
    
    // Transmission
    pub gear_ratios: Vec<f32>,     // Gear ratios (including reverse)
    pub current_gear: i8,          // Current gear (-1 = reverse, 0 = neutral, 1+ = forward)
    pub differential_ratio: f32,   // Final drive ratio
    
    // Engine state
    pub throttle_input: f32,       // 0.0 - 1.0
    pub brake_input: f32,          // 0.0 - 1.0
    pub clutch_engagement: f32,    // 0.0 - 1.0 (for manual transmission)
    pub engine_temp: f32,          // Engine temperature
}

#[derive(Component, Debug, Clone)]
pub struct VehicleWheel {
    pub index: usize,              // Wheel index (0-3 for cars)
    pub position: Vec3,            // Local position relative to vehicle
    pub steering_angle: f32,       // Current steering angle (radians)
    pub max_steering_angle: f32,   // Maximum steering angle
    pub is_drive_wheel: bool,      // Whether this wheel receives engine power
    pub is_brake_wheel: bool,      // Whether this wheel receives brake force
    
    // Wheel state
    pub angular_velocity: f32,     // Wheel rotation speed (rad/s)
    pub radius: f32,               // Wheel radius (meters)
    pub width: f32,                // Tire width (meters)
}

#[derive(Component, Debug, Clone)]
pub struct RealisticVehicle {
    // Vehicle type classification
    pub vehicle_type: RealisticVehicleType,
    
    // Performance parameters
    pub steering_sensitivity: f32,  // Steering response
    pub stability_control: bool,    // Electronic stability control
    pub abs_enabled: bool,          // Anti-lock braking system
    pub traction_control: bool,     // Traction control system
    
    // Damage and wear
    pub engine_damage: f32,         // 0.0 - 1.0 (0 = perfect, 1 = destroyed)
    pub body_damage: f32,           // Body damage affecting aerodynamics
    
    // Performance monitoring
    pub last_update_time: f32,      // For delta time calculations
    pub physics_enabled: bool,      // Can disable physics for LOD
}

#[derive(Debug, Clone, PartialEq)]
pub enum RealisticVehicleType {
    BasicCar,
    Truck,
    Motorcycle,
    SUV,
}

impl Default for VehicleSuspension {
    fn default() -> Self {
        Self {
            spring_strength: 25000.0,
            damping_ratio: 0.6,
            max_compression: 0.3,
            rest_length: 0.5,
            compression: 0.0,
            velocity: 0.0,
            force: 0.0,
            wheel_position: Vec3::ZERO,
            ground_contact: false,
            ground_normal: Vec3::Y,
        }
    }
}

impl Default for TirePhysics {
    fn default() -> Self {
        Self {
            dry_grip: 1.2,
            wet_grip: 0.8,
            lateral_grip: 1.0,
            rolling_resistance: 0.015,
            slip_ratio: 0.0,
            slip_angle: 0.0,
            tire_temperature: 20.0,
            wear_level: 1.0,
            longitudinal_force: 0.0,
            lateral_force: 0.0,
            normal_force: 0.0,
        }
    }
}

impl Default for VehicleDynamics {
    fn default() -> Self {
        Self {
            total_mass: 1200.0,
            front_weight_ratio: 0.6,
            center_of_gravity: Vec3::new(0.0, 0.3, 0.1),
            inertia_tensor: Vec3::new(800.0, 1500.0, 1000.0),
            drag_coefficient: 0.35,
            frontal_area: 2.2,
            downforce_coefficient: 0.1,
            weight_transfer: Vec3::ZERO,
            aerodynamic_force: Vec3::ZERO,
            speed: 0.0,
        }
    }
}

impl Default for EnginePhysics {
    fn default() -> Self {
        Self {
            max_torque: 200.0,
            power_curve: [0.3, 0.5, 0.7, 0.85, 1.0, 0.95, 0.8, 0.6], // Typical curve
            idle_rpm: 800.0,
            max_rpm: 6500.0,
            current_rpm: 800.0,
            gear_ratios: vec![-3.0, 3.5, 2.0, 1.3, 1.0, 0.8], // R, 1, 2, 3, 4, 5
            current_gear: 0,
            differential_ratio: 3.9,
            throttle_input: 0.0,
            brake_input: 0.0,
            clutch_engagement: 1.0,
            engine_temp: 90.0,
        }
    }
}

impl Default for VehicleWheel {
    fn default() -> Self {
        Self {
            index: 0,
            position: Vec3::ZERO,
            steering_angle: 0.0,
            max_steering_angle: 0.6, // ~35 degrees
            is_drive_wheel: true,
            is_brake_wheel: true,
            angular_velocity: 0.0,
            radius: 0.35,
            width: 0.2,
        }
    }
}

impl Default for RealisticVehicle {
    fn default() -> Self {
        Self {
            vehicle_type: RealisticVehicleType::BasicCar,
            steering_sensitivity: 1.0,
            stability_control: true,
            abs_enabled: true,
            traction_control: true,
            engine_damage: 0.0,
            body_damage: 0.0,
            last_update_time: 0.0,
            physics_enabled: true,
        }
    }
}

/// CRITICAL VALIDATION FUNCTIONS - Prevent physics explosions
impl VehicleSuspension {
    pub fn validate_and_clamp(&mut self) {
        self.spring_strength = self.spring_strength.clamp(5000.0, 100000.0);
        self.damping_ratio = self.damping_ratio.clamp(0.1, 1.0);
        self.max_compression = self.max_compression.clamp(0.05, 1.0);
        self.rest_length = self.rest_length.clamp(0.2, 1.5);
        self.compression = self.compression.clamp(0.0, self.max_compression);
        self.velocity = self.velocity.clamp(-50.0, 50.0);
        self.force = self.force.clamp(-50000.0, 50000.0);
    }
}

impl TirePhysics {
    pub fn validate_and_clamp(&mut self) {
        self.dry_grip = self.dry_grip.clamp(0.5, 2.0);
        self.wet_grip = self.wet_grip.clamp(0.3, 1.5);
        self.lateral_grip = self.lateral_grip.clamp(0.5, 2.0);
        self.rolling_resistance = self.rolling_resistance.clamp(0.005, 0.1);
        self.slip_ratio = self.slip_ratio.clamp(-1.0, 1.0);
        self.slip_angle = self.slip_angle.clamp(-1.57, 1.57); // ±90 degrees
        self.tire_temperature = self.tire_temperature.clamp(-40.0, 200.0);
        self.wear_level = self.wear_level.clamp(0.1, 1.0);
        
        // Clamp forces to prevent instability
        self.longitudinal_force = self.longitudinal_force.clamp(-10000.0, 10000.0);
        self.lateral_force = self.lateral_force.clamp(-10000.0, 10000.0);
        self.normal_force = self.normal_force.clamp(0.0, 20000.0);
    }
}

impl VehicleDynamics {
    pub fn validate_and_clamp(&mut self) {
        self.total_mass = self.total_mass.clamp(200.0, 50000.0);
        self.front_weight_ratio = self.front_weight_ratio.clamp(0.3, 0.8);
        
        // Clamp center of gravity to reasonable bounds
        self.center_of_gravity.x = self.center_of_gravity.x.clamp(-2.0, 2.0);
        self.center_of_gravity.y = self.center_of_gravity.y.clamp(0.1, 2.0);
        self.center_of_gravity.z = self.center_of_gravity.z.clamp(-2.0, 2.0);
        
        // Clamp inertia values
        self.inertia_tensor.x = self.inertia_tensor.x.clamp(100.0, 10000.0);
        self.inertia_tensor.y = self.inertia_tensor.y.clamp(100.0, 10000.0);
        self.inertia_tensor.z = self.inertia_tensor.z.clamp(100.0, 10000.0);
        
        self.drag_coefficient = self.drag_coefficient.clamp(0.1, 2.0);
        self.frontal_area = self.frontal_area.clamp(0.5, 10.0);
        self.downforce_coefficient = self.downforce_coefficient.clamp(0.0, 5.0);
        
        self.speed = self.speed.clamp(0.0, 200.0); // Max 200 m/s (720 km/h)
    }
}

impl EnginePhysics {
    pub fn validate_and_clamp(&mut self) {
        self.max_torque = self.max_torque.clamp(50.0, 2000.0);
        self.idle_rpm = self.idle_rpm.clamp(400.0, 2000.0);
        self.max_rpm = self.max_rpm.clamp(2000.0, 12000.0);
        self.current_rpm = self.current_rpm.clamp(0.0, self.max_rpm * 1.2);
        self.differential_ratio = self.differential_ratio.clamp(1.0, 8.0);
        
        self.throttle_input = self.throttle_input.clamp(0.0, 1.0);
        self.brake_input = self.brake_input.clamp(0.0, 1.0);
        self.clutch_engagement = self.clutch_engagement.clamp(0.0, 1.0);
        self.engine_temp = self.engine_temp.clamp(-40.0, 200.0);
        
        // Validate gear ratios
        for ratio in &mut self.gear_ratios {
            *ratio = ratio.clamp(-5.0, 5.0);
        }
        
        // Clamp current gear to valid range
        let max_gear = self.gear_ratios.len() as i8 - 2; // Excluding reverse
        self.current_gear = self.current_gear.clamp(-1, max_gear);
    }
}

impl VehicleWheel {
    pub fn validate_and_clamp(&mut self) {
        self.steering_angle = self.steering_angle.clamp(-1.57, 1.57); // ±90 degrees
        self.max_steering_angle = self.max_steering_angle.clamp(0.1, 1.57);
        self.angular_velocity = self.angular_velocity.clamp(-500.0, 500.0);
        self.radius = self.radius.clamp(0.1, 1.0);
        self.width = self.width.clamp(0.05, 0.5);
    }
}

impl RealisticVehicle {
    pub fn validate_and_clamp(&mut self) {
        self.steering_sensitivity = self.steering_sensitivity.clamp(0.1, 5.0);
        self.engine_damage = self.engine_damage.clamp(0.0, 1.0);
        self.body_damage = self.body_damage.clamp(0.0, 1.0);
    }
}
