use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::config::GameConfig;
use crate::components::ControlState;
use crate::systems::physics_utils::PhysicsUtilities;

/// CRITICAL: High-performance realistic vehicle physics system
/// Optimized for 60+ FPS with comprehensive safety checks and entity limits
pub fn realistic_vehicle_physics_system(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(
        Entity,
        &mut Velocity,
        &mut Transform,
        &mut RealisticVehicle,
        &mut VehicleDynamics,
        &mut EnginePhysics,
        &mut VehicleSuspension,
        &mut TirePhysics,
        &ControlState,
    ), With<ActiveEntity>>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<RealisticVehicle>)>,
    // rapier_context: Query<&RapierContext>, // TODO: Fix RapierContext integration
) {
    let start_time = std::time::Instant::now();
    let dt = time.delta_secs().clamp(0.001, 0.05);
    let max_processing_time = 5.0; // 5ms time budget
    
    // Get active entity position for distance calculations
    let active_pos = active_query.single().map(|t| t.translation).unwrap_or_default();
    
    // Sort vehicles by distance and importance for priority processing
    let mut vehicles_with_distance: Vec<_> = query.iter_mut()
        .map(|(entity, velocity, transform, vehicle, dynamics, engine, suspension, tire_physics, control_state)| {
            let distance = active_pos.distance(transform.translation);
            (distance, entity, velocity, transform, vehicle, dynamics, engine, suspension, tire_physics, control_state)
        })
        .collect();
    
    vehicles_with_distance.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut processed_count = 0;
    let max_physics_entities = 8; // Limit active physics simulations
    
    for (distance, _entity, mut velocity, mut transform, mut vehicle, mut dynamics, mut engine, mut suspension, mut tire_physics, control_state) in vehicles_with_distance {
        // Check time budget
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        // Skip physics for vehicles beyond 200m (state-only updates)
        if distance > 200.0 {
            vehicle.physics_enabled = false;
            continue;
        }
        
        // Use simplified physics for vehicles beyond 100m
        if distance > 100.0 {
            process_simplified_vehicle_physics(&mut velocity, &mut transform, &mut vehicle, dt);
            continue;
        }
        
        // Limit active physics simulations
        if processed_count >= max_physics_entities {
            vehicle.physics_enabled = false;
            continue;
        }
        
        // Performance check: Skip if physics disabled for LOD
        if !vehicle.physics_enabled {
            continue;
        }
        
        // CRITICAL: Validate all components before processing
        vehicle.validate_and_clamp();
        dynamics.validate_and_clamp();
        engine.validate_and_clamp();
        suspension.validate_and_clamp();
        tire_physics.validate_and_clamp();
        
        // Update vehicle speed for dynamics calculations
        dynamics.speed = velocity.linvel.length();
        
        // STEP 1: Process user input with realistic constraints
        process_vehicle_input(&control_state, &mut engine, &vehicle, dt);
        
        // STEP 2: Calculate engine forces with realistic power delivery
        let engine_force = calculate_engine_force(&mut engine, &dynamics, dt);
        
        // STEP 3: Calculate suspension forces (simplified without ground detection for now)
        let suspension_forces = Vec3::new(0.0, -dynamics.total_mass * 9.81, 0.0); // Basic gravity
        
        // STEP 4: Calculate tire forces with realistic friction model
        let tire_forces = calculate_tire_forces(
            &mut tire_physics,
            &suspension,
            &velocity,
            &transform,
            &dynamics,
            dt
        );
        
        // STEP 5: Calculate aerodynamic forces
        let aero_forces = calculate_aerodynamic_forces(&mut dynamics, &velocity);
        
        // STEP 6: Apply weight transfer effects
        calculate_weight_transfer(&mut dynamics, &velocity, &transform, dt);
        
        // STEP 7: Combine all forces and apply with safety checks
        apply_forces_to_vehicle(
            &mut velocity,
            &mut transform,
            engine_force,
            suspension_forces,
            tire_forces,
            aero_forces,
            &dynamics,
            dt
        );
        
        // STEP 8: Performance monitoring and safeguards
        apply_physics_safeguards(&mut velocity, &mut transform, &config);
        
        // Update last update time for performance tracking
        vehicle.last_update_time = time.elapsed_secs();
        
        processed_count += 1;
    }
    
    // Report performance metrics
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > 3.0 {
        warn!("Vehicle physics processing took {:.2}ms (> 3ms budget)", processing_time);
    }
}

