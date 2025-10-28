//! Visual-Physics Separation Validation Tests
//!
//! These tests enforce the core architectural pattern: physics components (RigidBody, Collider, Velocity)
//! ONLY on parent entities, visual meshes ONLY on children with VisualOnly marker.
//!
//! Pattern documented in VISUAL_PHYSICS_SEPARATION.md

use bevy::asset::AssetPlugin;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use gta_game::components::water::YachtSpecs;
use gta_game::components::{
    Car, F16, Helicopter, MainRotor, SimpleCarSpecs, SimpleF16Specs, SimpleHelicopterSpecs,
    TailRotor, VisualOnly, WheelVisual, Yacht,
};
use gta_game::config::GameConfig;
use gta_game::factories::VehicleFactory;

/// Setup test app with minimal plugins + Rapier physics
fn setup_test_app_with_rapier() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        RapierPhysicsPlugin::<NoUserData>::default(),
    ));
    app.insert_resource(GameConfig::default());
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<SimpleCarSpecs>();
    app.init_asset::<SimpleHelicopterSpecs>();
    app.init_asset::<SimpleF16Specs>();
    app.init_asset::<YachtSpecs>();
    app
}

/// Spawn a SuperCar for testing
fn spawn_test_car(app: &mut App) -> Entity {
    let factory = VehicleFactory::new();

    let entity = app
        .world_mut()
        .run_system_once(
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>,
                  asset_server: Res<AssetServer>| {
                factory
                    .spawn_super_car(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        Vec3::new(0.0, 5.0, 0.0),
                        None,
                    )
                    .expect("Failed to spawn test car")
            },
        )
        .expect("Failed to run system");

    app.update();
    entity
}

/// Spawn a Helicopter for testing
fn spawn_test_helicopter(app: &mut App) -> Entity {
    let factory = VehicleFactory::new();

    let entity = app
        .world_mut()
        .run_system_once(
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>,
                  asset_server: Res<AssetServer>| {
                factory
                    .spawn_helicopter(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        Vec3::new(0.0, 10.0, 0.0),
                        None,
                    )
                    .expect("Failed to spawn test helicopter")
            },
        )
        .expect("Failed to run system");

    app.update();
    entity
}

/// Spawn an F16 for testing
fn spawn_test_f16(app: &mut App) -> Entity {
    let factory = VehicleFactory::new();

    let entity = app
        .world_mut()
        .run_system_once(
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>,
                  asset_server: Res<AssetServer>| {
                factory
                    .spawn_f16(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        Vec3::new(0.0, 50.0, 0.0),
                        None,
                    )
                    .expect("Failed to spawn test F16")
            },
        )
        .expect("Failed to run system");

    app.update();
    entity
}

/// Spawn a Yacht for testing
fn spawn_test_yacht(app: &mut App) -> Entity {
    let factory = VehicleFactory::new();

    let entity = app
        .world_mut()
        .run_system_once(
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>,
                  asset_server: Res<AssetServer>| {
                factory
                    .spawn_yacht(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        Vec3::new(0.0, 0.0, 0.0),
                        None,
                    )
                    .expect("Failed to spawn test yacht")
            },
        )
        .expect("Failed to run system");

    app.update();
    entity
}

/// Helper to recursively collect all descendants of an entity
fn collect_all_descendants(world: &World, entity: Entity, descendants: &mut Vec<Entity>) {
    if let Some(children) = world.get::<Children>(entity) {
        for child in children.iter() {
            descendants.push(child);
            collect_all_descendants(world, child, descendants);
        }
    }
}

// ============================================================================
// Test 1: Visual Children Validation
// ============================================================================

#[test]
fn test_visual_children_have_no_physics() {
    let mut app = setup_test_app_with_rapier();
    let car = spawn_test_car(&mut app);
    app.update();

    let world = app.world();
    let mut all_descendants = Vec::new();
    collect_all_descendants(world, car, &mut all_descendants);

    let mut visual_only_count = 0;
    for &descendant in &all_descendants {
        if world.get::<VisualOnly>(descendant).is_some() {
            visual_only_count += 1;

            assert!(
                world.get::<RigidBody>(descendant).is_none(),
                "Visual child {:?} ({}) should not have RigidBody",
                world.get::<Name>(descendant).map(|n| n.as_str()),
                descendant
            );
            assert!(
                world.get::<Collider>(descendant).is_none(),
                "Visual child {:?} ({}) should not have Collider",
                world.get::<Name>(descendant).map(|n| n.as_str()),
                descendant
            );
            assert!(
                world.get::<Velocity>(descendant).is_none(),
                "Visual child {:?} ({}) should not have Velocity",
                world.get::<Name>(descendant).map(|n| n.as_str()),
                descendant
            );
            assert!(
                world.get::<ExternalForce>(descendant).is_none(),
                "Visual child {:?} ({}) should not have ExternalForce",
                world.get::<Name>(descendant).map(|n| n.as_str()),
                descendant
            );
        }
    }

    assert!(
        visual_only_count > 0,
        "Car should have at least some VisualOnly children (found 0)"
    );
}

