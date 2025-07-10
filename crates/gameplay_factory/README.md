# Gameplay Factory

A unified entity factory for prefab-based gameplay systems in the AMP game engine.

## Overview

The `gameplay_factory` crate provides a factory pattern for creating game entities from prefab definitions. It supports loading prefabs from various sources and spawning them into the ECS world using Bevy's entity-component system.

## Features

- **Prefab System**: Define reusable entity templates with component initialization
- **Factory Pattern**: Centralized entity creation with prefab registry
- **Multiple Sources**: Load prefabs from code, RON files, or custom sources
- **RON Support**: Built-in RON (Rusty Object Notation) loader with feature flag
- **Type Safety**: Strongly typed prefab IDs and error handling
- **Extensible**: Trait-based design for custom component initializers

## Usage

### Basic Factory Usage

```rust
use gameplay_factory::*;

// Create a factory
let mut factory = Factory::new();

// Create a prefab with components
let player_prefab = Prefab::new()
    .with_component(Box::new(Position { x: 0.0, y: 0.0, z: 0.0 }))
    .with_component(Box::new(Health { current: 100, max: 100 }));

// Register the prefab
factory.register(PrefabId::from(1), player_prefab);

// Spawn entities from the prefab
let world = bevy_ecs::world::World::new();
let mut queue = bevy_ecs::system::CommandQueue::default();
let mut commands = bevy_ecs::system::Commands::new(&mut queue, &world);

factory.spawn(&mut commands, PrefabId::from(1))?;
```

### RON Loader (Optional)

```rust
use gameplay_factory::*;

// RON prefab definition
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

// Load from RON
let ron_loader = RonLoader::new(ron_content.to_string());
factory.load_from_source(PrefabId::from(2), &ron_loader)?;
```

### Custom Component Initializers

```rust
use gameplay_factory::*;
use std::any::Any;

#[derive(Debug, Clone)]
struct MyComponent {
    value: i32,
}

impl ComponentInit for MyComponent {
    fn init(&self, cmd: &mut bevy_ecs::system::Commands) -> Result<(), Error> {
        // Initialize component logic here
        println!("Initializing MyComponent with value: {}", self.value);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

## API Reference

### Core Types

- **`PrefabId`**: Unique identifier for prefab definitions
- **`Prefab`**: Container for component initializers
- **`Factory`**: Registry and spawning system for prefabs
- **`ComponentInit`**: Trait for component initialization logic

### Traits

- **`PrefabSource`**: Interface for loading prefabs from various sources
- **`ComponentInit`**: Interface for component initialization

### Error Handling

The crate uses `amp_core::Error` for consistent error handling across the engine. All operations return `Result<T, Error>` for proper error propagation.

## Features

- **`default`**: Enables RON support
- **`ron`**: Enables RON (Rusty Object Notation) loader functionality

## Examples

See `examples/gameplay_factory_example.rs` for a complete working example.

## Testing

The crate includes comprehensive tests with 100% code coverage:

```bash
cargo test --package gameplay_factory
cargo llvm-cov --package gameplay_factory --all-features
```

## Dependencies

- `bevy_ecs`: Entity-component system integration
- `amp_core`: Error handling and utilities
- `serde`: Serialization support
- `ron`: RON format support (optional)

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