/// Simplified physics processing for distant vehicles
fn process_simplified_vehicle_physics(
    velocity: &mut Velocity,
    transform: &mut Transform,
    vehicle: &mut RealisticVehicle,
    dt: f32,
) {
    // Simple damping and gravity
    velocity.linvel *= 0.995;
    velocity.angvel *= 0.98;
    velocity.linvel.y -= 9.81 * dt;
    
    // Basic ground collision
    if transform.translation.y < 0.1 {
        transform.translation.y = 0.1;
        velocity.linvel.y = velocity.linvel.y.max(0.0);
    }
    
    // Clamp velocity for stability
    if velocity.linvel.length() > 50.0 {
        velocity.linvel = velocity.linvel.normalize() * 50.0;
    }
    
    vehicle.physics_enabled = false;
}

/// Process realistic vehicle input with proper control systems
fn process_vehicle_input(
    control_state: &ControlState,
    engine: &mut EnginePhysics,
    _vehicle: &RealisticVehicle,
    dt: f32,
) {
    // Use ControlState for direct, clean vehicle input
    // Throttle input with realistic response
    if control_state.is_accelerating() {
        engine.throttle_input = (engine.throttle_input + dt * 2.0).clamp(0.0, 1.0);
    } else {
        engine.throttle_input = (engine.throttle_input - dt * 3.0).clamp(0.0, 1.0);
    }
    
    // Brake input with ABS simulation
    if control_state.is_braking() {
        engine.brake_input = (engine.brake_input + dt * 4.0).clamp(0.0, 1.0);
    } else {
        engine.brake_input = (engine.brake_input - dt * 5.0).clamp(0.0, 1.0);
    }
    
    // Automatic transmission simulation (simplified)
    if engine.current_gear == 0 && engine.throttle_input > 0.1 {
        engine.current_gear = 1; // Engage first gear
    }
    
    // Simple gear shifting based on RPM
    if engine.current_gear > 0 {
        if engine.current_rpm > engine.max_rpm * 0.85 && engine.current_gear < engine.gear_ratios.len() as i8 - 2 {
            engine.current_gear += 1; // Upshift
        } else if engine.current_rpm < engine.max_rpm * 0.3 && engine.current_gear > 1 {
            engine.current_gear -= 1; // Downshift
        }
    }
}

/// Calculate realistic engine force with torque curves
fn calculate_engine_force(
    engine: &mut EnginePhysics,
    dynamics: &VehicleDynamics,
    dt: f32,
) -> Vec3 {
    // Calculate target RPM based on gear and speed
    let gear_ratio = if engine.current_gear >= 0 && engine.current_gear < engine.gear_ratios.len() as i8 {
        engine.gear_ratios[engine.current_gear as usize + 1] // +1 to skip reverse gear at index 0
    } else {
        1.0
    };
    
    let wheel_radius = 0.35; // Average wheel radius
    let target_rpm = if engine.current_gear > 0 {
        (dynamics.speed / wheel_radius) * gear_ratio * engine.differential_ratio * 9.549 // Convert to RPM
    } else {
        engine.idle_rpm
    };
    
    // Engine RPM follows target with realistic spool time
    let rpm_change_rate = if target_rpm > engine.current_rpm { 2000.0 } else { 3000.0 };
    engine.current_rpm = engine.current_rpm.lerp(
        target_rpm.clamp(engine.idle_rpm, engine.max_rpm),
        dt * rpm_change_rate / engine.max_rpm
    );
    
    // Calculate engine torque from power curve
    let rpm_normalized = (engine.current_rpm - engine.idle_rpm) / (engine.max_rpm - engine.idle_rpm);
    let curve_index = (rpm_normalized * (engine.power_curve.len() - 1) as f32).floor() as usize;
    let power_multiplier = engine.power_curve.get(curve_index).unwrap_or(&0.5);
    
    let engine_torque = engine.max_torque * power_multiplier * engine.throttle_input;
    let engine_force_magnitude = if engine.current_gear > 0 {
        engine_torque * gear_ratio * engine.differential_ratio / wheel_radius
    } else {
        0.0
    };
    
    // Return force in forward direction, clamped for safety
    Vec3::new(0.0, 0.0, engine_force_magnitude.clamp(-10000.0, 10000.0))
}



