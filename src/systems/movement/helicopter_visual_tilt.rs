use crate::components::{
    ActiveEntity, ControlState, Helicopter, HelicopterVisualBody, PlayerControlled,
    SimpleHelicopterSpecs, SimpleHelicopterSpecsHandle,
};
use bevy::prelude::*;

/// GTA-style visual tilt system for helicopters
/// Tilts the visual mesh child entity based on input, while physics body stays level
#[allow(clippy::type_complexity)]
pub fn helicopter_visual_tilt(
    time: Res<Time>,
    heli_specs_assets: Res<Assets<SimpleHelicopterSpecs>>,
    helicopter_query: Query<
        (&SimpleHelicopterSpecsHandle, &ControlState),
        (With<Helicopter>, With<ActiveEntity>, With<PlayerControlled>),
    >,
    mut visual_body_query: Query<(&mut Transform, &ChildOf), With<HelicopterVisualBody>>,
) {
    let dt = time.delta_secs().clamp(0.0, 0.1);

    for (mut visual_transform, child_of) in visual_body_query.iter_mut() {
        // Get parent helicopter's control state and specs
        let Ok((specs_handle, control_state)) = helicopter_query.get(child_of.parent()) else {
            continue;
        };

        let Some(specs) = heli_specs_assets.get(&specs_handle.0) else {
            continue;
        };

        let dz = specs.input_deadzone.clamp(0.0, 0.3);

        // Process pitch/roll input
        let forward_input = if control_state.pitch.abs() < dz {
            0.0
        } else {
            control_state.pitch
        };
        let strafe_input = if control_state.roll.abs() < dz {
            0.0
        } else {
            control_state.roll
        };

        // Target tilt angles based on input
        let target_pitch = -forward_input * specs.visual_tilt_pitch_max.to_radians();
        let target_roll = -strafe_input * specs.visual_tilt_roll_max.to_radians();

        // Get current pitch/roll from visual body
        let (current_yaw, current_pitch, current_roll) =
            visual_transform.rotation.to_euler(EulerRot::YXZ);

        // Smooth lerp to target angles
        let tilt_speed = specs.visual_tilt_speed.clamp(1.0, 20.0);
        let lerp_factor = (dt * tilt_speed).clamp(0.0, 1.0);

        let new_pitch = current_pitch + (target_pitch - current_pitch) * lerp_factor;
        let new_roll = current_roll + (target_roll - current_roll) * lerp_factor;

        // Apply tilt to visual body only (yaw stays at 0 - parent handles yaw rotation)
        visual_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, current_yaw, new_pitch, new_roll);
    }
}