#[test]
fn test_wheels_are_visual_only() {
    let mut app = setup_test_app_with_rapier();
    let car = spawn_test_car(&mut app);
    app.update();

    let world = app.world();
    let mut wheel_count = 0;

    let mut all_descendants = Vec::new();
    collect_all_descendants(world, car, &mut all_descendants);

    for &descendant in &all_descendants {
        if world.get::<WheelVisual>(descendant).is_some() {
            wheel_count += 1;

            assert!(
                world.get::<VisualOnly>(descendant).is_some(),
                "WheelVisual entity {:?} ({}) must have VisualOnly marker",
                world.get::<Name>(descendant).map(|n| n.as_str()),
                descendant
            );

            assert!(
                world.get::<RigidBody>(descendant).is_none(),
                "WheelVisual {:?} should not have RigidBody",
                world.get::<Name>(descendant).map(|n| n.as_str())
            );
            assert!(
                world.get::<Collider>(descendant).is_none(),
                "WheelVisual {:?} should not have Collider",
                world.get::<Name>(descendant).map(|n| n.as_str())
            );
        }
    }

    assert!(
        wheel_count >= 4,
        "Car should have at least 4 wheels (found {})",
        wheel_count
    );
}

#[test]
fn test_rotors_are_visual_only() {
    let mut app = setup_test_app_with_rapier();
    let helicopter = spawn_test_helicopter(&mut app);
    app.update();

    let world = app.world();
    let mut main_rotor_count = 0;
    let mut _tail_rotor_count = 0;

    let mut all_descendants = Vec::new();
    collect_all_descendants(world, helicopter, &mut all_descendants);

    for &descendant in &all_descendants {
        if world.get::<MainRotor>(descendant).is_some() {
            main_rotor_count += 1;

            assert!(
                world.get::<VisualOnly>(descendant).is_some(),
                "MainRotor entity {:?} ({}) must have VisualOnly marker",
                world.get::<Name>(descendant).map(|n| n.as_str()),
                descendant
            );

            assert!(
                world.get::<RigidBody>(descendant).is_none(),
                "MainRotor should not have RigidBody"
            );
            assert!(
                world.get::<Collider>(descendant).is_none(),
                "MainRotor should not have Collider"
            );
        }

        if world.get::<TailRotor>(descendant).is_some() {
            _tail_rotor_count += 1;

            assert!(
                world.get::<VisualOnly>(descendant).is_some(),
                "TailRotor entity {:?} ({}) must have VisualOnly marker",
                world.get::<Name>(descendant).map(|n| n.as_str()),
                descendant
            );

            assert!(
                world.get::<RigidBody>(descendant).is_none(),
                "TailRotor should not have RigidBody"
            );
            assert!(
                world.get::<Collider>(descendant).is_none(),
                "TailRotor should not have Collider"
            );
        }
    }

    assert!(
        main_rotor_count >= 1,
        "Helicopter should have at least 1 main rotor (found {})",
        main_rotor_count
    );
}

// ============================================================================
// Test 2: Parent Entity Validation
// ============================================================================

#[test]
fn test_vehicle_parents_have_physics() {
    let mut app = setup_test_app_with_rapier();

    let car = spawn_test_car(&mut app);
    let helicopter = spawn_test_helicopter(&mut app);
    let f16 = spawn_test_f16(&mut app);
    app.update();

    let world = app.world();

    // Car parent must have RigidBody::Dynamic
    assert!(
        world.get::<Car>(car).is_some(),
        "Car entity should have Car component"
    );
    let car_rb = world
        .get::<RigidBody>(car)
        .expect("Car parent must have RigidBody");
    assert!(
        matches!(car_rb, RigidBody::Dynamic),
        "Car parent must have RigidBody::Dynamic"
    );
    assert!(
        world.get::<Collider>(car).is_some(),
        "Car parent must have Collider"
    );
    assert!(
        world.get::<Velocity>(car).is_some(),
        "Car parent must have Velocity"
    );

    // Helicopter parent must have RigidBody::Dynamic
    assert!(
        world.get::<Helicopter>(helicopter).is_some(),
        "Helicopter entity should have Helicopter component"
    );
    let heli_rb = world
        .get::<RigidBody>(helicopter)
        .expect("Helicopter parent must have RigidBody");
    assert!(
        matches!(heli_rb, RigidBody::Dynamic),
        "Helicopter parent must have RigidBody::Dynamic"
    );
    assert!(
        world.get::<Collider>(helicopter).is_some(),
        "Helicopter parent must have Collider"
    );
    assert!(
        world.get::<Velocity>(helicopter).is_some(),
        "Helicopter parent must have Velocity"
    );

    // F16 parent must have RigidBody::Dynamic
    assert!(
        world.get::<F16>(f16).is_some(),
        "F16 entity should have F16 component"
    );
    let f16_rb = world
        .get::<RigidBody>(f16)
        .expect("F16 parent must have RigidBody");
    assert!(
        matches!(f16_rb, RigidBody::Dynamic),
        "F16 parent must have RigidBody::Dynamic"
    );
    assert!(
        world.get::<Collider>(f16).is_some(),
        "F16 parent must have Collider"
    );
    assert!(
        world.get::<Velocity>(f16).is_some(),
        "F16 parent must have Velocity"
    );
}

