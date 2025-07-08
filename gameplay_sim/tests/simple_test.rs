#![cfg(feature = "heavy_tests")]
// Simple test to verify basic compilation
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;

#[test]
fn test_basic_compilation() {
    // Test that we can create basic Bevy app
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // Test that we can use game_core components
    let entity = app.world_mut().spawn(Transform::default()).id();
    assert!(app.world().get::<Transform>(entity).is_some());
}
#[test]
fn test_physics_compilation() {
    // Test that we can use physics components
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
    ));
    let entity = app.world_mut().spawn((
        Transform::default(),
        RigidBody::Dynamic,
        Velocity::default(),
    )).id();
    assert!(app.world().get::<RigidBody>(entity).is_some());
    assert!(app.world().get::<Velocity>(entity).is_some());
}
