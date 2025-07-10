use bevy_core::Name;
use bevy_ecs::prelude::*;
use bevy_ecs::system::{CommandQueue, Commands};
use bevy_ecs::world::World;
use bevy_render::view::Visibility;
use bevy_transform::components::Transform;
use gameplay_factory::*;
use serial_test::serial;

// Simple test components for testing
#[derive(Component, Debug, PartialEq)]
pub struct TestTransform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, PartialEq)]
pub struct TestName(pub String);

#[derive(Component, Debug, PartialEq)]
pub struct TestVisibility(pub bool);

#[test]
fn test_component_registry_initialization() {
    // Initialize the component registry
    register_default_components();
    clear_all_prefab_ids();

    // Verify registry has basic components
    let components = registered_components();
    assert!(components.contains(&"Transform"));
    assert!(components.contains(&"Name"));
    assert!(components.contains(&"Visibility"));
}

#[test]
#[serial]
fn test_real_component_spawning() {
    // Initialize the component registry
    register_default_components();
    clear_all_prefab_ids();
    clear_all_prefab_ids();

    let ron_content = r#"
    RonPrefab(
        components: [
            RonComponent(
                component_type: "Transform",
                data: Map({
                    "translation": Map({"x": 1.0, "y": 2.0, "z": 3.0}),
                    "rotation": Map({"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0}),
                    "scale": Map({"x": 1.0, "y": 1.0, "z": 1.0})
                })
            ),
            RonComponent(
                component_type: "Name",
                data: String("TestEntity")
            ),
            RonComponent(
                component_type: "Visibility",
                data: String("Visible")
            )
        ]
    )
    "#;

    let loader = RonLoader::new(ron_content.to_string());
    let prefab = loader.load().expect("Should load prefab");

    // Create factory and register prefab
    let mut factory = Factory::new();
    let prefab_id = PrefabId::from(1u64);
    factory.register(prefab_id, prefab).unwrap();

    // Create world and spawn entity
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut cmd = Commands::new(&mut queue, &world);

    let entity = factory
        .spawn(&mut cmd, prefab_id)
        .expect("Should spawn entity");

    // Apply commands
    queue.apply(&mut world);

    // Verify the entity exists in the world
    assert!(
        world.get_entity(entity).is_some(),
        "Entity should exist in world"
    );

    // Verify components were actually added
    let entity_ref = world.get_entity(entity).unwrap();
    assert!(
        entity_ref.contains::<Transform>(),
        "Entity should have Transform component"
    );
    assert!(
        entity_ref.contains::<Name>(),
        "Entity should have Name component"
    );
    assert!(
        entity_ref.contains::<Visibility>(),
        "Entity should have Visibility component"
    );

    // Verify component values
    let transform = entity_ref.get::<Transform>().unwrap();
    assert_eq!(transform.translation.x, 1.0);
    assert_eq!(transform.translation.y, 2.0);
    assert_eq!(transform.translation.z, 3.0);

    let name = entity_ref.get::<Name>().unwrap();
    assert_eq!(name.as_str(), "TestEntity");

    let visibility = entity_ref.get::<Visibility>().unwrap();
    assert_eq!(*visibility, Visibility::Visible);
}

#[test]
#[serial]
fn test_unknown_component_type_error() {
    // Initialize the component registry
    register_default_components();
    clear_all_prefab_ids();

    let ron_content = r#"
    RonPrefab(
        components: [
            RonComponent(
                component_type: "UnknownComponent",
                data: Number(42.0)
            )
        ]
    )
    "#;

    let loader = RonLoader::new(ron_content.to_string());
    let prefab = loader.load().expect("Should load prefab");

    // Create factory and register prefab
    let mut factory = Factory::new();
    let prefab_id = PrefabId::from(1u64);
    factory.register(prefab_id, prefab).unwrap();

    // Create world and try to spawn entity
    let world = World::new();
    let mut queue = CommandQueue::default();
    let mut cmd = Commands::new(&mut queue, &world);

    let result = factory.spawn(&mut cmd, prefab_id);

    // Should fail with validation error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("UnknownComponent"));
    assert!(error.to_string().contains("not found in registry"));
}

#[test]
#[serial]
fn test_custom_component_registration() {
    // Initialize the component registry
    register_default_components();
    clear_all_prefab_ids();

    // Define a custom component
    #[derive(Component, Debug, PartialEq)]
    struct Health(f32);

    // Register a custom component
    let _ = register_component(
        "Health",
        Box::new(|value, cmd, entity| {
            let health = match value {
                ron::Value::Number(n) => Health(n.into_f64() as f32),
                _ => Health(0.0),
            };
            cmd.entity(entity).insert(health);
            Ok(())
        }),
    );

    let ron_content = r#"
    RonPrefab(
        components: [
            RonComponent(
                component_type: "Health",
                data: Number(100.0)
            )
        ]
    )
    "#;

    let loader = RonLoader::new(ron_content.to_string());
    let prefab = loader.load().expect("Should load prefab");

    // Create factory and register prefab
    let mut factory = Factory::new();
    let prefab_id = PrefabId::from(1u64);
    factory.register(prefab_id, prefab).unwrap();

    // Create world and spawn entity
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut cmd = Commands::new(&mut queue, &world);

    let entity = factory
        .spawn(&mut cmd, prefab_id)
        .expect("Should spawn entity");

    // Apply commands
    queue.apply(&mut world);

    // Verify the entity exists and has the custom component
    let entity_ref = world.get_entity(entity).unwrap();
    assert!(
        entity_ref.contains::<Health>(),
        "Entity should have Health component"
    );

    let health = entity_ref.get::<Health>().unwrap();
    assert_eq!(health.0, 100.0);
}

#[test]
fn test_prefab_id_collision_detection() {
    clear_all_prefab_ids();
    let mut factory = Factory::new();

    // Register first prefab
    let prefab1 = Prefab::new();
    let id = PrefabId::from(42u64);
    factory.register(id, prefab1).unwrap();

    // Register second prefab with same ID (should fail now)
    let prefab2 = Prefab::new();
    assert!(factory.register(id, prefab2).is_err());

    // Should still contain the first prefab
    assert!(factory.contains(id));
    assert_eq!(factory.len(), 1);
}

#[test]
fn test_64bit_prefab_id() {
    clear_all_prefab_ids();
    let mut factory = Factory::new();

    // Test with large u64 value
    let large_id = PrefabId::from(u64::MAX);
    let prefab = Prefab::new();
    factory.register(large_id, prefab).unwrap();

    assert!(factory.contains(large_id));

    // Test backwards compatibility with u32
    let u32_id = PrefabId::try_from(42u32).unwrap();
    let prefab2 = Prefab::new();
    factory.register(u32_id, prefab2).unwrap();

    assert!(factory.contains(u32_id));
    assert_eq!(u32_id.raw(), 42u64);
}

#[cfg(feature = "ron")]
#[test]
fn test_path_based_id_generation_full_path() {
    use std::path::Path;

    let factory = Factory::new();

    // Test different paths generate different IDs
    let path1 = Path::new("/tmp/prefab1.ron");
    let path2 = Path::new("/tmp/prefab2.ron");
    let path3 = Path::new("/tmp/subdir/prefab1.ron"); // Same filename, different directory

    let id1 = factory.generate_prefab_id_from_path(path1).unwrap();
    let id2 = factory.generate_prefab_id_from_path(path2).unwrap();
    let id3 = factory.generate_prefab_id_from_path(path3).unwrap();

    // All IDs should be different
    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id2, id3);
}

#[cfg(all(feature = "ron", feature = "hot-reload"))]
#[test]
#[serial]
fn test_hot_reload_feature_enabled() {
    use config_core::FactorySettings;

    let mut factory = Factory::new();
    let settings = FactorySettings {
        prefab_path: "/tmp/test/*.ron".to_string(),
        hot_reload: true,
    };

    // Should not panic with hot-reload enabled
    let _ = factory.load_directory(&settings);
}

#[cfg(all(feature = "ron", not(feature = "hot-reload")))]
#[test]
fn test_hot_reload_feature_disabled() {
    use config_core::FactorySettings;

    let mut factory = Factory::new();
    let settings = FactorySettings {
        prefab_path: "/tmp/test/*.ron".to_string(),
        hot_reload: true,
    };

    // Should warn but not fail with hot-reload disabled
    let _ = factory.load_directory(&settings);
}
