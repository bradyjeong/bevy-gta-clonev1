use bevy::prelude::*;

/// ECS-first vehicle control state component
///
/// This component stores the current control inputs for any controllable entity.
/// It replaces global state maps with per-entity data, following ECS principles.
///
/// ## Design Principles
/// - Per-entity: No global state or entity lookups required
/// - Pure data: No methods or logic, just normalized input values
/// - Unified: Works for player input, AI control, and testing
/// - Simple: Single responsibility - store current control state
#[derive(Component, Default, Debug, Clone)]
pub struct ControlState {
    /// Throttle input: 0.0 = no acceleration, 1.0 = full acceleration
    pub throttle: f32,

    /// Brake input: 0.0 = no braking, 1.0 = full braking (slows down)
    pub brake: f32,

    /// Reverse input: 0.0 = no reverse, 1.0 = full reverse
    pub reverse: f32,

    /// Steering input: -1.0 = full left, 0.0 = straight, 1.0 = full right
    pub steering: f32,

    /// Vertical control: -1.0 = down/descend, 0.0 = neutral, 1.0 = up/ascend
    /// Used for helicopters, aircraft, and swimming
    pub vertical: f32,

    /// Yaw control: -1.0 = left, 0.0 = neutral, 1.0 = right
    /// Used for aircraft rudder control and fine turning
    pub yaw: f32,

    /// Roll control: -1.0 = roll left, 0.0 = neutral, 1.0 = roll right
    /// Used for aircraft banking and advanced vehicle control
    pub roll: f32,

    /// Pitch control: -1.0 = pitch down, 0.0 = neutral, 1.0 = pitch up
    /// Used for aircraft and submarine control
    pub pitch: f32,

    /// Turbo/afterburner/nitrous activation: 0.0 = off, 1.0 = full boost
    pub boost: f32,

    /// Emergency brake flag: instant maximum braking
    pub emergency_brake: bool,

    /// Interaction flag: enter/exit vehicle, interact with objects
    pub interact: bool,

    /// Running/sprint modifier for walking
    pub run: bool,
}

impl ControlState {
    /// Create a new ControlState with all inputs set to neutral/off
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all control inputs to neutral/off position
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Check if any movement control is active
    pub fn has_movement_input(&self) -> bool {
        self.throttle > 0.0 || self.brake > 0.0 || self.reverse > 0.0 || self.steering.abs() > 0.0
    }

    /// Check if any flight control is active (pitch/roll/yaw)
    pub fn has_flight_input(&self) -> bool {
        self.pitch.abs() > 0.0
            || self.roll.abs() > 0.0
            || self.yaw.abs() > 0.0
            || self.vertical.abs() > 0.0
    }

    /// Check if boost is active
    pub fn is_boosting(&self) -> bool {
        self.boost > 0.0
    }

    /// Check if braking (regular brake or emergency brake)
    pub fn is_braking(&self) -> bool {
        self.brake > 0.0 || self.emergency_brake
    }

    /// Check if reversing
    pub fn is_reversing(&self) -> bool {
        self.reverse > 0.0
    }

    /// Check if accelerating
    pub fn is_accelerating(&self) -> bool {
        self.throttle > 0.0
    }

    /// Validate and clamp all inputs to safe ranges
    pub fn validate_and_clamp(&mut self) {
        self.throttle = self.throttle.clamp(0.0, 1.0);
        self.brake = self.brake.clamp(0.0, 1.0);
        self.reverse = self.reverse.clamp(0.0, 1.0);
        self.steering = self.steering.clamp(-1.0, 1.0);
        self.vertical = self.vertical.clamp(-1.0, 1.0);
        self.yaw = self.yaw.clamp(-1.0, 1.0);
        self.roll = self.roll.clamp(-1.0, 1.0);
        self.pitch = self.pitch.clamp(-1.0, 1.0);
        self.boost = self.boost.clamp(0.0, 1.0);
    }

