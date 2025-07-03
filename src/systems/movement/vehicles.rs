use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Car, SuperCar, ActiveEntity, ExhaustFlame};
use crate::systems::input::{ControlManager, ControlAction};
use crate::systems::physics_utils::PhysicsUtilities;
use crate::config::GameConfig;

pub fn car_movement(
    control_manager: Res<ControlManager>,
    config: Res<GameConfig>,
    mut car_query: Query<(&mut Velocity, &Transform), (With<Car>, With<ActiveEntity>, Without<SuperCar>)>,
) {
    let Ok((mut velocity, transform)) = car_query.single_mut() else {
        return;
    };

    let speed = 25.0;
    let rotation_speed = 2.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Use UNIFIED ControlManager controls
    if control_manager.is_control_active(ControlAction::Accelerate) {
        let accel_value = control_manager.get_control_value(ControlAction::Accelerate);
        let forward = transform.forward();
        target_linear_velocity += forward * speed * accel_value;
    }
    
    if control_manager.is_control_active(ControlAction::Brake) {
        let brake_value = control_manager.get_control_value(ControlAction::Brake);
        let forward = transform.forward();
        target_linear_velocity -= forward * speed * brake_value;
    }
    
    // Steering (only when moving)
    if control_manager.is_control_active(ControlAction::Accelerate) || control_manager.is_control_active(ControlAction::Brake) {
        let steering = control_manager.get_control_value(ControlAction::Steer);
        if steering.abs() > 0.1 {
            target_angular_velocity.y = steering * rotation_speed;
        }
    }
    
    // Emergency brake override
    if control_manager.is_emergency_active() {
        target_linear_velocity *= 0.1;
        target_angular_velocity *= 0.5;
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
    
    // Apply unified physics safety systems
    PhysicsUtilities::validate_velocity(&mut velocity, &config);
    PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 0.1, 1.0);
}

pub fn supercar_movement(
    time: Res<Time>,
    control_manager: Res<ControlManager>,
    config: Res<GameConfig>,
    mut supercar_query: Query<(&mut Velocity, &Transform, &mut SuperCar), (With<Car>, With<ActiveEntity>, With<SuperCar>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((mut velocity, transform, mut supercar)) = supercar_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    supercar.exhaust_timer += dt;
    
    // Calculate current speed in mph for realistic physics
    let current_speed_ms = velocity.linvel.length();
    let current_speed_mph = current_speed_ms * 2.237; // Convert m/s to mph
    
    // Update engine RPM based on speed and gear simulation
    let speed_ratio = current_speed_mph / supercar.max_speed;
    supercar.rpm = supercar.idle_rpm + (supercar.max_rpm - supercar.idle_rpm) * speed_ratio;
    
    // Advanced turbo system - use UNIFIED ControlManager
    let turbo_requested = control_manager.is_control_active(ControlAction::Turbo);
    update_turbo_system(&mut supercar, dt, turbo_requested, current_speed_mph);
    
    // Traction control system
    update_traction_control(&mut supercar, current_speed_mph, dt);
    
    // Calculate power output based on RPM and turbo
    let power_multiplier = calculate_power_curve(&supercar);
    let turbo_multiplier = if supercar.turbo_boost { 1.0 + supercar.turbo_pressure * 0.6 } else { 1.0 };
    let effective_power = supercar.power * power_multiplier * turbo_multiplier;
    
    // Realistic acceleration physics (power-to-weight ratio)
    let acceleration_ms2 = (effective_power * 745.7) / (supercar.weight * current_speed_ms.max(1.0)); // HP to watts conversion
    let max_acceleration = (acceleration_ms2 * supercar.current_traction).min(supercar.acceleration);
    
    // Aerodynamic drag calculation
    let drag_force = 0.5 * 1.225 * supercar.drag_coefficient * 2.5 * current_speed_ms.powi(2); // Air density * drag coeff * frontal area * v²
    let drag_deceleration = drag_force / supercar.weight;
    
    let mut target_linear_velocity = velocity.linvel;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Forward/backward movement with realistic physics - use ControlManager
    if control_manager.is_control_active(ControlAction::Accelerate) {
        let accel_value = control_manager.get_control_value(ControlAction::Accelerate);
        let forward = transform.forward();
        let acceleration_force = forward * max_acceleration * accel_value;
        target_linear_velocity += acceleration_force * dt;
        
        // Apply drag resistance
        let drag_resistance = -velocity.linvel.normalize_or_zero() * drag_deceleration * dt;
        target_linear_velocity += drag_resistance;
        
        // Spawn enhanced exhaust effects
        spawn_exhaust_effects(&mut commands, &mut meshes, &mut materials, &supercar, transform, dt);
        
    } else if control_manager.is_control_active(ControlAction::Brake) {
        // Braking with advanced brake system
        let brake_value = control_manager.get_control_value(ControlAction::Brake);
        let braking_force = supercar.acceleration * 1.5 * brake_value;
        let brake_deceleration = transform.forward() * -braking_force * dt;
        target_linear_velocity += brake_deceleration;
    } else {
        // Natural deceleration from drag and friction
        let natural_deceleration = -velocity.linvel * 0.98;
        target_linear_velocity = natural_deceleration;
    }
    
    // Emergency brake override
    if control_manager.is_emergency_active() {
        target_linear_velocity *= 0.9;
    }
    
    // Advanced steering with stability control - use UNIFIED ControlManager
    let steering_input = control_manager.get_control_value(ControlAction::Steer);
    
    if steering_input != 0.0 {
        // Speed-dependent steering sensitivity
        let speed_factor = (current_speed_mph / 60.0).min(1.0); // Reduce sensitivity at high speed
        let base_rotation_speed = 4.5 * (1.0 - speed_factor * 0.6);
        
        // Weight distribution affects handling
        let handling_modifier = 1.0 + (0.5 - supercar.front_weight_bias) * 0.3;
        
        // Traction affects steering effectiveness
        let steering_effectiveness = supercar.current_traction * handling_modifier;
        
        target_angular_velocity.y = steering_input * base_rotation_speed * steering_effectiveness;
        
        // Stability control - reduce oversteer
        if supercar.stability_control && current_speed_mph > 30.0 {
            target_angular_velocity.y *= 0.8;
        }
    }
    
    // Enforce maximum speed limit
    if target_linear_velocity.length() > supercar.max_speed / 2.237 {
        target_linear_velocity = target_linear_velocity.normalize() * (supercar.max_speed / 2.237);
    }
    
    // Apply suspension damping to velocity changes
    let velocity_change = target_linear_velocity - velocity.linvel;
    let damped_change = velocity_change * (1.0 - supercar.suspension_damping * dt);
    target_linear_velocity = velocity.linvel + damped_change;
    
    // Set final velocity
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
    
    // Apply unified physics safety systems
    PhysicsUtilities::validate_velocity(&mut velocity, &config);
    PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 0.1, 1.0);
}

