use crate::components::water::{Yacht, YachtState};
use crate::components::{ActiveEntity, PlayerControlled, PropellerHub};
use bevy::prelude::*;

#[allow(clippy::type_complexity)]
pub fn propeller_spin_system(
    time: Res<Time>,
    yacht_query: Query<(&YachtState, &Children), (With<Yacht>, With<ActiveEntity>, With<PlayerControlled>)>,
    mut propeller_query: Query<&mut Transform, With<PropellerHub>>,
) {
    for (yacht_state, children) in yacht_query.iter() {
        for child in children.iter() {
            if let Ok(mut transform) = propeller_query.get_mut(child) {
                let throttle = yacht_state.throttle;
                
                if throttle.abs() < 0.01 {
                    continue;
                }

                let target_rpm = throttle * 600.0;
                let rotation_speed = (target_rpm / 60.0) * std::f32::consts::TAU;
                let delta_rotation = rotation_speed * time.delta_secs();
                
                transform.rotate_local_z(delta_rotation);
            }
        }
    }
}
