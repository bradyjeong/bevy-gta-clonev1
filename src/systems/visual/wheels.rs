#![allow(clippy::type_complexity)]
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::components::{
    ActiveEntity, Car, CarWheelsConfig, ControlState, WheelMesh, WheelPos, WheelSteerPivot,
};

pub fn wheel_steering_system(
    car_query: Query<(&ControlState, &CarWheelsConfig, &Children), (With<Car>, With<ActiveEntity>)>,
    mut pivot_query: Query<(&WheelSteerPivot, &mut Transform), Without<Car>>,
    children_query: Query<&Children>,
) {
    for (control_state, wheel_config, car_children) in car_query.iter() {
        // Direct input-to-visual mapping: wheels follow raw driver input
        let steer_angle = control_state.steering * wheel_config.max_steer_rad;

        #[cfg(feature = "debug-movement")]
        if control_state.steering.abs() > 0.01 {
            info!(
                "Steering: raw={:.2}, angle={:.2}°",
                control_state.steering,
                steer_angle.to_degrees()
            );
        }

        for child in car_children.iter() {
            if let Ok(grandchildren) = children_query.get(child) {
                for grandchild in grandchildren.iter() {
                    if let Ok(great_grandchildren) = children_query.get(grandchild) {
                        for ggchild in great_grandchildren.iter() {
                            if let Ok((pivot, mut pivot_transform)) = pivot_query.get_mut(ggchild) {
                                // Only apply steering to FRONT wheels (FL, FR)
                                if matches!(pivot.pos, WheelPos::FL | WheelPos::FR) {
                                    pivot_transform.rotation = Quat::from_rotation_y(steer_angle);
                                }
                                // Rear wheels (RL, RR) stay fixed at 0 rotation
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn wheel_rolling_system(
    time: Res<Time>,
    car_query: Query<(&Velocity, &Transform, &Children), (With<Car>, Without<WheelMesh>)>,
    mut wheel_query: Query<(&mut WheelMesh, &mut Transform)>,
    children_query: Query<&Children>,
) {
    let dt = time.delta_secs();

    for (velocity, car_transform, car_children) in car_query.iter() {
        let forward_dir = car_transform.forward();
        let forward_velocity = velocity.linvel.dot(*forward_dir);

        for child in car_children.iter() {
            if let Ok(grandchildren) = children_query.get(child) {
                for grandchild in grandchildren.iter() {
                    if let Ok((mut wheel, mut wheel_transform)) = wheel_query.get_mut(grandchild) {
                        update_wheel_roll(&mut wheel, &mut wheel_transform, forward_velocity, dt);
                    }

                    if let Ok(great_grandchildren) = children_query.get(grandchild) {
                        for ggchild in great_grandchildren.iter() {
                            if let Ok((mut wheel, mut wheel_transform)) =
                                wheel_query.get_mut(ggchild)
                            {
                                update_wheel_roll(
                                    &mut wheel,
                                    &mut wheel_transform,
                                    forward_velocity,
                                    dt,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_wheel_roll(wheel: &mut WheelMesh, transform: &mut Transform, velocity: f32, dt: f32) {
    if velocity.abs() < 0.05 {
        return;
    }

    wheel.roll_angle += (velocity / wheel.radius) * dt * wheel.roll_dir;

    wheel.roll_angle = wheel.roll_angle.rem_euclid(std::f32::consts::TAU);

    // FIX: Combine base orientation (Z-axis 90°) with rolling rotation (Y-axis)
    // Base rotation aligns cylinder axis with axle (Y-axis), roll rotates around axle
    let base_rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let roll_rotation = Quat::from_rotation_y(wheel.roll_angle);
    transform.rotation = base_rotation * roll_rotation;
}
