/// Infrastructure validation test
/// Verifies that test helpers and setup functions work correctly
mod common;

use bevy::prelude::*;
use common::*;

#[test]
fn test_app_setup_works() {
    let app = setup_test_app();
    assert!(app.world().entities().len() == 0, "App should start empty");
}

#[test]
fn test_app_with_rapier_works() {
    let app = setup_test_app_with_rapier();
    assert!(app.world().entities().len() == 0, "App should start empty");
}

#[test]
fn test_spawn_car_works() {
    let mut app = setup_test_app_with_rapier();
    let car = test_entities::spawn_test_car(&mut app, TEST_SPAWN_POSITION);

    assert!(entity_exists(&app, car), "Car entity should exist");
    assert_position_near(&app, car, TEST_SPAWN_POSITION, POSITION_EPSILON);
}

#[test]
fn test_spawn_helicopter_works() {
    let mut app = setup_test_app_with_rapier();
    let heli = test_entities::spawn_test_helicopter(&mut app, TEST_SPAWN_POSITION);

    assert!(entity_exists(&app, heli), "Helicopter entity should exist");
}

#[test]
fn test_spawn_f16_works() {
    let mut app = setup_test_app_with_rapier();
    let f16 = test_entities::spawn_test_f16(&mut app, TEST_SPAWN_POSITION);

    assert!(entity_exists(&app, f16), "F16 entity should exist");
}

#[test]
fn test_control_state_modification() {
    let mut app = setup_test_app_with_rapier();
    let car = test_entities::spawn_test_car(&mut app, TEST_SPAWN_POSITION);

    test_entities::set_test_control_state(&mut app, car, 1.0, 0.0, 0.5);

    let control = app
        .world()
        .get::<gta_game::components::ControlState>(car)
        .unwrap();
    assert_eq!(control.throttle, 1.0);
    assert_eq!(control.brake, 0.0);
    assert_eq!(control.steering, 0.5);
}

#[test]
fn test_velocity_helpers() {
    let mut app = setup_test_app_with_rapier();
    let car = test_entities::spawn_test_car(&mut app, TEST_SPAWN_POSITION);

    let vel = get_velocity(&app, car);
    assert_eq!(vel, Vec3::ZERO, "Initial velocity should be zero");

    test_entities::apply_test_velocity(&mut app, car, Vec3::new(10.0, 0.0, 0.0));
    let vel = get_velocity(&app, car);
    assert_eq!(vel.x, 10.0);
}

#[test]
fn test_run_updates() {
    let mut app = setup_test_app_with_rapier();
    let car = test_entities::spawn_test_car(&mut app, TEST_SPAWN_POSITION);

    run_app_updates(&mut app, 10);

    assert!(
        entity_exists(&app, car),
        "Car should still exist after updates"
    );
}

#[test]
fn test_ground_spawn() {
    let mut app = setup_test_app_with_rapier();
    let ground = test_entities::spawn_test_ground(&mut app, Vec3::ZERO, 100.0);

    assert!(entity_exists(&app, ground), "Ground entity should exist");
}