    /// Apply smooth input filtering to reduce jitter
    pub fn apply_smoothing(&mut self, dt: f32, smoothing_factor: f32) {
        // Validate inputs
        if !dt.is_finite() || dt <= 0.0 || dt > 1.0 {
            warn!("Invalid dt in apply_smoothing: {}", dt);
            return;
        }
        if !smoothing_factor.is_finite() || smoothing_factor < 0.0 {
            warn!("Invalid smoothing_factor: {}", smoothing_factor);
            return;
        }

        let factor = 1.0 - (-dt * smoothing_factor).exp();
        let factor = factor.clamp(0.0, 1.0); // Safety clamp

        // Only smooth analog inputs, not digital flags
        self.throttle *= 1.0 - factor;
        self.brake *= 1.0 - factor;
        self.reverse *= 1.0 - factor;
        self.steering *= 1.0 - factor;
        self.vertical *= 1.0 - factor;
        self.yaw *= 1.0 - factor;
        self.roll *= 1.0 - factor;
        self.pitch *= 1.0 - factor;
        self.boost *= 1.0 - factor;
    }
}

/// Marker component for entities that can be controlled by player input
#[derive(Component, Default, Debug)]
pub struct PlayerControlled;

/// Marker component for entities controlled by AI
#[derive(Component, Default, Debug)]
pub struct AIControlled;

/// Marker for player that needs physics re-enabled next frame (safe vehicle exit)
#[derive(Component)]
pub struct PendingPhysicsEnable;

/// Component that tracks which vehicle type this entity represents
/// Used for determining control mappings and physics parameters
#[derive(
    Component, Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum VehicleControlType {
    Walking,
    Swimming,
    Car,
    Helicopter,
    F16,
    Yacht,
}

impl Default for VehicleControlType {
    fn default() -> Self {
        Self::Walking
    }
}

impl VehicleControlType {
    /// Get a human-readable name for the vehicle type
    pub fn name(&self) -> &'static str {
        match self {
            Self::Walking => "Walking",
            Self::Swimming => "Swimming",
            Self::Car => "Car",
            Self::Helicopter => "Helicopter",
            Self::F16 => "F16 Fighter Jet",
            Self::Yacht => "Yacht",
        }
    }

    /// Check if this vehicle type can use boost/turbo
    pub fn has_boost(&self) -> bool {
        matches!(self, Self::Car | Self::F16 | Self::Yacht)
    }

    /// Check if this vehicle type uses flight controls
    pub fn uses_flight_controls(&self) -> bool {
        matches!(self, Self::Helicopter | Self::F16)
    }

    /// Check if this vehicle type uses ground vehicle controls
    pub fn uses_ground_controls(&self) -> bool {
        matches!(
            self,
            Self::Walking | Self::Swimming | Self::Car | Self::Yacht
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_state_default() {
        let control = ControlState::default();

        assert_eq!(control.throttle, 0.0);
        assert_eq!(control.brake, 0.0);
        assert_eq!(control.steering, 0.0);
        assert!(!control.emergency_brake);
        assert!(!control.interact);
    }

    #[test]
    fn test_control_state_validation() {
        let mut control = ControlState {
            throttle: 2.0,  // Over limit
            steering: -2.0, // Under limit
            brake: 0.5,     // Valid
            ..Default::default()
        };

        control.validate_and_clamp();

        assert_eq!(control.throttle, 1.0);
        assert_eq!(control.steering, -1.0);
        assert_eq!(control.brake, 0.5);
    }

    #[test]
    fn test_movement_detection() {
        let mut control = ControlState::default();
        assert!(!control.has_movement_input());

        control.throttle = 0.5;
        assert!(control.has_movement_input());
        assert!(control.is_accelerating());

        control.throttle = 0.0;
        control.brake = 0.3;
        assert!(control.has_movement_input());
        assert!(control.is_braking());
    }

    #[test]
    fn test_flight_controls() {
        let mut control = ControlState::default();
        assert!(!control.has_flight_input());

        control.pitch = 0.5;
        assert!(control.has_flight_input());

        control.pitch = 0.0;
        control.roll = -0.3;
        assert!(control.has_flight_input());
    }

    #[test]
    fn test_vehicle_type_properties() {
        assert!(VehicleControlType::F16.has_boost());
        assert!(VehicleControlType::F16.uses_flight_controls());
        assert!(!VehicleControlType::F16.uses_ground_controls());

        assert!(VehicleControlType::Car.has_boost());
        assert!(!VehicleControlType::Car.uses_flight_controls());
        assert!(VehicleControlType::Car.uses_ground_controls());

        assert!(!VehicleControlType::Walking.has_boost());
        assert!(!VehicleControlType::Walking.uses_flight_controls());
        assert!(VehicleControlType::Walking.uses_ground_controls());
    }
}
