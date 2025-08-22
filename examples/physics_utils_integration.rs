// Example integration showing how to use the new unified physics utilities
// This demonstrates the extracted common patterns from movement systems

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use gta_game::config::GameConfig;
use gta_game::systems::physics::physics_utils::{
    CollisionGroupHelper, PhysicsUtilities, apply_universal_physics_safeguards,
};

// Example system showing velocity validation usage
fn example_vehicle_physics_with_utilities(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut vehicle_query: Query<&mut Velocity, With<ExampleVehicle>>,
) {
    let dt = time.delta_secs();

    for mut velocity in vehicle_query.iter_mut() {
        // Apply common physics patterns using utilities

        // 1. Clamp velocity for safety and gameplay limits
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);

        // 2. Apply simple damping (Rapier handles complex physics)
        velocity.linvel *= 0.95; // Simple deceleration
        velocity.angvel *= 0.90;
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

        // Apply forces directly to velocity (Dynamic Arcade Physics)
        let dt = PhysicsUtilities::stable_dt(&time);
        velocity.linvel += input_force * dt;
        velocity.angvel += input_torque * dt;

        // Clamp for gameplay limits
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);

        // Note: Ground collision would need velocity access in real implementation
        // This is just demonstrating the API pattern
    }
}

// Example showing how to set up physics bodies consistently
fn spawn_example_entities(mut commands: Commands, config: Res<GameConfig>) {
    // Vehicle with consistent collision groups
    let vehicle_groups = CollisionGroupHelper::vehicle_groups();

    commands.spawn((
        RigidBody::Dynamic,
        vehicle_groups,
        Damping {
            linear_damping: config.physics.linear_damping,
            angular_damping: config.physics.angular_damping,
        },
        Velocity::default(),
        ExampleVehicle,
    ));

    // Character with consistent collision groups
    let character_groups = CollisionGroupHelper::character_groups();

    commands.spawn((
        RigidBody::Dynamic,
        character_groups,
        Damping {
            linear_damping: config.physics.linear_damping * 2.0,
            angular_damping: config.physics.angular_damping * 2.0,
        },
        Velocity::default(),
        ExamplePlayer,
    ));
}

// Example showing stable delta-time usage
fn example_stable_timing() {
    use bevy::time::Time;
    let time = Time::default();

    // Unified delta-time for all physics systems
    let dt = PhysicsUtilities::stable_dt(&time);

    println!("Stable delta-time: {:.6}s (clamped 0.001-0.05)", dt);
}

// Example components for demonstration
#[derive(Component)]
struct ExampleVehicle;

#[derive(Component)]
struct ExamplePlayer;

// Example showing the universal safety system usage
fn setup_universal_safety_system(app: &mut App) {
    app.add_systems(
        Update,
        (
            apply_universal_physics_safeguards,
            example_vehicle_physics_with_utilities,
            example_player_movement_with_utilities,
        ),
    );
}

fn main() {
    // This is just an example - not meant to run as a standalone binary
    println!("Physics utilities integration example");
    example_stable_timing();
}
