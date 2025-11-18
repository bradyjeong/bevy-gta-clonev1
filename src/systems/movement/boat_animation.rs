use crate::components::control_state::ControlState;
use crate::components::{ActiveEntity, PlayerControlled, PropellerHub, Rudder, Yacht};
use bevy::prelude::*;

/// Handles visual animations for boats (Propeller spin, Rudder turning)
/// dependent on ControlState input.
#[allow(clippy::type_complexity)]
pub fn boat_animation_system(
    time: Res<Time>,
    yacht_query: Query<
        (&ControlState, &Children),
        (With<Yacht>, With<ActiveEntity>, With<PlayerControlled>),
    >,
    mut propeller_query: Query<(&mut Transform, &mut PropellerHub), Without<Rudder>>,
    mut rudder_query: Query<(&mut Transform, &Rudder), Without<PropellerHub>>,
) {
    let dt = time.delta_secs();

    for (controls, children) in yacht_query.iter() {
        // Propeller throttle input (forward - brake/reverse)
        // GTA style: brake acts as reverse when stopped, but for visuals we just want to know "effort"
        let throttle_effort = controls.throttle - controls.brake;
        
        // Steering input (-1.0 to 1.0)
        let steering_input = controls.steering;

        for child in children.iter() {
            // --- Propeller Animation ---
            if let Ok((mut transform, mut prop)) = propeller_query.get_mut(child) {
                // Target RPM based on effort. 
                // 600 RPM max seems reasonable for visual speed.
                let target_rpm = throttle_effort * 600.0;

                // Rotational Inertia Simulation
                // Spin up fast, spin down slow (water resistance vs engine torque)
                let lerp_factor = if target_rpm.abs() > prop.current_rpm.abs() {
                    3.0 * dt // Power on: fast response
                } else {
                    0.8 * dt // Power off: slow coast down
                };

                prop.current_rpm = prop.current_rpm + (target_rpm - prop.current_rpm) * lerp_factor;

                // Apply rotation if moving
                if prop.current_rpm.abs() > 1.0 {
                    // RPM to Radians Per Second: (RPM / 60) * 2PI
                    // Negative for Standard Clockwise Rotation (Right-Handed)
                    let rads_per_sec = -(prop.current_rpm / 60.0) * std::f32::consts::TAU;
                    transform.rotate_local_y(rads_per_sec * dt);
                }
            }

            // --- Rudder Animation ---
            if let Ok((mut transform, rudder)) = rudder_query.get_mut(child) {
                // Target angle based on steering input
                // Negative steering usually means turn left, which means rudder points left (trailing edge left)
                // Standard maritime: Right rudder (trailing edge right) turns boat right.
                // If steering is +1.0 (Right), rudder should rotate negative Y (Local) or positive?
                // Let's assume +Y is Left, -Y is Right in standard Bevy coordinates if Z is forward.
                // Visual check: If we rotate Y, we'll see. Usually steering * max_angle is fine, flip sign if inverted.
                let target_angle = -steering_input * rudder.max_angle;

                // Smoothly interpolate rudder position (hydraulic steering delay)
                let current_rot = transform.rotation;
                let target_rot = Quat::from_rotation_y(target_angle);
                
                // Slerp factor for steering speed
                transform.rotation = current_rot.slerp(target_rot, dt * 5.0);
            }
        }
    }
}
