use bevy::prelude::*;
use crate::components::{ControlState, PlayerControlled, VehicleControlType};
use crate::game_state::GameState;

/// Simple input mapping system that follows ECS principles
/// 
/// This system replaces the complex ControlManager with a focused approach:
/// - Reads keyboard input directly
/// - Writes to ControlState component on player-controlled entities
/// - No global state or entity lookups
/// - Single responsibility: map input to control state
pub fn simple_input_mapping_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut query: Query<(&mut ControlState, &VehicleControlType), With<PlayerControlled>>,
) {
    // Process all player-controlled entities
    for (mut control_state, vehicle_type) in query.iter_mut() {
        // Reset control state each frame
        control_state.reset();
        
        // Map input based on current game state and vehicle type
        match current_state.get() {
            GameState::Walking => {
                map_walking_controls(&keyboard_input, &mut control_state);
            }
            GameState::Driving => {
                map_driving_controls(&keyboard_input, &mut control_state, vehicle_type);
            }
            GameState::Flying => {
                map_helicopter_controls(&keyboard_input, &mut control_state);
            }
            GameState::Jetting => {
                map_f16_controls(&keyboard_input, &mut control_state);
            }
        }
        
        // Always validate inputs for safety
        control_state.validate_and_clamp();
    }
}

/// Map keyboard input for walking controls
fn map_walking_controls(keyboard_input: &ButtonInput<KeyCode>, control_state: &mut ControlState) {
    // Movement controls
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        control_state.throttle = 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        control_state.brake = 1.0; // Walking backward uses brake input
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        control_state.steering = 1.0;  // Turn left = positive rotation
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        control_state.steering = -1.0; // Turn right = negative rotation
    }
    
    // Sprint/run modifier
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        control_state.run = true;
    }
    
    // Interaction
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        control_state.interact = true;
    }
}

/// Map keyboard input for driving controls (Car/SuperCar)
fn map_driving_controls(
    keyboard_input: &ButtonInput<KeyCode>, 
    control_state: &mut ControlState,
    vehicle_type: &VehicleControlType,
) {
    // Acceleration
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        control_state.throttle = 1.0;
    }
    
    // Braking/Reverse
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        control_state.brake = 1.0;
    }
    
    // Steering
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        control_state.steering = 1.0;  // Turn left = positive rotation
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        control_state.steering = -1.0; // Turn right = negative rotation
    }
    
    // Turbo/Nitrous (only for vehicles that support it)
    if vehicle_type.has_boost() && keyboard_input.pressed(KeyCode::Space) {
        control_state.boost = 1.0;
    }
    
    // Exit vehicle
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        control_state.interact = true;
    }
    
    // Emergency brake (F2)
    if keyboard_input.just_pressed(KeyCode::F2) {
        control_state.emergency_brake = true;
    }
}

/// Map keyboard input for helicopter controls
fn map_helicopter_controls(keyboard_input: &ButtonInput<KeyCode>, control_state: &mut ControlState) {
    // Forward/backward pitch
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        control_state.pitch = 1.0; // Pitch forward
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        control_state.pitch = -1.0; // Pitch backward
    }
    
    // Left/right rotation (use steering for consistency with aircraft system)
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        control_state.steering = 1.0;  // Turn left = positive rotation
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        control_state.steering = -1.0; // Turn right = negative rotation
    }
    
    // Collective (vertical movement)
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        control_state.vertical = 1.0; // Ascend
    }
    if keyboard_input.pressed(KeyCode::ControlLeft) {
        control_state.vertical = -1.0; // Descend
    }
    
    // Exit helicopter
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        control_state.interact = true;
    }
    
    // Emergency systems
    if keyboard_input.just_pressed(KeyCode::F2) {
        control_state.emergency_brake = true;
    }
}