#[test]
fn test_parents_have_single_collider() {
    let mut app = setup_test_app_with_rapier();
    let car = spawn_test_car(&mut app);
    app.update();

    let world = app.world();

    // Parent should have exactly 1 collider
    let parent_has_collider = world.get::<Collider>(car).is_some();
    assert!(
        parent_has_collider,
        "Vehicle parent must have exactly 1 Collider"
    );

    // Check that visual children don't have colliders (except special cases like yacht railings)
    let mut all_descendants = Vec::new();
    collect_all_descendants(world, car, &mut all_descendants);

    for &descendant in &all_descendants {
        if world.get::<VisualOnly>(descendant).is_some() {
            assert!(
                world.get::<Collider>(descendant).is_none(),
                "VisualOnly child {:?} should not have Collider",
                world.get::<Name>(descendant).map(|n| n.as_str())
            );
        }
    }
}

// ============================================================================
// Test 3: Hierarchy Validation
// ============================================================================

#[test]
fn test_vehicle_hierarchy_structure() {
    let mut app = setup_test_app_with_rapier();

    let car = spawn_test_car(&mut app);
    let helicopter = spawn_test_helicopter(&mut app);
    let f16 = spawn_test_f16(&mut app);
    app.update();

    let world = app.world();

    // Test car hierarchy
    let mut car_descendants = Vec::new();
    collect_all_descendants(world, car, &mut car_descendants);
    assert!(
        !car_descendants.is_empty(),
        "Car should have visual children"
    );

    let car_has_physics = world.get::<RigidBody>(car).is_some()
        && world.get::<Collider>(car).is_some()
        && world.get::<Velocity>(car).is_some();
    assert!(car_has_physics, "Car parent must have physics components");

    // Test helicopter hierarchy
    let mut heli_descendants = Vec::new();
    collect_all_descendants(world, helicopter, &mut heli_descendants);
    assert!(
        !heli_descendants.is_empty(),
        "Helicopter should have visual children"
    );

    let heli_has_physics = world.get::<RigidBody>(helicopter).is_some()
        && world.get::<Collider>(helicopter).is_some()
        && world.get::<Velocity>(helicopter).is_some();
    assert!(
        heli_has_physics,
        "Helicopter parent must have physics components"
    );

    // Test F16 hierarchy
    let mut f16_descendants = Vec::new();
    collect_all_descendants(world, f16, &mut f16_descendants);
    assert!(
        !f16_descendants.is_empty(),
        "F16 should have visual children"
    );

    let f16_has_physics = world.get::<RigidBody>(f16).is_some()
        && world.get::<Collider>(f16).is_some()
        && world.get::<Velocity>(f16).is_some();
    assert!(f16_has_physics, "F16 parent must have physics components");
}

#[test]
fn test_no_physics_on_mesh_entities() {
    let mut app = setup_test_app_with_rapier();
    let car = spawn_test_car(&mut app);
    app.update();

    let world = app.world();
    let mut all_descendants = Vec::new();
    collect_all_descendants(world, car, &mut all_descendants);

    let mut mesh_with_visual_only = 0;
    let mut mesh_without_physics = 0;

    for &descendant in &all_descendants {
        if world.get::<Mesh3d>(descendant).is_some() {
            let is_parent = descendant == car;

            if !is_parent {
                // Non-parent mesh entities should NOT have physics components (critical)
                assert!(
                    world.get::<RigidBody>(descendant).is_none(),
                    "Mesh3d child {:?} ({}) should not have RigidBody",
                    world.get::<Name>(descendant).map(|n| n.as_str()),
                    descendant
                );
                assert!(
                    world.get::<Collider>(descendant).is_none(),
                    "Mesh3d child {:?} ({}) should not have Collider",
                    world.get::<Name>(descendant).map(|n| n.as_str()),
                    descendant
                );

                mesh_without_physics += 1;

                // Track how many have VisualOnly marker (good practice but not required)
                if world.get::<VisualOnly>(descendant).is_some() {
                    mesh_with_visual_only += 1;
                }
            }
        }
    }

    assert!(
        mesh_without_physics > 0,
        "Car should have mesh children without physics"
    );
}

