//! ───────────────────────────────────────────────
//! System:   Vehicle Movement System
//! Purpose:  Handles realistic vehicle physics and movement for cars, supercars, and aircraft
//! Schedule: Update (continuous)
//! Reads:    ControlManager, GameConfig, Time, Velocity, Transform, vehicle components
//! Writes:   Velocity, vehicle physics state, audio events
//! Invariants:
//!   * Vehicle physics respect mass and drag properties
//!   * Only active vehicle can be controlled
//!   * Physics values are validated and clamped
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use crate::systems::input::{ControlManager, ControlAction};
use crate::systems::physics_utils::PhysicsUtilities;

pub fn car_movement(
    control_manager: Res<ControlManager>,
    config: Res<GameConfig>,
    mut car_query: Query<(&mut Velocity, &Transform), (With<Car>, With<ActiveEntity>, Without<SuperCar>)>,
    _time: Res<Time>,
) {
    let start_time = std::time::Instant::now();
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
        target_linear_velocity -= forward * speed * brake_value;
    // Steering (only when moving)
    if control_manager.is_control_active(ControlAction::Accelerate) || control_manager.is_control_active(ControlAction::Brake) {
        let steering = control_manager.get_control_value(ControlAction::Steer);
        if steering.abs() > 0.1 {
            target_angular_velocity.y = steering * rotation_speed;
        }
    // Emergency brake override
    if control_manager.is_emergency_active() {
        target_linear_velocity *= 0.1;
        target_angular_velocity *= 0.5;
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
    // Apply unified physics safety systems
    PhysicsUtilities::validate_velocity(&mut velocity, &config);
    PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 0.1, 1.0);
    // Performance monitoring
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > 1.0 {
        warn!("Car movement took {:.2}ms (> 1ms budget)", processing_time);
}
// DEPRECATED: This system has been split into focused sub-systems:
// - supercar_input_system (handles input processing)
// - supercar_physics_system (handles physics calculations)
// - supercar_effects_system (handles visual effects using pre-spawned entities)
// The new systems provide better performance and maintainability.
pub fn supercar_movement_deprecated(
    time: Res<Time>,
    mut supercar_query: Query<(&mut Velocity, &Transform, &mut SuperCar), (With<Car>, With<ActiveEntity>, With<SuperCar>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    let Ok((mut velocity, transform, mut supercar)) = supercar_query.single_mut() else {
    let dt = time.delta_secs();
    supercar.exhaust_timer += dt;
    supercar.performance_timer += dt;
    supercar.backfire_timer += dt;
    // Calculate current speed in mph for realistic physics
    let current_speed_ms = velocity.linvel.length();
    let current_speed_mph = current_speed_ms * 2.237; // Convert m/s to mph
    // STEP 1: Handle driving mode changes
    handle_driving_mode_changes(&mut supercar, &control_manager);
    // STEP 2: Enhanced gear simulation and RPM calculation
    update_gear_and_rpm(&mut supercar, current_speed_mph, dt);
    // STEP 3: Advanced quad-turbo system
    let turbo_requested = control_manager.is_control_active(ControlAction::Turbo);
    update_advanced_turbo_system(&mut supercar, dt, turbo_requested, current_speed_mph);
    // STEP 4: Launch control system
    handle_launch_control(&mut supercar, &control_manager, current_speed_mph);
    // STEP 5: Enhanced traction control
    update_enhanced_traction_control(&mut supercar, current_speed_mph, dt);
    // STEP 6: Active aerodynamics
    update_active_aerodynamics(&mut supercar, current_speed_mph, dt);
    // STEP 7: Calculate G-forces and performance metrics
    calculate_performance_metrics(&mut supercar, &velocity, dt);
    // STEP 8: Enhanced power calculation with driving modes
    let power_multiplier = calculate_enhanced_power_curve(&supercar);
    let turbo_multiplier = calculate_turbo_multiplier(&supercar);
    let driving_mode_multiplier = get_driving_mode_multiplier(&supercar);
    let effective_power = supercar.power * power_multiplier * turbo_multiplier * driving_mode_multiplier;
    // STEP 9: Realistic acceleration physics with launch control
    let acceleration_ms2 = calculate_acceleration_force(&supercar, effective_power, current_speed_ms);
    let max_acceleration = (acceleration_ms2 * supercar.current_traction).min(supercar.acceleration);
    // STEP 10: Enhanced aerodynamic drag with active aero
    let drag_force = calculate_aerodynamic_drag(&supercar, current_speed_ms);
    let drag_deceleration = drag_force / supercar.weight;
    let mut target_linear_velocity = velocity.linvel;
    // STEP 11: Advanced acceleration with launch control
        let acceleration_force = forward * max_acceleration * accel_value;
        target_linear_velocity += acceleration_force * dt;
        
        // Apply drag resistance
        let drag_resistance = -velocity.linvel.normalize_or_zero() * drag_deceleration * dt;
        target_linear_velocity += drag_resistance;
        // Spawn hypercar exhaust effects
        spawn_hypercar_exhaust_effects(&mut commands, &mut meshes, &mut materials, &supercar, transform, dt);
    } else if control_manager.is_control_active(ControlAction::Brake) {
        // Advanced braking with brake-by-wire system
        let braking_force = calculate_braking_force(&supercar, brake_value, current_speed_mph);
        let brake_deceleration = transform.forward() * -braking_force * dt;
        target_linear_velocity += brake_deceleration;
    } else {
        // Natural deceleration with engine braking
        let natural_deceleration = calculate_natural_deceleration(&supercar, current_speed_ms);
        target_linear_velocity *= natural_deceleration;
        target_linear_velocity *= 0.85; // More aggressive emergency braking
    // STEP 12: Hypercar steering with advanced stability systems
    let steering_input = control_manager.get_control_value(ControlAction::Steer);
    if steering_input != 0.0 {
        target_angular_velocity = calculate_hypercar_steering(&supercar, steering_input, current_speed_mph);
    // STEP 13: Speed limiter and physics validation
    if target_linear_velocity.length() > supercar.max_speed / 2.237 {
        target_linear_velocity = target_linear_velocity.normalize() * (supercar.max_speed / 2.237);
    // STEP 14: Apply enhanced suspension damping
    let velocity_change = target_linear_velocity - velocity.linvel;
    let damped_change = apply_suspension_damping(&supercar, velocity_change, dt);
    target_linear_velocity = velocity.linvel + damped_change;
    // STEP 15: Engine temperature and protection systems
    update_engine_protection(&mut supercar, dt, current_speed_mph);
    // Set final velocity
fn spawn_hypercar_exhaust_effects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    supercar: &SuperCar,
    transform: &Transform,
    _dt: f32,
    if supercar.exhaust_timer > 0.04 { // Even more frequent for hypercar
        let exhaust_pos = transform.translation + transform.back() * 2.8 + Vec3::new(0.0, 0.15, 0.0);
        // Quad exhaust for W16 engine (4 tailpipes)
        for i in 0..4 {
            let side_offset = match i {
                0 => Vec3::new(-0.6, 0.0, 0.0),  // Left outer
                1 => Vec3::new(-0.2, 0.0, 0.0),  // Left inner
                2 => Vec3::new(0.2, 0.0, 0.0),   // Right inner
                3 => Vec3::new(0.6, 0.0, 0.0),   // Right outer
                _ => Vec3::ZERO,
            };
            let final_pos = exhaust_pos + transform.right() * side_offset.x;
            
            let (flame_color, emission_intensity) = if supercar.turbo_boost && supercar.turbo_stage >= 3 {
                // Quad-turbo plasma flames - intense blue-white
                (Color::srgb(0.1, 0.4, 1.0), 4.5)
            } else if supercar.turbo_boost && supercar.turbo_stage >= 2 {
                // Dual-turbo blue flames
                (Color::srgb(0.3, 0.7, 1.0), 3.2)
            } else if supercar.turbo_boost {
                // Single turbo blue-orange
                (Color::srgb(0.6, 0.8, 1.0), 2.8)
            } else if supercar.rpm > 5500.0 {
                // High RPM red-orange flames (near redline)
                (Color::srgb(1.0, 0.2, 0.0), 2.5)
            } else if supercar.rpm > 4000.0 {
                // Medium RPM orange flames
                (Color::srgb(1.0, 0.4, 0.1), 1.8)
            } else {
                // Normal exhaust flames
                (Color::srgb(1.0, 0.6, 0.2), 1.0)
            // Primary flame
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.15))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: flame_color,
                    emissive: LinearRgba::rgb(
                        flame_color.to_linear().red * emission_intensity,
                        flame_color.to_linear().green * emission_intensity,
                        flame_color.to_linear().blue * emission_intensity,
                    ),
                    alpha_mode: AlphaMode::Add,
                    ..default()
                })),
                Transform::from_translation(final_pos),
                ExhaustFlame,
            ));
            // Secondary flame trail (for turbo mode)
            if supercar.turbo_boost {
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.08))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.4, 0.8, 1.0),
                        emissive: LinearRgba::rgb(0.3, 0.6, 0.9),
                        alpha_mode: AlphaMode::Add,
                        ..default()
                    })),
                    Transform::from_translation(final_pos + transform.back() * 0.3),
                    ExhaustFlame,
                ));
            }
            // Spark effects for high RPM
            if supercar.rpm > 5000.0 {
                    Mesh3d(meshes.add(Sphere::new(0.04))),
                        base_color: Color::srgb(1.0, 0.9, 0.4),
                        emissive: LinearRgba::rgb(2.0, 1.8, 0.8),
                    Transform::from_translation(final_pos + Vec3::new(
                        (i as f32 - 1.5) * 0.1,
                        0.05,
                        -0.2,
                    )),
        // Heat shimmer effect for turbo mode
        if supercar.turbo_boost {
                Mesh3d(meshes.add(Sphere::new(0.25))),
                    base_color: Color::srgba(0.2, 0.3, 0.5, 0.3),
                    emissive: LinearRgba::rgb(0.1, 0.2, 0.4),
                    alpha_mode: AlphaMode::Blend,
                Transform::from_translation(exhaust_pos + transform.back() * 0.5),
// HYPERCAR SYSTEMS - Enhanced functions for ultimate performance
fn handle_driving_mode_changes(supercar: &mut SuperCar, _control_manager: &ControlManager) {
    // Driving mode changes based on control inputs
    // TODO: Add dedicated key binding for driving mode cycling
    // For now, driving mode cycling is disabled in this demo
    if false {
        // Cycle through driving modes (placeholder - would need dedicated key)
        supercar.driving_mode = match supercar.driving_mode {
            DrivingMode::Comfort => DrivingMode::Sport,
            DrivingMode::Sport => DrivingMode::Track,
            DrivingMode::Track => DrivingMode::Comfort,
            DrivingMode::Custom => DrivingMode::Sport,
        };
    // Update mode flags
    supercar.sport_mode_active = matches!(supercar.driving_mode, DrivingMode::Sport | DrivingMode::Track);
    supercar.track_mode_active = matches!(supercar.driving_mode, DrivingMode::Track);
fn update_gear_and_rpm(supercar: &mut SuperCar, current_speed_mph: f32, dt: f32) {
    // Calculate target RPM based on current gear and speed
    let gear_ratio = supercar.gear_ratios.get((supercar.gear - 1) as usize).unwrap_or(&1.0);
    let target_rpm = (current_speed_mph * 60.0 * gear_ratio).min(supercar.max_rpm);
    // Smooth RPM changes
    let rpm_change_rate = 4000.0; // RPM per second
    let rpm_diff = target_rpm - supercar.rpm;
    let rpm_change = rpm_diff.signum() * (rpm_diff.abs().min(rpm_change_rate * dt));
    supercar.rpm = (supercar.rpm + rpm_change).clamp(supercar.idle_rpm, supercar.max_rpm);
    // Automatic gear shifting (7-speed dual-clutch)
    if supercar.rpm >= supercar.shift_rpm && supercar.gear < 7 {
        supercar.gear += 1;
        supercar.rpm = supercar.rpm * 0.7; // RPM drop on upshift
    } else if supercar.rpm <= supercar.downshift_rpm && supercar.gear > 1 {
        supercar.gear -= 1;
        supercar.rpm = supercar.rpm * 1.3; // RPM increase on downshift
fn update_advanced_turbo_system(supercar: &mut SuperCar, dt: f32, turbo_requested: bool, current_speed_mph: f32) {
    if turbo_requested && supercar.turbo_cooldown <= 0.0 && current_speed_mph > 5.0 {
        // Progressive turbo staging (quad-turbo W16)
        supercar.turbo_pressure += dt * supercar.turbo_pressure_buildup;
        supercar.turbo_pressure = supercar.turbo_pressure.min(1.0);
        // Determine number of active turbos based on pressure
        supercar.turbo_stage = match supercar.turbo_pressure {
            p if p < 0.25 => 1,
            p if p < 0.5 => 2,
            p if p < 0.75 => 3,
            _ => 4,
        supercar.turbo_boost = supercar.turbo_pressure > 0.2;
            supercar.current_turbo_time += dt;
            // Overheat protection with progressive cooling
            if supercar.current_turbo_time >= supercar.max_turbo_time {
                supercar.turbo_cooldown = 6.0 + (supercar.turbo_stage as f32 * 1.5); // Longer cooldown for more turbos
                supercar.current_turbo_time = 0.0;
        // Turbo decay with realistic lag
        supercar.turbo_pressure -= dt * 1.5; // Realistic decay rate
        supercar.turbo_pressure = supercar.turbo_pressure.max(0.0);
        supercar.turbo_boost = false;
        supercar.turbo_stage = 0;
        if supercar.turbo_cooldown > 0.0 {
            supercar.turbo_cooldown -= dt;
        if supercar.current_turbo_time > 0.0 && !turbo_requested {
            supercar.current_turbo_time -= dt * 0.3; // Gradual cooldown
            supercar.current_turbo_time = supercar.current_turbo_time.max(0.0);
    // Update turbo whistle intensity for audio
    supercar.turbo_whistle_intensity = supercar.turbo_pressure * (supercar.turbo_stage as f32 * 0.25);
fn handle_launch_control(supercar: &mut SuperCar, control_manager: &ControlManager, current_speed_mph: f32) {
    // Launch control activation (both brake and accelerate pressed)
    if control_manager.is_control_active(ControlAction::Brake) && 
       control_manager.is_control_active(ControlAction::Accelerate) &&
       current_speed_mph < 5.0 {
        supercar.launch_control_engaged = true;
        supercar.rpm = supercar.launch_rpm_limit; // Hold at launch RPM
        supercar.is_timing_launch = true;
        supercar.zero_to_sixty_time = 0.0;
    } else if supercar.launch_control_engaged && !control_manager.is_control_active(ControlAction::Brake) {
        // Launch control release
        supercar.launch_control_engaged = false;
    // Track 0-60 time
    if supercar.is_timing_launch && current_speed_mph >= 60.0 {
        supercar.is_timing_launch = false;
        info!("🏁 CHIRON 0-60: {:.2}s", supercar.zero_to_sixty_time);
fn update_enhanced_traction_control(supercar: &mut SuperCar, current_speed_mph: f32, dt: f32) {
    if supercar.traction_control {
        // Enhanced traction control with multiple driving modes
        let optimal_traction = match supercar.driving_mode {
            DrivingMode::Comfort => 0.95, // Maximum grip
            DrivingMode::Sport => 0.88,   // Allow some slip for performance
            DrivingMode::Track => 0.85,   // Minimal intervention
            DrivingMode::Custom => 0.90,  // Balanced
        // Speed-dependent traction adjustment
        let speed_factor = if current_speed_mph < 20.0 {
            0.85 // Lower traction during launch
        } else if current_speed_mph < 60.0 {
            0.92 // Good traction in mid-range
        } else {
            1.0  // Maximum traction at high speed
        let target_traction = optimal_traction * speed_factor;
        // Fast traction adjustment (hypercar has advanced systems)
        let traction_diff = target_traction - supercar.current_traction;
        supercar.current_traction += traction_diff * dt * 5.0; // 0.2 second response
        supercar.current_traction = supercar.current_traction.clamp(0.6, 1.0);
        // Without traction control, more wheel spin possible
        supercar.current_traction = 0.75;
fn update_active_aerodynamics(supercar: &mut SuperCar, current_speed_mph: f32, dt: f32) {
    if supercar.active_aero {
        // Active rear wing deployment
        let target_wing_angle = if current_speed_mph > 150.0 {
            0.8 // High downforce for high speed
        } else if current_speed_mph > 80.0 {
            0.4 // Moderate downforce for medium speed
            0.0 // Minimum drag for low speed
        // Smooth wing adjustment
        let wing_diff = target_wing_angle - supercar.rear_wing_angle;
        supercar.rear_wing_angle += wing_diff * dt * 2.0; // 0.5 second adjustment
        supercar.rear_wing_angle = supercar.rear_wing_angle.clamp(0.0, 1.0);
        // Calculate downforce (affects traction and handling)
        supercar.downforce = supercar.rear_wing_angle * current_speed_mph * current_speed_mph * 0.05;
        // Front splitter adjustment
        supercar.front_splitter_level = (current_speed_mph / 100.0).min(1.0);
fn calculate_performance_metrics(supercar: &mut SuperCar, velocity: &Velocity, dt: f32) {
    // Calculate G-forces
    let acceleration = velocity.linvel.length() / dt.max(0.001);
    supercar.g_force_longitudinal = acceleration / 9.81; // Convert to G
    // Lateral G-force approximation
    let angular_speed = velocity.angvel.y.abs();
    let lateral_accel = angular_speed * velocity.linvel.length();
    supercar.g_force_lateral = lateral_accel / 9.81;
    // Performance timing
    if supercar.is_timing_launch {
        supercar.zero_to_sixty_time += dt;
fn calculate_enhanced_power_curve(supercar: &SuperCar) -> f32 {
    // More realistic power curve with gear consideration
    let _rpm_ratio = (supercar.rpm - supercar.idle_rpm) / (supercar.max_rpm - supercar.idle_rpm);
    if supercar.rpm < supercar.power_band_start {
        // Below power band - turbo lag simulation
        0.5 + 0.5 * (supercar.rpm / supercar.power_band_start)
    } else if supercar.rpm <= supercar.power_band_end {
        // In power band - peak power with slight variation
        0.95 + 0.05 * (1.0 - (supercar.rpm - supercar.power_band_start) / (supercar.power_band_end - supercar.power_band_start))
        // Above power band - power drops off more gradually
        1.0 - 0.2 * ((supercar.rpm - supercar.power_band_end) / (supercar.max_rpm - supercar.power_band_end))
fn calculate_turbo_multiplier(supercar: &SuperCar) -> f32 {
    if supercar.turbo_boost {
        // Progressive turbo boost based on number of active turbos
        1.0 + (supercar.turbo_pressure * supercar.turbo_stage as f32 * 0.2)
        1.0
fn get_driving_mode_multiplier(supercar: &SuperCar) -> f32 {
    match supercar.driving_mode {
        DrivingMode::Comfort => 0.75, // Reduced power for comfort
        DrivingMode::Sport => 0.95,   // Near-full power
        DrivingMode::Track => 1.0,    // Maximum power
        DrivingMode::Custom => 0.85,  // Customizable (placeholder)
fn calculate_acceleration_force(supercar: &SuperCar, effective_power: f32, current_speed_ms: f32) -> f32 {
    // Enhanced acceleration with launch control
    let base_acceleration = (effective_power * 745.7) / (supercar.weight * current_speed_ms.max(0.5));
    if supercar.launch_control_engaged {
        // Launch control limits torque to prevent wheel spin
        base_acceleration * 0.85
        base_acceleration
fn calculate_aerodynamic_drag(supercar: &SuperCar, current_speed_ms: f32) -> f32 {
    // Enhanced drag calculation with active aerodynamics
    let frontal_area = 2.2; // m² - Chiron frontal area
    let air_density = 1.225; // kg/m³ - sea level air density
    // Dynamic drag coefficient based on active aero
    let dynamic_drag_coeff = supercar.drag_coefficient + (supercar.rear_wing_angle * 0.1);
    0.5 * air_density * dynamic_drag_coeff * frontal_area * current_speed_ms.powi(2)
fn calculate_braking_force(supercar: &SuperCar, brake_value: f32, current_speed_mph: f32) -> f32 {
    // Enhanced braking with brake-by-wire and regenerative braking
    let base_braking = supercar.acceleration * 1.8; // Stronger brakes than acceleration
    // Speed-dependent braking efficiency
    let speed_factor = if current_speed_mph > 100.0 {
        1.2 // More effective at high speed (aerodynamic braking)
    base_braking * brake_value * speed_factor
fn calculate_natural_deceleration(supercar: &SuperCar, _current_speed_ms: f32) -> f32 {
    // Natural deceleration with enhanced engine braking
    let base_decel = 0.98;
    // Engine braking based on gear
    let engine_braking = 1.0 - (supercar.gear as f32 * 0.005); // Lower gears have more engine braking
    base_decel * engine_braking
fn calculate_hypercar_steering(supercar: &SuperCar, steering_input: f32, current_speed_mph: f32) -> Vec3 {
    // Enhanced steering with speed-dependent response
    let speed_factor = (current_speed_mph / 80.0).min(1.2); // Slight increase at very high speed
    let base_rotation_speed = match supercar.driving_mode {
        DrivingMode::Comfort => 3.8,
        DrivingMode::Sport => 4.5,
        DrivingMode::Track => 5.2,
        DrivingMode::Custom => 4.0,
    // Reduce sensitivity at high speed but maintain precision
    let speed_adjusted_rotation = base_rotation_speed * (1.0 - speed_factor * 0.4);
    // Weight distribution and downforce effects
    let handling_modifier = 1.0 + (0.5 - supercar.front_weight_bias) * 0.3;
    let downforce_modifier = 1.0 + (supercar.downforce * 0.001); // Downforce improves handling
    // Traction affects steering effectiveness
    let steering_effectiveness = supercar.current_traction * handling_modifier * downforce_modifier;
    let mut angular_velocity = Vec3::new(0.0, steering_input * speed_adjusted_rotation * steering_effectiveness, 0.0);
    // Advanced stability control systems
    if supercar.stability_control {
        let stability_factor = match supercar.driving_mode {
            DrivingMode::Comfort => 0.6, // Maximum stability intervention
            DrivingMode::Sport => 0.8,   // Moderate intervention
            DrivingMode::Track => 0.95,  // Minimal intervention
            DrivingMode::Custom => 0.75, // Balanced
        if current_speed_mph > 40.0 {
            angular_velocity.y *= stability_factor;
    angular_velocity
fn apply_suspension_damping(supercar: &SuperCar, velocity_change: Vec3, dt: f32) -> Vec3 {
    // Enhanced suspension with adaptive damping
    let damping_factor = match supercar.driving_mode {
        DrivingMode::Comfort => supercar.suspension_damping * 0.8, // Softer damping
        DrivingMode::Sport => supercar.suspension_damping,         // Standard damping
        DrivingMode::Track => supercar.suspension_damping * 1.2,   // Stiffer damping
        DrivingMode::Custom => supercar.suspension_damping,        // Customizable
    velocity_change * (1.0 - damping_factor * dt)
fn update_engine_protection(supercar: &mut SuperCar, dt: f32, _current_speed_mph: f32) {
    // Engine temperature management
    let heat_generation = supercar.rpm / supercar.max_rpm * 0.1 * dt;
    let cooling_rate = 0.05 * dt;
    supercar.engine_temperature += heat_generation - cooling_rate;
    supercar.engine_temperature = supercar.engine_temperature.clamp(0.0, 1.0);
    // Rev limiter
    if supercar.rpm >= supercar.max_rpm * 0.98 {
        supercar.rev_limiter_active = true;
    } else if supercar.rpm <= supercar.max_rpm * 0.95 {
        supercar.rev_limiter_active = false;
    // Oil pressure simulation
    let target_oil_pressure = (supercar.rpm / supercar.max_rpm * 0.5 + 0.5).min(1.0);
    supercar.oil_pressure += (target_oil_pressure - supercar.oil_pressure) * dt * 3.0;
    supercar.oil_pressure = supercar.oil_pressure.clamp(0.0, 1.0);
