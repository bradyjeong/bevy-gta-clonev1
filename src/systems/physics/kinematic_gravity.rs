use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Player, Car, Helicopter, F16};

/// Apply gravity to kinematic bodies since they ignore Rapier's built-in gravity
/// 
/// This system runs before all movement systems to ensure vehicles fall properly.
/// Kinematic bodies with RigidBody::KinematicVelocityBased ignore forces and gravity,
/// so we must manually apply downward acceleration.
/// 
/// Only applies to vehicles (Car, Helicopter, F16) - not players or other entities.
pub fn apply_kinematic_gravity(
    time: Res<Time>,
    mut vehicle_query: Query<&mut Velocity, (Or<(With<Car>, With<Helicopter>, With<F16>)>, Without<Player>)>,
) {
    const GRAVITY: f32 = 9.81;
    let dt = time.delta_secs();
    
    for mut velocity in vehicle_query.iter_mut() {
        // Apply downward acceleration only to vehicles
        velocity.linvel.y -= GRAVITY * dt;
    }
}
