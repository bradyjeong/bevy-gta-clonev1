/// Common test utilities for Bevy game testing
/// Provides test app setup, helpers, and constants following AGENTS.md guidelines
pub mod test_entities;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy_rapier3d::prelude::*;
use std::time::Duration;

/// Test timeout duration (30 seconds)
pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Common test delta time (16.67ms for 60 FPS)
pub const TEST_DELTA_TIME: f32 = 1.0 / 60.0;

/// Default test position (above ground to avoid immediate collisions)
pub const TEST_SPAWN_POSITION: Vec3 = Vec3::new(0.0, 5.0, 0.0);

/// Velocity threshold for considering an entity stopped (m/s)
pub const VELOCITY_EPSILON: f32 = 0.01;

/// Position threshold for considering entities at same location (meters)
pub const POSITION_EPSILON: f32 = 0.1;

/// Setup minimal Bevy app for testing without physics
/// Use this for basic component/system tests that don't require physics simulation
pub fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TransformPlugin)
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            TEST_DELTA_TIME,
        )));
    app
}

/// Setup Bevy app with Rapier physics for vehicle/physics tests
/// Use this for tests that require collision detection, rigid bodies, or dynamics
pub fn setup_test_app_with_rapier() -> App {
    let mut app = setup_test_app();
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(bevy::scene::ScenePlugin);
    app
}

/// Run app updates for specified number of frames
/// Advances both fixed timestep (physics) and variable time
///
/// # Arguments
/// * `app` - Mutable reference to the Bevy App
/// * `frames` - Number of frames to simulate (at 60 FPS)
///
/// # Example
/// ```ignore
/// let mut app = setup_test_app_with_rapier();
/// run_app_updates(&mut app, 10); // Simulate 10 frames (~166ms)
/// ```
pub fn run_app_updates(app: &mut App, frames: u32) {
    for _ in 0..frames {
        app.update();
    }
}

/// Extract velocity from entity as Vec3
/// Returns zero vector if entity doesn't exist or has no Velocity component
///
/// # Arguments
/// * `app` - Reference to the Bevy App
/// * `entity` - Entity ID to query
pub fn get_velocity(app: &App, entity: Entity) -> Vec3 {
    app.world()
        .get::<Velocity>(entity)
        .map(|v| v.linvel)
        .unwrap_or(Vec3::ZERO)
}

/// Extract position from entity as Vec3
/// Returns zero vector if entity doesn't exist or has no Transform component
///
/// # Arguments
/// * `app` - Reference to the Bevy App
/// * `entity` - Entity ID to query
pub fn get_position(app: &App, entity: Entity) -> Vec3 {
    app.world()
        .get::<Transform>(entity)
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO)
}

/// Check if entity still exists in world
pub fn entity_exists(app: &App, entity: Entity) -> bool {
    app.world().get_entity(entity).is_ok()
}

/// Assert entity has expected velocity within epsilon
#[track_caller]
pub fn assert_velocity_near(app: &App, entity: Entity, expected: Vec3, epsilon: f32) {
    let actual = get_velocity(app, entity);
    let diff = (actual - expected).length();
    assert!(
        diff < epsilon,
        "Velocity mismatch: expected {:?}, got {:?} (diff: {:.6})",
        expected,
        actual,
        diff
    );
}

/// Assert entity has expected position within epsilon
#[track_caller]
pub fn assert_position_near(app: &App, entity: Entity, expected: Vec3, epsilon: f32) {
    let actual = get_position(app, entity);
    let diff = (actual - expected).length();
    assert!(
        diff < epsilon,
        "Position mismatch: expected {:?}, got {:?} (diff: {:.6})",
        expected,
        actual,
        diff
    );
}
