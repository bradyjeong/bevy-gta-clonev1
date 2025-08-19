use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{F16, ActiveEntity, AircraftFlight, SimpleF16Specs, ControlState, PlayerControlled, MainRotor, TailRotor};
use crate::systems::physics_utils::PhysicsUtilities;
use crate::systems::movement::simple_flight_common::SimpleFlightCommon;
use crate::config::GameConfig;

/// Apply damping from SimpleF16Specs to newly spawned F16s
pub fn apply_f16_damping(
    mut commands: Commands,
    f16_query: Query<(Entity, &SimpleF16Specs), (With<F16>, Added<SimpleF16Specs>)>,
) {
    for (entity, specs) in f16_query.iter() {
        commands.entity(entity).insert(Damping {
            linear_damping: specs.linear_damping,
            angular_damping: specs.angular_damping,
        });
    }
}

/// Simplified F16 flight system following AGENT.md simplicity principles
/// 
/// Replaces complex aerodynamic calculations with straightforward flight physics:
/// - Pitch/roll → Direct angular velocity
/// - Throttle → Forward thrust
/// - Yaw → Rotation around Y-axis
/// - Afterburner → Thrust multiplier
/// 
/// Benefits:
/// - Easy to understand: Direct control mapping
/// - Maintainable: No complex aerodynamic formulas
/// - Performant: Minimal calculations per frame
/// - Flight Feel: Maintains responsive aircraft control
pub fn simple_f16_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut f16_query: Query<(
        &mut Velocity,
        &mut Transform,
        &mut AircraftFlight,
        &SimpleF16Specs,
        &ControlState,
    ), (With<F16>, With<ActiveEntity>, With<PlayerControlled>)>,
) {
    let dt = SimpleFlightCommon::stable_dt(&time);
    
    for (mut velocity, transform, mut flight, specs, control_state) in f16_query.iter_mut() {
        
        // === ULTRA-SIMPLIFIED INPUT PROCESSING ===
        
        // Direct throttle processing (no round-trip conversion)
        flight.throttle = SimpleFlightCommon::process_throttle(
            control_state,
            flight.throttle,
            specs.throttle_increase_rate,
            specs.throttle_decrease_rate,
            dt,
        );
        
        // Direct afterburner state
        flight.afterburner_active = control_state.is_boosting();
        
        // === MINIMAL FLIGHT PHYSICS ===
        
        // Calculate thrust (all values from specs, no magic numbers)
        let boost_multiplier = if flight.afterburner_active { specs.afterburner_multiplier } else { 1.0 };
        let thrust_force = specs.max_thrust * flight.throttle * boost_multiplier;
        let forward_force = transform.forward() * thrust_force;
        
        // === DIRECT ANGULAR CONTROL ===
        
        // Read controls directly (no state duplication)
        let local_target_ang = Vec3::new(
            control_state.pitch * specs.pitch_rate_max,   // +X pitch
            control_state.yaw * specs.yaw_rate_max,       // +Y yaw
            -control_state.roll * specs.roll_rate_max,    // -Z roll
        );
        let world_target_ang = transform.rotation.mul_vec3(local_target_ang);
        
        // Apply angular velocity (lerp factor from specs)
        velocity.angvel = velocity.angvel.lerp(world_target_ang, dt * specs.angular_lerp_factor);
        
        // === LINEAR FORCES ===
        
        // Apply forward thrust (no magic mass clamp)
        velocity.linvel += forward_force * dt / specs.mass;
        
        // Orientation-aware lift assistance (deadzone from specs)
        if flight.throttle > specs.throttle_deadzone {
            velocity.linvel += transform.up() * flight.throttle * specs.lift_per_throttle * dt;
        }
        
        // === MINIMAL STATE TRACKING ===
        
        flight.airspeed = velocity.linvel.length();
        
        // === SHARED PHYSICS (NO GRAVITY DUPLICATION) ===
        
        SimpleFlightCommon::apply_velocity_clamps(&mut velocity, &config);
        
        // Keep aircraft above minimum altitude (simple terrain avoidance)
        if transform.translation.y < specs.min_altitude {
            // One-shot upward impulse instead of hard override
            velocity.linvel.y += specs.emergency_pullup_force * dt;
        }
    }
}

/// Simple helicopter controls that work alongside F16
/// Uses similar simplified approach for consistency
pub fn simple_helicopter_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut helicopter_query: Query<(&mut Velocity, &Transform, &ControlState), 
        (With<crate::components::Helicopter>, With<ActiveEntity>, With<PlayerControlled>)>,
) {
    let dt = time.delta_secs().clamp(0.001, 0.05);
    
    for (mut velocity, transform, control_state) in helicopter_query.iter_mut() {
        
        // Basic helicopter parameters
        let speed = 30.0;
        let rotation_speed = 4.0;
        let vertical_speed = 15.0;
        let roll_speed = 2.0; // Gentler than yaw
        let lateral_speed = 15.0; // Lateral movement when banking
        
        let mut target_linear_velocity = Vec3::ZERO;
        let mut target_angular_velocity = Vec3::ZERO;
        
        // Forward/backward movement using pitch
        if control_state.pitch > 0.1 {
            target_linear_velocity += transform.forward() * speed * control_state.pitch;
        } else if control_state.pitch < -0.1 {
            target_linear_velocity -= transform.forward() * speed * control_state.pitch.abs();
        }
        
        // Rotation using yaw (invert sign for correct direction)
        if control_state.yaw.abs() > 0.1 {
            target_angular_velocity.y = -control_state.yaw * rotation_speed;
        }
        
        // Vertical movement (collective)
        if control_state.vertical > 0.1 {
            target_linear_velocity.y += vertical_speed * control_state.vertical;
        } else if control_state.vertical < -0.1 {
            target_linear_velocity.y -= vertical_speed * control_state.vertical.abs();
        }
        
        // Roll controls (Q/E keys) - banking and lateral movement
        if control_state.roll.abs() > 0.1 {
            // Roll angular velocity (banking around Z-axis)
            target_angular_velocity.z = -control_state.roll * roll_speed;
            
            // Lateral movement when rolling (helicopter banks into turn)  
            let lateral_force = transform.right() * -control_state.roll * lateral_speed;
            target_linear_velocity += lateral_force;
        }
        
        // Apply forces with smooth interpolation
        velocity.linvel = velocity.linvel.lerp(target_linear_velocity, dt * 4.0);
        velocity.angvel = velocity.angvel.lerp(target_angular_velocity, dt * 6.0);
        
        // Simple physics
        velocity.linvel *= 0.95; // Air resistance
        velocity.angvel *= 0.90; // Angular damping
        
        // Gravity
        velocity.linvel += Vec3::new(0.0, -9.81, 0.0) * dt;
        
        // Safety systems
        PhysicsUtilities::validate_velocity(&mut velocity, &config);
        PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 1.0, 5.0);
    }
}

/// Rotate helicopter main and tail rotors every frame
pub fn rotate_helicopter_rotors(
    time: Res<Time>,
    mut main_rotor_query: Query<&mut Transform, (With<MainRotor>, Without<TailRotor>)>,
    mut tail_rotor_query: Query<&mut Transform, (With<TailRotor>, Without<MainRotor>)>,
) {
    // Use delta-based rotation to preserve initial blade offsets
    let main_delta = time.delta_secs() * 20.0; // rad/s
    let tail_delta = time.delta_secs() * 35.0; // rad/s

    for mut transform in main_rotor_query.iter_mut() {
        transform.rotate_y(main_delta);
    }

    for mut transform in tail_rotor_query.iter_mut() {
        transform.rotate_z(tail_delta);
    }
}

