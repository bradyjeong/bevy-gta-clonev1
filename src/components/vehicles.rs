//! # Vehicle Components - Asset-Driven Architecture
//!
//! This module defines all vehicle-related components following AGENT.md "simplicity first" principles.
//! The architecture prioritizes clean, asset-driven configuration over complex runtime components.
//!
//! ## Essential Components
//!
//! ### Marker Components:
//! - [`Car`] - Basic car marker
//! - [`Helicopter`] - Helicopter marker  
//! - [`F16`] - F16 fighter jet marker
//!
//! ### Asset-Driven Specs:
//! - [`SimpleCarSpecs`] - Car physics loaded from RON files
//! - [`SimpleHelicopterSpecs`] - Helicopter physics loaded from RON files
//! - [`SimpleF16Specs`] - F16 physics loaded from RON files
//!
//! ### Core Vehicle System:
//! - [`VehicleType`] - Vehicle classification enum
//! - [`VehicleState`] - Lightweight runtime state
//! - [`VehicleRendering`] - LOD rendering management
//! - [`AircraftFlight`] - Minimal flight state for aircraft

use bevy::prelude::*;
use bevy::reflect::TypePath;

// Essential marker components
#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct Helicopter;

#[derive(Component)]
pub struct F16;

// Phase 2: Ground detection for car stability
#[derive(Component, Clone, Debug, Default)]
pub struct Grounded {
    pub is_grounded: bool,
    pub ground_distance: f32,
}

// Phase 3: Visual-only body lean component (cosmetic rotation, no physics impact)
#[derive(Component, Clone, Debug, Default)]
pub struct VisualRig {
    pub current_roll: f32,   // Current visual roll angle (radians)
    pub current_pitch: f32,  // Current visual pitch angle (radians)
    pub roll_velocity: f32,  // Roll angular velocity for spring-damper
    pub pitch_velocity: f32, // Pitch angular velocity for spring-damper
    pub last_velocity: Vec3, // Previous frame velocity for acceleration calculation
}

// Phase 3: Visual rig root - single child that receives visual rotation
#[derive(Component)]
pub struct VisualRigRoot;

// Vehicle health component for boundary effects and damage
#[derive(Component, Clone, Debug)]
pub struct VehicleHealth {
    pub current: f32,
    pub max: f32,
}

impl Default for VehicleHealth {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

impl VehicleHealth {
    pub fn new(max_health: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.current <= 0.0
    }

    pub fn health_percentage(&self) -> f32 {
        (self.current / self.max).clamp(0.0, 1.0)
    }
}

// Ultra-simplified aircraft flight state - minimal necessary data
#[derive(Component, Clone)]
pub struct AircraftFlight {
    // Engine state only (eliminate derived data)
    pub throttle: f32, // 0.0-1.0, processed from controls
    pub airspeed: f32, // For UI/debugging only
    pub afterburner_active: bool,
}

// Simplified F16 specifications - all tuning constants data-driven
// Asset-driven configuration following YachtSpecs pattern
#[derive(Asset, TypePath, Component, Clone, serde::Deserialize)]
pub struct SimpleF16Specs {
    pub max_forward_speed: f32, // Maximum forward velocity (m/s)
    pub roll_rate_max: f32,     // Maximum roll rate (rad/s)
    pub pitch_rate_max: f32,    // Maximum pitch rate (rad/s)
    pub yaw_rate_max: f32,      // Maximum yaw rate (rad/s)
    pub throttle_increase_rate: f32,
    pub throttle_decrease_rate: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub lift_per_throttle: f32,
    pub drag_factor: f32, // Momentum decay when engine off

    // Previously magic numbers in code
    pub afterburner_multiplier: f32, // Speed multiplier for afterburner
    pub linear_lerp_factor: f32,     // Linear velocity smoothing rate
    pub angular_lerp_factor: f32,    // Angular velocity smoothing rate
    pub throttle_deadzone: f32,      // Minimum throttle for lift activation

    // GTA-style input shaping and discrete step rates
    pub input_deadzone: f32, // Input threshold below which it's treated as zero
    pub input_step_threshold: f32, // Input magnitude threshold for min/max rate selection
    pub roll_rate_min: f32,  // Minimum roll rate for small inputs
    pub pitch_rate_min: f32, // Minimum pitch rate for small inputs
    pub yaw_rate_min: f32,   // Minimum yaw rate for small inputs