/// Map keyboard input for F16 fighter jet controls
fn map_f16_controls(keyboard_input: &ButtonInput<KeyCode>, control_state: &mut ControlState) {
    // Primary flight controls (Arrow keys)
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        control_state.pitch = 1.0; // Pitch up
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        control_state.pitch = -1.0; // Pitch down
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        control_state.roll = -1.0; // Roll left
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        control_state.roll = 1.0; // Roll right
    }
    
    // Throttle and yaw (WASD)
    if keyboard_input.pressed(KeyCode::KeyW) {
        control_state.throttle = 1.0; // Throttle up
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        control_state.brake = 1.0; // Throttle down (uses brake input)
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        control_state.yaw = 1.0; // Rudder left = positive rotation
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        control_state.yaw = -1.0; // Rudder right = negative rotation
    }
    
    // Afterburner
    if keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::ShiftLeft) {
        control_state.boost = 1.0;
    }
    
    // Exit F16
    if keyboard_input.just_pressed(KeyCode::KeyF) || keyboard_input.just_pressed(KeyCode::Enter) {
        control_state.interact = true;
    }
    
    // Emergency systems
    if keyboard_input.just_pressed(KeyCode::F2) {
        control_state.emergency_brake = true;
    }
}

/// System to apply input smoothing for more realistic control feel
pub fn input_smoothing_system(
    time: Res<Time>,
    mut query: Query<&mut ControlState, With<PlayerControlled>>,
) {
    let dt = time.delta_secs();
    let smoothing_factor = 5.0; // Adjust for feel
    
    for mut control_state in query.iter_mut() {
        control_state.apply_smoothing(dt, smoothing_factor);
    }
}

/// Debug system to display current control state
pub fn debug_control_state_system(
    query: Query<(&ControlState, &VehicleControlType), With<PlayerControlled>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Only show debug info when F1 is pressed
    if !keyboard_input.pressed(KeyCode::F1) {
        return;
    }
    
    for (control_state, vehicle_type) in query.iter() {
        info!(
            "CONTROL DEBUG [{}]: throttle={:.2}, brake={:.2}, steering={:.2}, boost={:.2}, interact={}",
            vehicle_type.name(),
            control_state.throttle,
            control_state.brake,
            control_state.steering,
            control_state.boost,
            control_state.interact
        );
        
        if vehicle_type.uses_flight_controls() {
            info!(
                "FLIGHT CONTROLS: pitch={:.2}, roll={:.2}, yaw={:.2}, vertical={:.2}",
                control_state.pitch,
                control_state.roll,
                control_state.yaw,
                control_state.vertical
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    fn create_test_input(pressed_keys: &[KeyCode]) -> ButtonInput<KeyCode> {
        let mut input = ButtonInput::default();
        for &key in pressed_keys {
            input.press(key);
        }
        input
    }
    
    #[test]
    fn test_walking_controls() {
        let input = create_test_input(&[KeyCode::ArrowUp, KeyCode::ShiftLeft]);
        let mut control_state = ControlState::default();
        
        map_walking_controls(&input, &mut control_state);
        
        assert_eq!(control_state.throttle, 1.0);
        assert!(control_state.run);
        assert_eq!(control_state.steering, 0.0);
    }
    
    #[test]
    fn test_driving_controls() {
        let input = create_test_input(&[KeyCode::ArrowUp, KeyCode::Space, KeyCode::ArrowLeft]);
        let mut control_state = ControlState::default();
        let vehicle_type = VehicleControlType::SuperCar;
        
        map_driving_controls(&input, &mut control_state, &vehicle_type);
        
        assert_eq!(control_state.throttle, 1.0);
        assert_eq!(control_state.boost, 1.0);
        assert_eq!(control_state.steering, 1.0); // Left arrow now maps to positive rotation
    }
    
    #[test]
    fn test_f16_controls() {
        let input = create_test_input(&[KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Space]);
        let mut control_state = ControlState::default();
        
        map_f16_controls(&input, &mut control_state);
        
        assert_eq!(control_state.pitch, 1.0);
        assert_eq!(control_state.throttle, 1.0);
        assert_eq!(control_state.boost, 1.0);
    }
    
    #[test]
    fn test_control_validation() {
        let input = create_test_input(&[KeyCode::ArrowUp]);
        let mut control_state = ControlState::default();
        
        map_walking_controls(&input, &mut control_state);
        control_state.validate_and_clamp();
        
        assert!(control_state.throttle >= 0.0 && control_state.throttle <= 1.0);
        assert!(control_state.steering >= -1.0 && control_state.steering <= 1.0);
    }
}
