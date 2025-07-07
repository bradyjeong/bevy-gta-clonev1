//! ───────────────────────────────────────────────
//! System:   Supercar Input
//! Purpose:  Processes user input and control mapping
//! Schedule: Update
//! Reads:    ActiveEntity, Car, SuperCar, mut, ControlManager
//! Writes:   SuperCar
//! Invariants:
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;
use crate::systems::input::{ControlManager, ControlAction};

/// Focused system for handling supercar input processing
pub fn supercar_input_system(
    control_manager: Res<ControlManager>,
    mut supercar_query: Query<&mut SuperCar, (With<Car>, With<ActiveEntity>, With<SuperCar>)>,
) {
    let Ok(mut supercar) = supercar_query.single_mut() else {
        return;
    };
    // Handle driving mode changes
    handle_driving_mode_changes(&mut supercar, &control_manager);
    
    // Handle launch control activation
    handle_launch_control(&mut supercar, &control_manager);
    // Update mode flags based on current driving mode
    supercar.sport_mode_active = matches!(supercar.driving_mode, DrivingMode::Sport | DrivingMode::Track);
    supercar.track_mode_active = matches!(supercar.driving_mode, DrivingMode::Track);
}
fn handle_driving_mode_changes(supercar: &mut SuperCar, _control_manager: &ControlManager) {
    // Driving mode changes based on control inputs
    // TODO: Add dedicated key binding for driving mode cycling
    // For now, driving mode cycling is disabled in this demo
    if false {
        // Cycle through driving modes (placeholder - would need dedicated key)
        supercar.driving_mode = match supercar.driving_mode {
            DrivingMode::Comfort => DrivingMode::Sport,
            DrivingMode::Sport => DrivingMode::Track,
            DrivingMode::Track => DrivingMode::Comfort,
            DrivingMode::Custom => DrivingMode::Sport,
        };
    }
}
fn handle_launch_control(supercar: &mut SuperCar, control_manager: &ControlManager) {
    // Launch control activation (both brake and accelerate pressed)
    let current_speed_ms = supercar.last_velocity_cache.map(|v| v.length()).unwrap_or(0.0);
    let current_speed_mph = current_speed_ms * 2.237; // Convert m/s to mph
    if control_manager.is_control_active(ControlAction::Brake) && 
       control_manager.is_control_active(ControlAction::Accelerate) &&
       current_speed_mph < 5.0 {
        
        supercar.launch_control_engaged = true;
        supercar.rpm = supercar.launch_rpm_limit; // Hold at launch RPM
        supercar.is_timing_launch = true;
        supercar.zero_to_sixty_time = 0.0;
    } else if supercar.launch_control_engaged && !control_manager.is_control_active(ControlAction::Brake) {
        // Launch control release
        supercar.launch_control_engaged = false;
    }
}