    // Speed-based control effectiveness
    pub control_full_speed: f32, // Speed at which control effectiveness reaches 1.0
    pub min_control_factor: f32, // Minimum control effectiveness at zero speed

    // GTA-style auto-stabilization
    pub roll_stab: f32, // Multiplicative angular velocity damping (0.9 = moderate)
    pub pitch_stab: f32, // Higher = more stable, lower = more agile
    pub yaw_stab: f32,  // Usually highest for yaw stability
    pub roll_auto_level_gain: f32, // Additive horizon leveling (rad/s per unit tilt)
    pub pitch_auto_level_gain: f32, // Auto pitch toward horizon
    pub yaw_auto_level_gain: f32, // Lateral slip correction gain

    // Auto banking from lateral velocity
    pub auto_bank_gain: f32,     // Roll rate per m/s lateral velocity
    pub auto_bank_max_rate: f32, // Maximum auto-bank contribution

    // Optional banked-lift feedback
    pub bank_lift_scale: f32, // How much lift reduces when banked (0-1)
}

impl Default for AircraftFlight {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            airspeed: 0.0,
            afterburner_active: false,
        }
    }
}

impl Default for SimpleF16Specs {
    fn default() -> Self {
        Self {
            max_forward_speed: 200.0_f32.clamp(50.0, 500.0), // m/s - realistic fighter jet speed
            roll_rate_max: 6.3_f32.clamp(0.1, 10.0),         // rad/s - prevent excessive rotation
            pitch_rate_max: 3.5_f32.clamp(0.1, 10.0),        // rad/s
            yaw_rate_max: 1.05_f32.clamp(0.1, 5.0),          // rad/s
            throttle_increase_rate: 2.0_f32.clamp(0.1, 10.0),
            throttle_decrease_rate: 3.0_f32.clamp(0.1, 10.0),
            linear_damping: 0.15_f32.clamp(0.01, 5.0),
            angular_damping: 0.05_f32.clamp(0.01, 5.0),
            lift_per_throttle: 3.0_f32.clamp(0.1, 50.0),
            drag_factor: 0.995_f32.clamp(0.9, 1.0), // Momentum decay per second when engine off

            // Formerly magic numbers - with safety limits
            afterburner_multiplier: 1.5_f32.clamp(1.0, 3.0), // Speed multiplier for afterburner
            linear_lerp_factor: 4.0_f32.clamp(1.0, 20.0),    // Linear velocity smoothing
            angular_lerp_factor: 8.0_f32.clamp(1.0, 20.0),   // Angular velocity smoothing
            throttle_deadzone: 0.1_f32.clamp(0.0, 0.5),      // Minimum throttle for lift

            // GTA-style input shaping
            input_deadzone: 0.10_f32.clamp(0.0, 0.3),
            input_step_threshold: 0.45_f32.clamp(0.0, 1.0),
            roll_rate_min: 3.0_f32.clamp(0.1, 10.0),
            pitch_rate_min: 1.6_f32.clamp(0.1, 10.0),
            yaw_rate_min: 0.4_f32.clamp(0.1, 5.0),

            // Speed-based control effectiveness
            control_full_speed: 120.0_f32.clamp(10.0, 500.0),
            min_control_factor: 0.40_f32.clamp(0.1, 1.0),

            // GTA-style auto-stabilization
            roll_stab: 0.90_f32.clamp(0.5, 1.0),
            pitch_stab: 0.95_f32.clamp(0.5, 1.0),
            yaw_stab: 0.98_f32.clamp(0.5, 1.0),
            roll_auto_level_gain: 3.0_f32.clamp(0.0, 10.0),
            pitch_auto_level_gain: 2.5_f32.clamp(0.0, 10.0),
            yaw_auto_level_gain: 0.8_f32.clamp(0.0, 5.0),

            // Auto banking
            auto_bank_gain: 0.02_f32.clamp(0.0, 0.1),
            auto_bank_max_rate: 2.0_f32.clamp(0.0, 10.0),

            // Banked-lift feedback
            bank_lift_scale: 0.7_f32.clamp(0.0, 1.0),
        }
    }
}