/// Calculate tire forces with realistic friction model
fn calculate_tire_forces(
    tire_physics: &mut TirePhysics,
    suspension: &VehicleSuspension,
    velocity: &Velocity,
    transform: &Transform,
    dynamics: &VehicleDynamics,
    dt: f32,
) -> Vec3 {
    if !suspension.ground_contact {
        // No ground contact = no tire forces
        tire_physics.longitudinal_force = 0.0;
        tire_physics.lateral_force = 0.0;
        tire_physics.normal_force = 0.0;
        return Vec3::ZERO;
    }
    
    // Calculate normal force from weight distribution
    tire_physics.normal_force = (dynamics.total_mass * 9.81 * 0.25) + suspension.force * 0.5;
    
    // Calculate slip ratio (simplified)
    let wheel_speed = 0.35 * 10.0; // Simplified wheel speed calculation
    let vehicle_speed = velocity.linvel.length();
    
    tire_physics.slip_ratio = if vehicle_speed > 0.1 {
        (wheel_speed - vehicle_speed) / vehicle_speed.max(0.1)
    } else {
        0.0
    };
    tire_physics.slip_ratio = tire_physics.slip_ratio.clamp(-1.0, 1.0);
    
    // Calculate lateral slip angle (simplified)
    let forward_velocity = transform.forward().dot(velocity.linvel);
    let lateral_velocity = transform.right().dot(velocity.linvel);
    tire_physics.slip_angle = if forward_velocity.abs() > 0.1 {
        (lateral_velocity / forward_velocity.abs()).atan().clamp(-1.0, 1.0)
    } else {
        0.0
    };
    
    // Tire temperature affects grip (simplified)
    tire_physics.tire_temperature = (tire_physics.tire_temperature + dt * 
        (20.0 + tire_physics.slip_ratio.abs() * 50.0)).clamp(0.0, 150.0);
    
    let temp_multiplier = if tire_physics.tire_temperature > 80.0 {
        1.0 - (tire_physics.tire_temperature - 80.0) * 0.01
    } else {
        0.9 + tire_physics.tire_temperature * 0.00125
    }.clamp(0.5, 1.2);
    
    // Calculate longitudinal force (Pacejka-like simplified model)
    let max_long_force = tire_physics.normal_force * tire_physics.dry_grip * temp_multiplier;
    tire_physics.longitudinal_force = max_long_force * 
        (tire_physics.slip_ratio * 10.0).tanh() * tire_physics.wear_level;
    
    // Calculate lateral force
    let max_lat_force = tire_physics.normal_force * tire_physics.lateral_grip * temp_multiplier;
    tire_physics.lateral_force = max_lat_force * 
        (tire_physics.slip_angle * 5.0).tanh() * tire_physics.wear_level;
    
    // Combine forces in vehicle coordinate system
    let longitudinal = transform.forward() * tire_physics.longitudinal_force;
    let lateral = transform.right() * tire_physics.lateral_force;
    
    // Apply rolling resistance
    let rolling_resistance = -velocity.linvel.normalize_or_zero() * 
        tire_physics.rolling_resistance * tire_physics.normal_force;
    
    longitudinal + lateral + rolling_resistance
}

/// Calculate aerodynamic forces
fn calculate_aerodynamic_forces(
    dynamics: &mut VehicleDynamics,
    velocity: &Velocity,
) -> Vec3 {
    let speed_squared = velocity.linvel.length_squared();
    let air_density = 1.225; // kg/m³ at sea level
    
    // Drag force
    let drag_force = -velocity.linvel.normalize_or_zero() * 
        0.5 * air_density * dynamics.drag_coefficient * dynamics.frontal_area * speed_squared;
    
    // Downforce
    let downforce = Vec3::NEG_Y * 
        0.5 * air_density * dynamics.downforce_coefficient * dynamics.frontal_area * speed_squared;
    
    dynamics.aerodynamic_force = drag_force + downforce;
    dynamics.aerodynamic_force
}

