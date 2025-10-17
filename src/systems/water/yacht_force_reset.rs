use crate::components::water::Yacht;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn reset_yacht_forces(mut query: Query<&mut ExternalForce, With<Yacht>>) {
    for mut external_force in query.iter_mut() {
        external_force.force = Vec3::ZERO;
        external_force.torque = Vec3::ZERO;
    }
}