#[derive(Component)]
pub struct MainRotor;

#[derive(Component)]
pub struct TailRotor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum HeliState {
    Grounded,
    Flying,
}

impl Default for HeliState {
    fn default() -> Self {
        Self::Grounded
    }
}

#[derive(Component, Reflect)]
pub struct HelicopterRuntime {
    pub rpm: f32,
    pub state: HeliState,
}

impl Default for HelicopterRuntime {
    fn default() -> Self {
        Self {
            rpm: 0.0,
            state: HeliState::Grounded,
        }
    }
}

#[derive(Component)]
pub struct RotorBlurDisk {
    pub min_rpm_for_blur: f32,
    pub is_main_rotor: bool,
}

impl Default for RotorBlurDisk {
    fn default() -> Self {
        Self {
            min_rpm_for_blur: 10.0,
            is_main_rotor: true,
        }
    }
}

// NEW LOD SYSTEM

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum VehicleType {
    SuperCar,
    Helicopter,
    F16,
    Yacht,
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
            VehicleType::SuperCar => (70.0, 40.0),
            VehicleType::Helicopter => (83.0, 30.0),
            VehicleType::F16 => (600.0, 80.0),
            VehicleType::Yacht => (30.0, 25.0), // TODO: Read from loaded YachtSpecs asset in Phase 2
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

// Simple vehicle physics configurations (asset-driven)
// Phase 1: Visual wheel system components
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelPos {
    FL, // Front Left
    FR, // Front Right
    RL, // Rear Left
    RR, // Rear Right
}

#[derive(Component)]
pub struct WheelMesh {
    pub pos: WheelPos,
    pub radius: f32,
    pub roll_angle: f32,
    pub roll_dir: f32, // 1.0 or -1.0 for axis correction
}

#[derive(Component)]
pub struct WheelSteerPivot {
    pub pos: WheelPos,
}

#[derive(Component)]
pub struct WheelsRoot;

#[derive(Component)]
pub struct CarWheelsConfig {
    pub max_steer_rad: f32,
    pub wheel_radius: f32,
}

// Asset-driven configuration following YachtSpecs pattern
#[derive(Asset, TypePath, Component, Debug, Clone, serde::Deserialize)]
pub struct SimpleCarSpecs {
    pub base_speed: f32,
    pub rotation_speed: f32,
    pub linear_lerp_factor: f32,
    pub angular_lerp_factor: f32,
    pub emergency_brake_linear: f32,
    pub emergency_brake_angular: f32,
    pub drag_factor: f32, // Momentum decay when no input

    // New arcade physics fields
    pub accel_lerp: f32,       // Acceleration smoothing rate
    pub brake_lerp: f32,       // Braking smoothing rate
    pub grip: f32,             // Lateral grip (higher = sticks better)
    pub drift_grip: f32,       // Grip during emergency brake (lower = drifts)
    pub steer_gain: f32,       // Base steering responsiveness
    pub steer_speed_drop: f32, // How much steering decreases with speed
    pub stability: f32,        // Auto-straightening torque (higher = more stable)
    pub ebrake_yaw_boost: f32, // Extra yaw when e-braking
    pub downforce_scale: f32,  // Grip increase at high speeds

    // Phase 1 GTA-style helpers
    pub auto_brake_gain: f32, // Auto-brake strength when throttle opposes velocity
    pub slip_extremum: f32,   // Slip ratio for maximum grip
    pub slip_asymptote: f32,  // Slip ratio for full slide
    pub slip_stiffness: f32,  // Overall slip curve scale
    pub brake_grip_loss: f32, // Lateral grip reduction during heavy braking

    // Phase 2: Stability helpers
    pub ground_ray_length: f32,    // Raycast distance for ground detection
    pub air_gravity_scale: f32,    // Extra downward force when airborne
    pub airborne_steer_scale: f32, // Steering reduction when airborne (0-1)
    pub roll_kp: f32,              // Roll correction proportional gain
    pub roll_kd: f32,              // Roll correction derivative gain
    pub roll_torque_limit: f32,    // Maximum roll correction torque

    // Phase 3: Drivability polish
    pub reverse_steer_invert: bool, // Invert steering when reversing (realistic)
    pub airborne_angular_scale: f32, // Angular velocity scale when airborne (0-1)
    pub visual_roll_gain: f32,      // Visual body lean gain (radians per m/s² lateral)
    pub visual_pitch_gain: f32,     // Visual body pitch gain (radians per m/s² longitudinal)
    pub visual_spring: f32,         // Visual rig spring constant
    pub visual_damper: f32,         // Visual rig damping constant

    // Phase 1: Visual wheel system
    pub max_steer_deg: f32,
    pub wheel_radius: f32,
    pub wheel_positions: [(f32, f32, f32); 4], // FL, FR, RL, RR positions
}

impl Default for SimpleCarSpecs {
    fn default() -> Self {
        Self {
            base_speed: 70.0_f32.clamp(1.0, 100.0), // m/s - super car speeds
            rotation_speed: 3.0_f32.clamp(0.1, 10.0), // rad/s - super cars turn faster
            linear_lerp_factor: 4.0_f32.clamp(1.0, 20.0), // Smooth movement response
            angular_lerp_factor: 6.0_f32.clamp(1.0, 20.0), // Smooth rotation response
            emergency_brake_linear: 0.1_f32.clamp(0.01, 1.0), // Multiplier - keep some movement
            emergency_brake_angular: 0.5_f32.clamp(0.01, 1.0), // Multiplier
            drag_factor: 0.92_f32.clamp(0.9, 1.0),  // Momentum decay per second when no input

            // New arcade physics defaults (ULTRA AGGRESSIVE!)
            accel_lerp: 5.0_f32.clamp(1.0, 20.0), // Acceleration smoothing
            brake_lerp: 8.0_f32.clamp(1.0, 20.0), // Braking smoothing - INCREASED
            grip: 8.0_f32.clamp(0.1, 50.0),       // Lateral grip
            drift_grip: 1.2_f32.clamp(0.1, 50.0), // Grip during e-brake - VERY LOW
            steer_gain: 8.0_f32.clamp(0.1, 20.0), // Steering responsiveness - 8.0 ULTRA AGGRESSIVE!
            steer_speed_drop: 0.015_f32.clamp(0.0, 1.0), // Speed steering reduction - MINIMAL
            stability: 0.3_f32.clamp(0.0, 5.0),   // Auto-straightening - VERY LOW
            ebrake_yaw_boost: 2.0_f32.clamp(0.0, 5.0), // E-brake yaw boost - MASSIVE
            downforce_scale: 0.3_f32.clamp(0.0, 2.0), // High-speed grip boost

            // Phase 1 GTA-style helpers
            auto_brake_gain: 12.0_f32.clamp(1.0, 50.0), // Auto-brake when throttle opposes velocity
            slip_extremum: 0.5_f32.clamp(0.1, 5.0),     // Peak grip at 0.5 slip ratio
            slip_asymptote: 20.0_f32.clamp(1.0, 50.0),  // Full slide at 20.0 slip
            slip_stiffness: 1.0_f32.clamp(0.1, 5.0),    // Overall slip curve scale
            brake_grip_loss: 0.25_f32.clamp(0.0, 1.0),  // 25% grip loss during heavy braking

            // Phase 2: Stability helpers
            ground_ray_length: 1.2_f32.clamp(0.5, 5.0),
            air_gravity_scale: 1.5_f32.clamp(0.0, 10.0),
            airborne_steer_scale: 0.3_f32.clamp(0.0, 1.0),
            roll_kp: 6.0_f32.clamp(0.0, 20.0),
            roll_kd: 1.5_f32.clamp(0.0, 10.0),
            roll_torque_limit: 600.0_f32.clamp(0.0, 2000.0),

            // Phase 3: Drivability polish defaults
            reverse_steer_invert: true,
            airborne_angular_scale: 0.5_f32.clamp(0.0, 1.0),
            visual_roll_gain: 0.02_f32.clamp(0.0, 0.1),
            visual_pitch_gain: 0.01_f32.clamp(0.0, 0.1),
            visual_spring: 12.0_f32.clamp(1.0, 50.0),
            visual_damper: 2.5_f32.clamp(0.1, 10.0),

            // Phase 1: Visual wheel system defaults
            max_steer_deg: 28.0_f32.clamp(10.0, 45.0),
            wheel_radius: 0.33_f32.clamp(0.1, 1.0),
            wheel_positions: [
                (0.85, -0.32, -1.40),  // FL (front = negative Z in Bevy Z-forward)
                (-0.85, -0.32, -1.40), // FR
                (0.85, -0.32, 1.40),   // RL (rear = positive Z)
                (-0.85, -0.32, 1.40),  // RR
            ],
        }
    }
}

// Asset-driven configuration following YachtSpecs pattern
#[derive(Asset, TypePath, Component, Debug, Clone, serde::Deserialize)]
pub struct SimpleHelicopterSpecs {
    pub vertical_speed: f32,
    pub yaw_rate: f32,
    pub pitch_rate: f32,
    pub roll_rate: f32,
    pub angular_lerp_factor: f32,
    pub main_rotor_rpm: f32,
    pub tail_rotor_rpm: f32,
    pub spool_up_rate: f32,
    pub spool_down_rate: f32,
    pub min_rpm_for_lift: f32,
    pub rpm_to_lift_exp: f32,
    pub max_lift_margin_g: f32,
    pub cyclic_tilt_max_deg: f32,
    pub horiz_drag: f32,
    pub damage_authority_min: f32,
    pub rotor_wash_scale: f32,
    pub hover_bias: f32,      // Lift bias above weight to ensure liftoff
    pub collective_gain: f32, // Collective control sensitivity
    pub input_deadzone: f32,  // Input deadzone for all axes

    // GTA SA-style per-axis stabilization damping (applied as stab.powf(dt) when input neutral)
    pub pitch_stab: f32, // Pitch axis multiplicative damping (0.5-1.0, ~0.97 typical)
    pub roll_stab: f32,  // Roll axis multiplicative damping (0.5-1.0, ~0.96 typical)
    pub yaw_stab: f32,   // Yaw axis multiplicative damping (0.5-1.0, ~0.98 typical)

    // Ground detection
    pub ground_ray_length: f32, // Raycast length for ground detection (m)
}

impl Default for SimpleHelicopterSpecs {
    fn default() -> Self {
        Self {
            vertical_speed: 15.0_f32.clamp(1.0, 50.0), // m/s - vertical flight limits
            yaw_rate: 1.5_f32.clamp(0.1, 5.0),         // rad/s - prevent excessive rotation
            pitch_rate: 1.0_f32.clamp(0.1, 5.0),       // rad/s
            roll_rate: 1.0_f32.clamp(0.1, 5.0),        // rad/s
            angular_lerp_factor: 4.0_f32.clamp(1.0, 20.0), // Smooth control response
            main_rotor_rpm: 20.0_f32.clamp(1.0, 100.0), // rad/s - main rotor speed
            tail_rotor_rpm: 35.0_f32.clamp(1.0, 100.0), // rad/s - tail rotor speed
            spool_up_rate: 0.6,
            spool_down_rate: 0.3,
            min_rpm_for_lift: 0.35,
            rpm_to_lift_exp: 1.7,
            max_lift_margin_g: 1.8,
            cyclic_tilt_max_deg: 18.0,
            horiz_drag: 1.0,
            damage_authority_min: 0.3,
            rotor_wash_scale: 1.0,
            hover_bias: 0.03,
            collective_gain: 0.55,
            input_deadzone: 0.10,

            // GTA SA-style per-axis stabilization
            pitch_stab: 0.97_f32.clamp(0.5, 1.0),
            roll_stab: 0.96_f32.clamp(0.5, 1.0),
            yaw_stab: 0.98_f32.clamp(0.5, 1.0),

            // Ground detection
            ground_ray_length: 5.0,
        }
    }
}

// Asset handle components for asset-driven vehicle specs
// Following YachtSpecsHandle pattern from simple_yacht.rs
#[derive(Component)]
pub struct SimpleCarSpecsHandle(pub Handle<SimpleCarSpecs>);

#[derive(Component)]
pub struct SimpleHelicopterSpecsHandle(pub Handle<SimpleHelicopterSpecs>);

#[derive(Component)]
pub struct SimpleF16SpecsHandle(pub Handle<SimpleF16Specs>);