fn update_turbo_system(supercar: &mut SuperCar, dt: f32, turbo_requested: bool, current_speed_mph: f32) {
    if turbo_requested && supercar.turbo_cooldown <= 0.0 && current_speed_mph > 10.0 {
        // Build turbo pressure with lag
        if supercar.turbo_pressure < 1.0 {
            supercar.turbo_pressure += dt / supercar.turbo_lag;
            supercar.turbo_pressure = supercar.turbo_pressure.min(1.0);
        }
        
        supercar.turbo_boost = supercar.turbo_pressure > 0.3;
        
        if supercar.turbo_boost {
            supercar.current_turbo_time += dt;
            
            // Overheat protection
            if supercar.current_turbo_time >= supercar.max_turbo_time {
                supercar.turbo_cooldown = 8.0; // 8 second cooldown
                supercar.current_turbo_time = 0.0;
            }
        }
    } else {
        // Turbo decay
        supercar.turbo_pressure -= dt * 2.0; // Faster decay than buildup
        supercar.turbo_pressure = supercar.turbo_pressure.max(0.0);
        supercar.turbo_boost = false;
        
        if supercar.turbo_cooldown > 0.0 {
            supercar.turbo_cooldown -= dt;
        }
        
        if supercar.current_turbo_time > 0.0 && !turbo_requested {
            supercar.current_turbo_time -= dt * 0.5; // Gradual cooldown when not using
            supercar.current_turbo_time = supercar.current_turbo_time.max(0.0);
        }
    }
}

fn update_traction_control(supercar: &mut SuperCar, current_speed_mph: f32, dt: f32) {
    if supercar.traction_control {
        // Simulate wheel spin detection and correction
        let optimal_traction = if current_speed_mph < 30.0 {
            0.85 // Lower traction during launch
        } else if current_speed_mph < 60.0 {
            0.95 // Good traction in mid-range
        } else {
            1.0  // Maximum traction at high speed
        };
        
        // Gradual traction adjustment
        let traction_diff = optimal_traction - supercar.current_traction;
        supercar.current_traction += traction_diff * dt * 3.0; // 3 second response time
        supercar.current_traction = supercar.current_traction.clamp(0.5, 1.0);
    } else {
        // Without traction control, more wheel spin possible
        supercar.current_traction = 0.8;
    }
}

fn calculate_power_curve(supercar: &SuperCar) -> f32 {
    // Realistic power curve based on RPM
    if supercar.rpm < supercar.power_band_start {
        // Below power band - reduced power
        0.6 + 0.4 * (supercar.rpm / supercar.power_band_start)
    } else if supercar.rpm <= supercar.power_band_end {
        // In power band - maximum power
        1.0
    } else {
        // Above power band - power drops off
        1.0 - 0.3 * ((supercar.rpm - supercar.power_band_end) / (supercar.max_rpm - supercar.power_band_end))
    }
}

fn spawn_exhaust_effects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    supercar: &SuperCar,
    transform: &Transform,
    _dt: f32,
) {
    if supercar.exhaust_timer > 0.08 { // More frequent exhaust for hypercar
        let exhaust_pos = transform.translation + transform.back() * 2.8 + Vec3::new(0.0, 0.15, 0.0);
        
        // Dual exhaust for hypercar
        for i in 0..2 {
            let side_offset = if i == 0 { Vec3::new(-0.4, 0.0, 0.0) } else { Vec3::new(0.4, 0.0, 0.0) };
            let final_pos = exhaust_pos + transform.right() * side_offset.x;
            
            let flame_color = if supercar.turbo_boost {
                Color::srgb(0.1, 0.3, 1.0) // Blue turbo flames
            } else {
                Color::srgb(1.0, 0.4, 0.1) // Orange flames
            };
            
            let emission_intensity = if supercar.turbo_boost { 2.0 } else { 1.0 };
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.12))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: flame_color,
                    emissive: LinearRgba::rgb(
                        flame_color.to_linear().red * emission_intensity,
                        flame_color.to_linear().green * emission_intensity,
                        flame_color.to_linear().blue * emission_intensity,
                    ),
                    ..default()
                })),
                Transform::from_translation(final_pos),
                ExhaustFlame,
            ));
        }
    }
}
