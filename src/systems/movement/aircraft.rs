use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Helicopter, F16, ActiveEntity, MainRotor, TailRotor, AircraftFlight, F16Specs, ControlState, PlayerControlled};
use crate::systems::physics_utils::PhysicsUtilities;
use crate::config::GameConfig;

pub fn helicopter_movement(
    config: Res<GameConfig>,
    mut helicopter_query: Query<(&mut Velocity, &Transform, &ControlState), (With<Helicopter>, With<ActiveEntity>, With<PlayerControlled>)>,
) {
    let Ok((mut velocity, transform, control_state)) = helicopter_query.single_mut() else {
        return;
    };

    let speed = 40.0;
    let rotation_speed = 5.0;
    let vertical_speed = 20.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Simple, direct control state access
    // Forward/backward movement
    if control_state.is_accelerating() {
        let forward = transform.forward();
        target_linear_velocity += forward * speed;
    }
    if control_state.is_braking() {
        let forward = transform.forward();
        target_linear_velocity -= forward * speed;
    }
    
    // Rotation using steering input
    if control_state.steering != 0.0 {
        target_angular_velocity.y = control_state.steering * rotation_speed;
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation
    }
    
    // Helicopter vertical movement using vertical control
    if control_state.vertical > 0.0 {
        target_linear_velocity.y += vertical_speed * control_state.vertical;
    } else if control_state.vertical < 0.0 {
        target_linear_velocity.y -= vertical_speed * control_state.vertical.abs();
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
    
    // Use unified velocity validation and ground collision
    PhysicsUtilities::validate_velocity(&mut velocity, &config);
    PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 0.5, 5.0);
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
    time: Res<Time>,
    config: Res<GameConfig>,
    mut f16_query: Query<(
        Entity,
        &mut Velocity, 
        &mut Transform, 
        &mut AircraftFlight, 
        &F16Specs,
        &ControlState,
        Option<&mut ExternalForce>
    ), (With<F16>, With<ActiveEntity>, With<PlayerControlled>)>,
) {
    let dt = time.delta_secs();
    
    // Support multiple F16s (following AGENT.md principle)
    for (_entity, mut velocity, mut transform, mut flight, specs, control_state, external_force) in f16_query.iter_mut() {
        
        // === FLIGHT CONTROL INPUT PROCESSING ===
        
        // Direct control deflection using simple ControlState
        flight.pitch = control_state.pitch.clamp(-1.0, 1.0);
        flight.roll = control_state.roll.clamp(-1.0, 1.0);
        flight.yaw = control_state.yaw.clamp(-1.0, 1.0);
        
        // Throttle control with smooth response and validation
        let throttle_input = control_state.throttle.clamp(-1.0, 1.0);
        if throttle_input > 0.0 {
            flight.throttle = (flight.throttle + dt * 1.5 * throttle_input).clamp(0.0, 1.0);
        } else if throttle_input < 0.0 {
            flight.throttle = (flight.throttle + dt * 2.0 * throttle_input).clamp(0.0, 1.0);
        }
        
        // Safety validation for all flight control inputs
        flight.throttle = flight.throttle.clamp(0.0, 1.0);
        flight.pitch = flight.pitch.clamp(-1.0, 1.0);
        flight.roll = flight.roll.clamp(-1.0, 1.0);
        flight.yaw = flight.yaw.clamp(-1.0, 1.0);
        
        // Afterburner using simple boost input
        flight.afterburner = control_state.is_boosting();
        
        if flight.afterburner {
            flight.afterburner_timer += dt;
            // Afterburner lights after 0.3 seconds (fuel flow delay)
            if flight.afterburner_timer > 0.3 {
                flight.afterburner_active = true;
            }
        } else {
            flight.afterburner_timer = 0.0;
            flight.afterburner_active = false;
        }
    
        // === FLIGHT PHYSICS CALCULATIONS ===
        
        // Calculate current airspeed from velocity magnitude
        flight.airspeed = velocity.linvel.length();
        
        // Calculate angle of attack from actual velocity vector FIRST (Oracle fix)
        flight.angle_of_attack = if flight.airspeed > 1.0 {
            let body_forward = transform.forward();
            let velocity_direction = velocity.linvel.normalize();
            let aoa_radians = body_forward.angle_between(velocity_direction);
            
            // Determine sign (positive when nose up relative to velocity)
            let cross_product = body_forward.cross(velocity_direction);
            let sign = if cross_product.y > 0.0 { 1.0 } else { -1.0 };
            
            aoa_radians * sign
        } else {
            0.0 // No meaningful AoA at very low speeds
        };
        
        // Realistic engine thrust with idle thrust (Oracle feedback - reduced to 8%)
        let idle_thrust = specs.max_thrust * 0.08; // ~8% idle thrust (realistic)
        let target_thrust = if flight.afterburner_active {
            // Afterburner adds 35% additional thrust when active (1.35x ratio)
            let base_thrust = idle_thrust + (specs.max_thrust - idle_thrust) * flight.throttle.clamp(0.0, 1.0);
            base_thrust + (specs.afterburner_thrust - specs.max_thrust) * flight.throttle.clamp(0.0, 1.0)
        } else {
            idle_thrust + (specs.max_thrust - idle_thrust) * flight.throttle.clamp(0.0, 1.0)
        };
        
        // Realistic engine spool time using specs
        let spool_rate = if flight.afterburner_active { 
            specs.spool_rate_afterburner 
        } else { 
            specs.spool_rate_normal 
        };
        flight.current_thrust = flight.current_thrust.lerp(target_thrust, dt * spool_rate);
        
        // Thrust force along aircraft's forward direction
        let thrust_force = transform.forward() * flight.current_thrust;
        
        // Proper aerodynamic drag with induced drag: CD = CD0 + k*CL²
        let air_density = 1.225;
        let dynamic_pressure = 0.5 * air_density * flight.airspeed.powi(2);
        
        // Calculate lift coefficient using current AoA
        let lift_coefficient = if flight.angle_of_attack.abs() < specs.max_angle_of_attack {
            specs.lift_coefficient_0 + specs.lift_coefficient_alpha * flight.angle_of_attack
        } else {
            // Gradual stall degradation instead of cliff-edge
            let stall_factor = 1.0 - ((flight.angle_of_attack.abs() - specs.max_angle_of_attack) / 0.2).clamp(0.0, 1.0);
            (specs.lift_coefficient_0 + specs.lift_coefficient_alpha * specs.max_angle_of_attack) * stall_factor
        };
        
        // Total drag coefficient: parasitic + induced (Oracle fix - realistic k factor)
        let induced_drag_factor = 0.13; // k factor for F-16 (AR ≈ 3, e ≈ 0.8)
        let total_drag_coefficient = specs.drag_coefficient + induced_drag_factor * lift_coefficient * lift_coefficient;
        
        let drag_force = -velocity.linvel.normalize_or_zero() * 
            dynamic_pressure * specs.wing_area * total_drag_coefficient;
        
        // Generate lift force using calculated lift coefficient
        let lift_force = if flight.airspeed > specs.stall_speed {
            // Lift direction perpendicular to velocity vector (Oracle fix - correct cross product)
            let velocity_normalized = velocity.linvel.normalize_or_zero();
            let right = transform.right();
            let lift_direction = right.cross(velocity_normalized).normalize_or_zero();
            
            lift_direction * dynamic_pressure * specs.wing_area * lift_coefficient
        } else {
            Vec3::ZERO // Below stall speed - no effective lift
        };
        
        // Proper gravity force (Oracle feedback - correct value)
        let gravity_force = Vec3::new(0.0, -9.81 * specs.mass, 0.0);
        
        // Combined forces
        let total_force = thrust_force + drag_force + lift_force + gravity_force;
    
        // === ROTATIONAL DYNAMICS ===
        
        // Calculate control effectiveness based on airspeed
        let control_effectiveness = (flight.airspeed / 50.0).clamp(0.2, 1.0);
        
        // Calculate proper torques WITHOUT inertia multiplication (Oracle fix - ExternalForce handles inertia)
        let pitch_moment = flight.pitch * specs.control_sensitivity * control_effectiveness;
        let roll_moment = flight.roll * specs.control_sensitivity * control_effectiveness;  
        let yaw_moment = flight.yaw * specs.control_sensitivity * control_effectiveness * 0.5;
        
        // Add aerodynamic damping proportional to angular rates (without inertia for ExternalForce)
        let damping_factor = 0.1;
        let pitch_damping = -velocity.angvel.x * damping_factor;
        let roll_damping = -velocity.angvel.z * damping_factor;
        let yaw_damping = -velocity.angvel.y * damping_factor;
        
        // Total torque vector with damping
        let torque = Vec3::new(
            pitch_moment + pitch_damping,
            yaw_moment + yaw_damping, 
            -roll_moment + roll_damping
        );
        
        // Use ExternalForce if available, otherwise apply torque through velocity (fallback)
        if let Some(mut ext_force) = external_force {
            ext_force.torque = torque;
            ext_force.force = total_force;
        } else {
            // Fallback: Apply forces manually with proper inertia scaling (Oracle feedback - F = ma, τ = Iα)
            let acceleration = total_force / specs.mass;
            let angular_acceleration = Vec3::new(
                torque.x / specs.inertia_pitch,  // Pitch
                torque.y / specs.inertia_yaw,    // Yaw
                torque.z / specs.inertia_roll    // Roll
            );
            
            // Safety checks
            if acceleration.is_finite() && acceleration.length() < 100.0 {
                velocity.linvel += acceleration * dt;
            }
            if angular_acceleration.is_finite() && angular_acceleration.length() < 10.0 {
                velocity.angvel += angular_acceleration * dt;
            }
        }
        
        // Use unified physics safety systems
        PhysicsUtilities::validate_velocity(&mut velocity, &config);
        PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 1.0, 10.0);
        
        // CRITICAL: Add position bounds checking to prevent Rapier crash (Oracle feedback)
        let max_world_bound = 50000.0; // Safe limit for Rapier physics engine
        let current_pos = transform.translation;
        
        if current_pos.x.abs() > max_world_bound || 
           current_pos.y.abs() > max_world_bound || 
           current_pos.z.abs() > max_world_bound {
            
            // Emergency position reset - teleport back to safe zone
            warn!("F16 exceeded world bounds at {:?}, resetting to safe position", current_pos);
            
            // Reset to a safe position near world center
            let safe_position = Vec3::new(
                current_pos.x.clamp(-max_world_bound * 0.8, max_world_bound * 0.8),
                current_pos.y.clamp(10.0, 2000.0), // Keep above ground, below stratosphere
                current_pos.z.clamp(-max_world_bound * 0.8, max_world_bound * 0.8)
            );
            
            // Immediately update transform to safe position
            transform.translation = safe_position;
            
            // Dramatically reduce velocity to prevent further overrun
            velocity.linvel *= 0.1;
            velocity.angvel *= 0.1;
        }
        
        // Additional velocity safety check - prevent extreme speeds
        let max_safe_velocity = 1000.0; // m/s (~Mach 3, well above F-16 limits)
        if velocity.linvel.length() > max_safe_velocity {
            warn!("F16 velocity {} exceeds safe limit, clamping", velocity.linvel.length());
            velocity.linvel = velocity.linvel.normalize() * max_safe_velocity;
        }
        
        if velocity.angvel.length() > 50.0 { // Reasonable angular velocity limit
            warn!("F16 angular velocity {} exceeds safe limit, clamping", velocity.angvel.length());
            velocity.angvel = velocity.angvel.normalize() * 50.0;
        }
        
        // === STALL HANDLING ===
        
        // Enhanced stall handling based on AoA and airspeed
        let is_stalled = flight.angle_of_attack.abs() > specs.max_angle_of_attack;
        if is_stalled || flight.airspeed < specs.stall_speed {
            velocity.linvel *= 0.98; // Gradual stall drag
            
            // Add turbulence only when stalled for realism
            if is_stalled {
                let turbulence = Vec3::new(
                    (time.elapsed_secs() * 3.0).sin() * 1.0,
                    (time.elapsed_secs() * 2.0).cos() * 0.5,
                    (time.elapsed_secs() * 4.0).sin() * 0.8,
                );
                velocity.linvel += turbulence * dt;
            }
        }
    }
}
