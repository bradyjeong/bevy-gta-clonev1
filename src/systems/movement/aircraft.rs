use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Helicopter, F16, ActiveEntity, MainRotor, TailRotor, AircraftFlight};
use crate::systems::input::{ControlManager, is_accelerating, is_braking, get_steering_input, get_throttle_input, get_pitch_input, get_roll_input, get_yaw_input, is_afterburner_active};

pub fn helicopter_movement(
    control_manager: Res<ControlManager>,
    mut helicopter_query: Query<(&mut Velocity, &Transform), (With<Helicopter>, With<ActiveEntity>)>,
) {
    let Ok((mut velocity, transform)) = helicopter_query.single_mut() else {
        return;
    };

    let speed = 40.0;
    let rotation_speed = 5.0;
    let vertical_speed = 20.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Use ControlManager for helicopter movement
    // Forward/backward movement
    if is_accelerating(&control_manager) {
        let forward = transform.forward();
        target_linear_velocity += forward * speed;
    }
    if is_braking(&control_manager) {
        let forward = transform.forward();
        target_linear_velocity -= forward * speed;
    }
    
    // Rotation - DIRECT velocity control
    let steering = get_steering_input(&control_manager);
    if steering != 0.0 {
        target_angular_velocity.y = steering * rotation_speed;
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation
    }
    
    // HELICOPTER SPECIFIC: Vertical movement using throttle input
    let throttle = get_throttle_input(&control_manager);
    if throttle > 0.0 {
        target_linear_velocity.y += vertical_speed * throttle;
    } else if throttle < 0.0 {
        target_linear_velocity.y -= vertical_speed * throttle.abs();
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
    
    // Ground collision: prevent helicopter from going underground
    let ground_level = 0.5; // Minimum altitude above ground for helicopter
    if transform.translation.y < ground_level {
        // Stop downward movement
        if velocity.linvel.y < 0.0 {
            velocity.linvel.y = 0.0;
        }
        // Add small upward force to keep helicopter above ground
        velocity.linvel.y += 5.0;
    }
}

pub fn rotate_helicopter_rotors(
    time: Res<Time>,
    mut main_rotor_query: Query<&mut Transform, (With<MainRotor>, Without<TailRotor>)>,
    mut tail_rotor_query: Query<&mut Transform, (With<TailRotor>, Without<MainRotor>)>,
) {
    let main_rotor_speed = 20.0; // Fast rotation for main rotor
    let tail_rotor_speed = 35.0; // Even faster for tail rotor

    // Rotate main rotors (around Y axis)
    for mut transform in main_rotor_query.iter_mut() {
        let rotation = Quat::from_rotation_y(time.elapsed_secs() * main_rotor_speed);
        transform.rotation = rotation;
    }

    // Rotate tail rotors (around Z axis)  
    for mut transform in tail_rotor_query.iter_mut() {
        let rotation = Quat::from_rotation_z(time.elapsed_secs() * tail_rotor_speed);
        transform.rotation = rotation;
    }
}

pub fn f16_movement(
    control_manager: Res<ControlManager>,
    time: Res<Time>,
    mut f16_query: Query<(&mut Velocity, &mut Transform, &mut AircraftFlight), (With<F16>, With<ActiveEntity>)>,
) {
    let Ok((mut velocity, transform, mut flight)) = f16_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    
    // === FLIGHT CONTROL INPUT PROCESSING ===
    
    // Use ControlManager for F16 flight controls
    // Pitch control (nose up/down) 
    let pitch_input = get_pitch_input(&control_manager);
    if pitch_input > 0.0 {
        flight.pitch = (flight.pitch + dt * 3.0 * pitch_input).clamp(-1.0, 1.0); // Nose up
    } else if pitch_input < 0.0 {
        flight.pitch = (flight.pitch + dt * 3.0 * pitch_input).clamp(-1.0, 1.0); // Nose down
    } else {
        flight.pitch = flight.pitch * (1.0 - dt * 5.0); // Return to center
    }
    
    // Roll control (banking left/right)
    let roll_input = get_roll_input(&control_manager);
    if roll_input != 0.0 {
        flight.roll = (flight.roll + dt * 4.0 * roll_input).clamp(-1.0, 1.0);
    } else {
        flight.roll = flight.roll * (1.0 - dt * 3.0); // Return to center
    }
    
    // Yaw control (rudder)
    let yaw_input = get_yaw_input(&control_manager);
    if yaw_input != 0.0 {
        flight.yaw = (flight.yaw + dt * 2.0 * yaw_input).clamp(-1.0, 1.0);
    } else {
        flight.yaw = flight.yaw * (1.0 - dt * 4.0); // Return to center
    }
    
    // Throttle control
    let throttle_input = get_throttle_input(&control_manager);
    if throttle_input > 0.0 {
        flight.throttle = (flight.throttle + dt * 1.5 * throttle_input).clamp(0.0, 1.0);
    } else if throttle_input < 0.0 {
        flight.throttle = (flight.throttle + dt * 2.0 * throttle_input).clamp(0.0, 1.0);
    }
    
    // Afterburner
    flight.afterburner = is_afterburner_active(&control_manager);
    
    // === FLIGHT PHYSICS CALCULATIONS ===
    
    // Calculate current airspeed from velocity magnitude
    flight.airspeed = velocity.linvel.length();
    
    // F16 engine thrust with realistic spool-up time
    let target_thrust = if flight.afterburner {
        flight.thrust_power * 2.2 * flight.throttle.clamp(0.0, 1.0) // F16 afterburner boost
    } else {
        flight.thrust_power * flight.throttle.clamp(0.0, 1.0)
    };
    
    // Realistic engine spool time (F100 turbofan characteristics)
    let spool_rate = if flight.afterburner { 1.5 } else { 2.5 };
    flight.current_thrust = flight.current_thrust.lerp(target_thrust, dt * spool_rate);
    
    // Thrust force along aircraft's nose direction (F16 nose points in -Z local space)
    let thrust_force = transform.forward() * flight.current_thrust.clamp(0.0, flight.thrust_power * 2.2);
    
    // Enhanced aerodynamic drag with realistic F16 characteristics
    let speed_factor = (flight.airspeed / flight.max_speed).clamp(0.0, 1.0);
    let drag_multiplier = 1.0 + speed_factor * speed_factor * 2.0; // Increased drag at high speeds
    let drag_force = -velocity.linvel.normalize_or_zero() * 
        flight.drag_coefficient * flight.airspeed * flight.airspeed * drag_multiplier * 0.008;
    
    // Realistic lift generation with angle of attack
    flight.angle_of_attack = flight.pitch * 0.8; // Pitch affects AoA
    let up_vector = transform.up();
    let lift_force = if flight.airspeed > flight.stall_speed {
        let lift_efficiency = (1.0 - (flight.angle_of_attack.abs() / 1.5).clamp(0.0, 1.0)).max(0.2);
        up_vector * flight.lift_coefficient * flight.airspeed * lift_efficiency * flight.angle_of_attack * 0.6
    } else {
        Vec3::ZERO // Stall condition - no lift
    };
    
    // Realistic gravity for fighter jet
    let gravity_force = Vec3::new(0.0, -9.81 * 8.0, 0.0); // Slightly reduced for fighter jet feel
    
    // Combined forces
    let total_force = thrust_force + drag_force + lift_force + gravity_force;
    
    // === ROTATIONAL DYNAMICS ===
    
    // Calculate rotational velocity based on control inputs and airspeed
    let control_effectiveness = (flight.airspeed / 50.0).clamp(0.2, 1.0); // Less control at low speed
    
    let pitch_rate = flight.pitch * flight.control_sensitivity * control_effectiveness;
    let roll_rate = flight.roll * flight.control_sensitivity * control_effectiveness;
    let yaw_rate = flight.yaw * flight.control_sensitivity * control_effectiveness * 0.5; // Rudder less effective
    
    // Apply rotational forces - correct aircraft axis mapping
    let angular_velocity = Vec3::new(pitch_rate, yaw_rate, -roll_rate);
    
    // Safety check: ensure angular velocity is finite and reasonable
    if angular_velocity.is_finite() && angular_velocity.length() < 50.0 {
        velocity.angvel = angular_velocity;
    }
    
    // === VELOCITY UPDATE ===
    
    // Apply forces to linear velocity (F = ma, assuming mass = 1 for simplicity)
    let force_delta = total_force * dt;
    
    // Safety check: prevent extreme force values that could destabilize physics
    if force_delta.is_finite() && force_delta.length() < 10000.0 {
        velocity.linvel += force_delta;
    }
    
    // Safety check: clamp velocity to prevent physics instability
    velocity.linvel = velocity.linvel.clamp_length_max(flight.max_speed);
    
    // Additional safety: ensure velocity components are finite
    if !velocity.linvel.is_finite() {
        velocity.linvel = Vec3::ZERO;
    }
    
    // Ground collision: prevent F16 from going underground
    let ground_level = 1.0; // Minimum altitude above ground
    if transform.translation.y < ground_level {
        // Stop downward movement and apply bounce
        if velocity.linvel.y < 0.0 {
            velocity.linvel.y = 0.0;
        }
        // Add small upward force to keep aircraft above ground
        velocity.linvel.y += 10.0 * dt;
    }
    
    // === STALL HANDLING ===
    
    // If below stall speed and not gaining thrust, apply extra drag
    if flight.airspeed < flight.stall_speed && flight.current_thrust < 20.0 {
        velocity.linvel *= 0.95; // Gradual stall
        // Add some random turbulence for realism
        let turbulence = Vec3::new(
            (time.elapsed_secs() * 3.0).sin() * 2.0,
            (time.elapsed_secs() * 2.0).cos() * 1.5,
            (time.elapsed_secs() * 4.0).sin() * 1.0,
        );
        velocity.linvel += turbulence * dt;
    }
}
