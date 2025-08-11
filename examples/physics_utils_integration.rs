// Example integration showing how to use the new unified physics utilities
// This demonstrates the extracted common patterns from movement systems

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use gta_game::systems::{PhysicsUtilities, CollisionGroupHelper, PhysicsBodySetup, InputProcessor};
use gta_game::config::GameConfig;

// Example system showing velocity validation usage
fn example_vehicle_physics_with_utilities(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut vehicle_query: Query<&mut Velocity, With<ExampleVehicle>>,
) {
    let dt = time.delta_secs();
    
    for mut velocity in vehicle_query.iter_mut() {
        // Apply common physics patterns using utilities
        
        // 1. Validate velocity for safety
        PhysicsUtilities::validate_velocity(&mut velocity, &config);
        
        // 2. Apply natural deceleration when no input
        PhysicsUtilities::apply_natural_deceleration(
            &mut velocity,
            config.gameplay.physics.linear_damping,
            config.gameplay.physics.angular_damping,
            dt
        );
        
        // 3. Apply drag forces
        let drag = PhysicsUtilities::calculate_drag_force(
            &velocity,
            0.3, // drag coefficient
            1.225, // air density
            2.5  // frontal area
        );
        
        // Apply drag to velocity
        velocity.linvel += drag * dt;
    }
}

// Example system showing velocity-based movement
fn example_player_movement_with_utilities(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut player_query: Query<(&mut Velocity, &Transform), With<ExamplePlayer>>,
) {
    let dt = time.delta_secs();
    
    for (mut velocity, transform) in player_query.iter_mut() {
        // Simulate input processing
        let input_force = Vec3::new(1.0, 0.0, 0.0) * 50.0;
        let input_torque = Vec3::new(0.0, 1.0, 0.0) * 10.0;
        
        // Use utilities for safe velocity-based movement
        PhysicsUtilities::apply_force_safe(
            &mut velocity,
            input_force,
            input_torque,
            dt,
            1000.0 // max force
        );
        
        // Note: Ground collision would need velocity access in real implementation
        // This is just demonstrating the API pattern
    }
}

// Example showing how to set up physics bodies consistently
fn spawn_example_entities(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    // Vehicle with consistent collision groups
    let vehicle_collision_groups = CollisionGroupHelper::vehicle_groups();
    let (vehicle_body, vehicle_groups, vehicle_damping) = PhysicsBodySetup::create_dynamic_body(
        vehicle_collision_groups,
        config.gameplay.physics.linear_damping,
        config.gameplay.physics.angular_damping
    );
    
    commands.spawn((
        vehicle_body,
        vehicle_groups,
        vehicle_damping,
        ExampleVehicle,
        // Add other components...
    ));
    
    // Character with consistent collision groups
    let character_collision_groups = CollisionGroupHelper::character_groups();
    let (character_body, character_groups, character_damping) = PhysicsBodySetup::create_dynamic_body(
        character_collision_groups,
        config.gameplay.physics.linear_damping * 2.0, // Higher damping for characters
        config.gameplay.physics.angular_damping * 2.0
    );
    
    commands.spawn((
        character_body,
        character_groups, 
        character_damping,
        ExamplePlayer,
        // Add other components...
    ));
}

// Example showing input processing utilities
fn example_input_processing() {
    let mut current_throttle = 0.5;
    let target_throttle = 1.0;
    let dt = 0.016; // 60 FPS
    
    // Smooth input ramping
    current_throttle = InputProcessor::process_acceleration_input(
        current_throttle,
        target_throttle,
        3.0, // ramp up rate
        2.0, // ramp down rate
        dt
    );
    
    // Speed-dependent steering
    let steering_input = 1.0;
    let current_speed = 30.0;
    let adjusted_steering = InputProcessor::apply_speed_dependent_steering(
        steering_input,
        current_speed,
        1.0, // base sensitivity
        60.0 // speed threshold
    );
    
    // Force calculation with power curve
    let force = InputProcessor::calculate_force_from_input(
        current_throttle,
        1000.0, // base force
        1.5     // power curve
    );
    
    println!("Processed input - Throttle: {}, Steering: {}, Force: {}", 
             current_throttle, adjusted_steering, force);
}

// Example components for demonstration
#[derive(Component)]
struct ExampleVehicle;

#[derive(Component)]
struct ExamplePlayer;

// Example showing the universal safety system usage
fn setup_universal_safety_system(app: &mut App) {
    app.add_systems(Update, (
        gta_game::systems::apply_universal_physics_safeguards,
        example_vehicle_physics_with_utilities,
        example_player_movement_with_utilities,
    ));
}

fn main() {
    // This is just an example - not meant to run as a standalone binary
    println!("Physics utilities integration example");
    example_input_processing();
}