/// Calculate weight transfer effects
fn calculate_weight_transfer(
    dynamics: &mut VehicleDynamics,
    velocity: &Velocity,
    _transform: &Transform,
    dt: f32,
) {
    // Longitudinal weight transfer (acceleration/braking)
    let accel = velocity.linvel.length() / dt.max(0.001);
    let long_transfer = accel * dynamics.center_of_gravity.y / 2.0;
    
    // Lateral weight transfer (cornering)
    let angular_accel = velocity.angvel.y / dt.max(0.001);
    let lat_transfer = angular_accel * dynamics.center_of_gravity.y / 2.0;
    
    dynamics.weight_transfer = Vec3::new(lat_transfer, 0.0, long_transfer);
}

/// Apply all forces to the vehicle with safety checks
fn apply_forces_to_vehicle(
    velocity: &mut Velocity,
    transform: &mut Transform,
    engine_force: Vec3,
    suspension_forces: Vec3,
    tire_forces: Vec3,
    aero_forces: Vec3,
    dynamics: &VehicleDynamics,
    dt: f32,
) {
    // Combine all forces
    let total_force = engine_force + suspension_forces + tire_forces + aero_forces;
    
    // Calculate acceleration (F = ma)
    let acceleration = total_force / dynamics.total_mass.max(100.0);
    
    // Apply linear acceleration with safety limits
    if acceleration.is_finite() && acceleration.length() < 200.0 {
        velocity.linvel += acceleration * dt;
    }
    
    // Apply gravity
    velocity.linvel += Vec3::new(0.0, -9.81, 0.0) * dt;
    
    // Calculate and apply torque for rotation
    let _steering_input = if transform.forward().dot(velocity.linvel.normalize_or_zero()) > 0.0 {
        // Forward motion
        if velocity.linvel.length() > 1.0 {
            // Steering only effective when moving
            velocity.angvel.y = tire_forces.cross(Vec3::Y).y * 0.0001;
        }
    } else {
        // Reverse motion - opposite steering
        if velocity.linvel.length() > 1.0 {
            velocity.angvel.y = -tire_forces.cross(Vec3::Y).y * 0.0001;
        }
    };
    
    // Apply damping to prevent physics instability
    velocity.linvel *= 0.999;
    velocity.angvel *= 0.995;
}

/// Apply critical physics safeguards to prevent instability (UNIFIED)
fn apply_physics_safeguards(
    velocity: &mut Velocity,
    transform: &mut Transform,
    config: &GameConfig,
) {
    // Use unified velocity validation
    PhysicsUtilities::validate_velocity(velocity, config);
    
    // Use unified ground collision system
    PhysicsUtilities::apply_ground_collision(velocity, transform, 0.1, 2.0);
    
    // Use unified world bounds system
    PhysicsUtilities::apply_world_bounds(transform, velocity, config);
}

/// System to update vehicle wheel positions and rotations
pub fn vehicle_wheel_update_system(
    time: Res<Time>,
    mut wheel_query: Query<(&mut Transform, &VehicleWheel, &ChildOf)>,
    vehicle_query: Query<(&Transform, &VehicleDynamics), (With<RealisticVehicle>, Without<VehicleWheel>)>,
) {
    let dt = time.delta_secs();
    
    for (mut wheel_transform, wheel, child_of) in wheel_query.iter_mut() {
        if let Ok((_vehicle_transform, dynamics)) = vehicle_query.get(child_of.0) {
            // Update wheel rotation based on vehicle speed
            let rotation_speed = dynamics.speed / wheel.radius;
            wheel_transform.rotate_local_x(rotation_speed * dt);
            
            // Update wheel position relative to vehicle
            wheel_transform.translation = wheel.position;
        }
    }
}

/// Performance monitoring system for realistic vehicle physics
pub fn realistic_vehicle_performance_system(
    time: Res<Time>,
    query: Query<&RealisticVehicle, With<ActiveEntity>>,
    mut last_report: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    if current_time - *last_report > 10.0 {
        *last_report = current_time;
        let active_vehicles = query.iter().filter(|v| v.physics_enabled).count();
        info!("REALISTIC PHYSICS: {} active vehicles with full physics", active_vehicles);
    }
}
