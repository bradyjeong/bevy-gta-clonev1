# Gameplay Factory Architecture

## Overview

The `gameplay_factory` crate provides a unified entity factory system for prefab-based gameplay systems. It implements the Factory pattern to create game entities from prefab definitions, supporting loading from various sources including RON (Rusty Object Notation) files.

## Architecture

### Core Components

```
gameplay_factory/
├─ Factory              # Main factory for prefab registry and spawning
├─ Prefab               # Prefab definition with component initializers
├─ PrefabSource         # Trait for loading prefabs from various sources
├─ ComponentInit        # Trait for component initialization
└─ RonLoader            # RON-based prefab loader (feature-gated)
```

### Factory Pattern Implementation

The factory follows a two-stage approach:

1. **Registration Phase**: Prefabs are loaded from sources and registered with unique IDs
2. **Spawning Phase**: Entities are spawned from registered prefabs into the ECS world

```rust
// Registration
let mut factory = Factory::new();
factory.register(PrefabId::from(1), prefab);

// Spawning
factory.spawn(&mut commands, PrefabId::from(1))?;
```

## Prefab Pipeline

### Data Flow

```
RON File → RonLoader → RonPrefab → Prefab → Factory → ECS Entity
```

1. **Source Loading**: RON files are loaded via `RonLoader::from_file()`
2. **Deserialization**: RON content is parsed into `RonPrefab` structure
3. **Prefab Creation**: `RonPrefab` is converted to `Prefab` with component initializers
4. **Registration**: Prefabs are registered in the factory with unique IDs
5. **Spawning**: Factory spawns entities by calling component initializers

### Component Initialization

Components implement the `ComponentInit` trait for custom initialization:

```rust
pub trait ComponentInit: Send + Sync {
    fn init(&self, cmd: &mut Commands) -> Result<(), Error>;
    fn as_any(&self) -> &dyn Any;
}
```

This allows for:
- Custom component setup logic
- Error handling during initialization
- Type-safe downcasting for introspection

## RON File Format

### Structure

```ron
RonPrefab(
    components: [
        RonComponent(
            component_type: "Transform",
            data: Map({
                "translation": Map({
                    "x": Number(0.0),
                    "y": Number(0.0),
                    "z": Number(0.0)
                }),
                "rotation": Map({
                    "x": Number(0.0),
                    "y": Number(0.0),
                    "z": Number(0.0),
                    "w": Number(1.0)
                }),
                "scale": Map({
                    "x": Number(1.0),
                    "y": Number(1.0),
                    "z": Number(1.0)
                })
            })
        ),
        RonComponent(
            component_type: "Health",
            data: Number(100.0)
        )
    ]
)
```

### Component Types

- **Transform**: Position, rotation, and scale data
- **Health**: Numeric health values
- **Mesh**: 3D model references
- **Material**: Texture and shader properties
- **Physics**: Collider and rigidbody settings

## Hot-Reload Path

### Development Workflow

1. **File Watching**: Development tools monitor RON files for changes
2. **Reload Trigger**: File system events trigger prefab reloading
3. **Factory Update**: Modified prefabs are re-registered with existing IDs
4. **Entity Refresh**: Existing entities can be updated with new prefab data

### Implementation Strategy

```rust
// Hot-reload implementation (conceptual)
impl Factory {
    pub fn reload_prefab(&mut self, id: PrefabId, source: &dyn PrefabSource) -> Result<(), Error> {
        let prefab = source.load()?;
        self.register(id, prefab); // Overwrites existing
        Ok(())
    }
}
```

### Performance Considerations

- **Incremental Loading**: Only changed prefabs are reloaded
- **Validation**: RON syntax errors are caught before registration
- **Fallback**: Invalid prefabs don't replace valid ones
- **Memory**: Old prefab data is cleaned up automatically

## Performance Constraints

### Memory Management

- **Prefab Registry**: HashMap with O(1) lookup by PrefabId
- **Component Storage**: Vec<Box<dyn ComponentInit>> for type erasure
- **Cloning**: Prefabs are not cloned during spawning, only component initializers

### Runtime Performance

- **Spawn Time**: O(n) where n is number of components in prefab
- **Registry Lookup**: O(1) average case for prefab retrieval
- **Component Initialization**: Depends on component complexity

### Scaling Considerations

```rust
// Performance targets
const MAX_PREFABS: usize = 10_000;          // Registry capacity
const MAX_COMPONENTS_PER_PREFAB: usize = 50; // Component limit
const TARGET_SPAWN_TIME_MS: f32 = 0.1;      // 100μs per spawn
```

## Error Handling

### Error Types

1. **Resource Loading**: File I/O errors, missing files
2. **Serialization**: RON parsing errors, invalid format
3. **Validation**: Missing prefabs, invalid component data
4. **Initialization**: Component setup failures

### Recovery Strategies

- **Graceful Degradation**: Failed components don't break entire prefab
- **Fallback Prefabs**: Default prefabs for essential entities
- **Error Logging**: Detailed error information for debugging
- **Retry Logic**: Transient errors are retried with backoff

## Integration Points

### ECS Integration

- **Commands**: Uses Bevy's `Commands` for entity spawning
- **World**: Integrates with Bevy's ECS world system
- **Components**: Works with any type implementing `ComponentInit`

### File System Integration

- **Asset Loading**: Integrates with Bevy's asset system
- **Path Resolution**: Supports relative and absolute paths
- **File Watching**: Compatible with hot-reload systems

### Configuration Integration

- **Feature Flags**: RON support is feature-gated
- **Build Configuration**: Supports different prefab sources per build
- **Runtime Configuration**: Factory can be configured at runtime

## Testing Strategy

### Unit Tests

- **Factory Operations**: Registration, spawning, error handling
- **Prefab Management**: Component addition, validation
- **RON Loading**: File parsing, deserialization
- **Component Initialization**: Success and failure cases

### Integration Tests

- **End-to-End**: RON file → Entity spawning
- **Error Recovery**: Invalid prefabs, missing files
- **Performance**: Spawning benchmarks, memory usage

### Coverage Requirements

- **Minimum Coverage**: 80% line coverage
- **Critical Paths**: 100% coverage for error handling
- **Edge Cases**: Empty prefabs, malformed RON files