// ============================================================================
// Test 4: Yacht Special Cases
// ============================================================================

#[test]
fn test_yacht_visual_separation() {
    let mut app = setup_test_app_with_rapier();
    let yacht = spawn_test_yacht(&mut app);
    app.update();

    let world = app.world();

    // Parent must have physics
    assert!(
        world.get::<Yacht>(yacht).is_some(),
        "Yacht entity should have Yacht component"
    );
    assert!(
        world.get::<RigidBody>(yacht).is_some(),
        "Yacht parent must have RigidBody"
    );
    assert!(
        world.get::<Collider>(yacht).is_some(),
        "Yacht parent must have Collider"
    );
    assert!(
        world.get::<Velocity>(yacht).is_some(),
        "Yacht parent must have Velocity"
    );

    // Check visual children
    let mut all_descendants = Vec::new();
    collect_all_descendants(world, yacht, &mut all_descendants);

    let mut visual_mesh_count = 0;
    for &descendant in &all_descendants {
        if world.get::<Mesh3d>(descendant).is_some() {
            visual_mesh_count += 1;

            // Yacht railings may have sensor colliders, but visual meshes should not have RigidBody
            if world.get::<RigidBody>(descendant).is_some() {
                panic!(
                    "Yacht visual mesh {:?} ({}) should not have RigidBody",
                    world.get::<Name>(descendant).map(|n| n.as_str()),
                    descendant
                );
            }
        }
    }

    assert!(
        visual_mesh_count > 0,
        "Yacht should have visual mesh children"
    );
}

// ============================================================================
// Test 5: Cross-Vehicle Consistency
// ============================================================================

#[test]
fn test_all_vehicles_follow_pattern() {
    let mut app = setup_test_app_with_rapier();

    let car = spawn_test_car(&mut app);
    let helicopter = spawn_test_helicopter(&mut app);
    let f16 = spawn_test_f16(&mut app);
    let yacht = spawn_test_yacht(&mut app);
    app.update();

    let world = app.world();
    let vehicles = vec![
        (car, "Car"),
        (helicopter, "Helicopter"),
        (f16, "F16"),
        (yacht, "Yacht"),
    ];

    for (vehicle, name) in vehicles {
        // All parents must have physics
        assert!(
            world.get::<RigidBody>(vehicle).is_some(),
            "{} parent must have RigidBody",
            name
        );
        assert!(
            world.get::<Collider>(vehicle).is_some(),
            "{} parent must have Collider",
            name
        );
        assert!(
            world.get::<Velocity>(vehicle).is_some(),
            "{} parent must have Velocity",
            name
        );

        // All visual children must NOT have physics
        let mut all_descendants = Vec::new();
        collect_all_descendants(world, vehicle, &mut all_descendants);

        for &descendant in &all_descendants {
            if world.get::<VisualOnly>(descendant).is_some() {
                assert!(
                    world.get::<RigidBody>(descendant).is_none(),
                    "{} visual child {:?} should not have RigidBody",
                    name,
                    world.get::<Name>(descendant).map(|n| n.as_str())
                );
                assert!(
                    world.get::<Collider>(descendant).is_none(),
                    "{} visual child {:?} should not have Collider",
                    name,
                    world.get::<Name>(descendant).map(|n| n.as_str())
                );
            }
        }
    }
}

// ============================================================================
// Test 6: VisibleChildBundle Validation
// ============================================================================

#[test]
fn test_visible_child_bundle_usage() {
    let mut app = setup_test_app_with_rapier();
    let car = spawn_test_car(&mut app);
    app.update();

    let world = app.world();
    let mut all_descendants = Vec::new();
    collect_all_descendants(world, car, &mut all_descendants);

    let mut visible_child_count = 0;
    for &descendant in &all_descendants {
        // VisibleChildBundle includes VisualOnly, so check for it
        if world.get::<VisualOnly>(descendant).is_some() {
            visible_child_count += 1;

            // Should have visibility components
            assert!(
                world.get::<Visibility>(descendant).is_some()
                    || world.get::<InheritedVisibility>(descendant).is_some(),
                "VisualOnly child {:?} should have visibility components from VisibleChildBundle",
                world.get::<Name>(descendant).map(|n| n.as_str())
            );
        }
    }

    assert!(
        visible_child_count > 0,
        "Car should have VisibleChildBundle children"
    );
}
