//! Example demonstrating the gameplay_factory crate
//!
//! This example shows how to use the Factory pattern to create and spawn
//! entities from prefab definitions using RON configuration files.

use gameplay_factory::*;
use std::any::Any;

// Example component implementations
#[derive(Debug, Clone)]
struct Position {
    #[allow(dead_code)]
    x: f32,
    #[allow(dead_code)]
    y: f32,
    #[allow(dead_code)]
    z: f32,
}

impl ComponentInit for Position {
    fn init(
        &self,
        _cmd: &mut bevy_ecs::system::Commands,
        _entity: bevy_ecs::entity::Entity,
    ) -> Result<(), Error> {
        println!("Initializing Position component: {self:?}");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
struct Health {
    #[allow(dead_code)]
    current: i32,
    #[allow(dead_code)]
    max: i32,
}

impl ComponentInit for Health {
    fn init(
        &self,
        _cmd: &mut bevy_ecs::system::Commands,
        _entity: bevy_ecs::entity::Entity,
    ) -> Result<(), Error> {
        println!("Initializing Health component: {self:?}");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Gameplay Factory Example");
    println!("========================");

    // Create a factory
    let mut factory = Factory::new();

    // Create a prefab manually
    let player_prefab = Prefab::new()
        .with_component(Box::new(Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }))
        .with_component(Box::new(Health {
            current: 100,
            max: 100,
        }));

    // Register the prefab
    let _ = factory.register(PrefabId::from(1u32), player_prefab);

    // Create another prefab with different components
    let enemy_prefab = Prefab::new()
        .with_component(Box::new(Position {
            x: 10.0,
            y: 0.0,
            z: 5.0,
        }))
        .with_component(Box::new(Health {
            current: 50,
            max: 50,
        }));

    let _ = factory.register(PrefabId::from(2u32), enemy_prefab);

    // Example using RON loader
    #[cfg(feature = "ron")]
    {
        let ron_content = r#"
        RonPrefab(
            components: [
                RonComponent(
                    component_type: "Position",
                    data: Map({"x": Number(5.0), "y": Number(3.0), "z": Number(1.0)})
                ),
                RonComponent(
                    component_type: "Health",
                    data: Map({"current": Number(75), "max": Number(100)})
                )
            ]
        )
        "#;

        let ron_loader = RonLoader::new(ron_content.to_string());
        let result = factory.load_from_source(PrefabId::from(3u32), &ron_loader);

        match result {
            Ok(()) => println!("Successfully loaded prefab from RON"),
            Err(e) => println!("Failed to load prefab from RON: {e}"),
        }
    }

    // Simulate spawning entities
    println!("\nSpawning entities:");

    let world = bevy_ecs::world::World::new();
    let mut queue = bevy_ecs::system::CommandQueue::default();
    let mut commands = bevy_ecs::system::Commands::new(&mut queue, &world);

    // Spawn player
    match factory.spawn(&mut commands, PrefabId::from(1u32)) {
        Ok(entity) => println!("✓ Player spawned successfully as entity {entity:?}"),
        Err(e) => println!("✗ Failed to spawn player: {e}"),
    }

    // Spawn enemy
    match factory.spawn(&mut commands, PrefabId::from(2u32)) {
        Ok(entity) => println!("✓ Enemy spawned successfully as entity {entity:?}"),
        Err(e) => println!("✗ Failed to spawn enemy: {e}"),
    }

    // Try to spawn non-existent prefab
    match factory.spawn(&mut commands, PrefabId::from(999u32)) {
        Ok(entity) => println!("✓ Unknown entity spawned as {entity:?}"),
        Err(e) => println!("✗ Expected error for unknown prefab: {e}"),
    }

    // Display factory statistics
    println!("\nFactory Statistics:");
    println!("- Total prefabs registered: {}", factory.len());
    println!("- Factory is empty: {}", factory.is_empty());
    println!(
        "- Contains player prefab: {}",
        factory.contains(PrefabId::from(1u32))
    );
    println!(
        "- Contains enemy prefab: {}",
        factory.contains(PrefabId::from(2u32))
    );

    Ok(())
}
