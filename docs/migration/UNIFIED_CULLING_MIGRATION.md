# Unified Distance/Culling System Migration Guide

## Overview
This migration replaces 7 separate LOD/culling systems with a single, configurable, high-performance system that handles all entity types through the `UnifiedCullable` component.

## Systems Being Replaced

### Current Systems (TO BE REMOVED):
1. `src/systems/world/culling.rs` - Basic distance culling
2. `src/systems/world/unified_lod.rs` - Unified LOD and culling
3. `src/systems/world/optimized_lod.rs` - Optimized LOD with dirty flags
4. `src/systems/vehicles/lod_manager.rs` - Vehicle-specific LOD
5. `src/systems/world/npc_lod.rs` - NPC-specific LOD
6. `src/systems/world/vegetation_lod.rs` - Vegetation-specific LOD
7. `src/systems/world/map_system.rs` - Map chunk LOD (partially)

### New Unified System:
- `src/systems/world/unified_distance_culling.rs` - Single system handling all entity types

## Migration Steps

### Step 1: Update Plugin Registration

**Remove old systems from plugins:**
```rust
// REMOVE these from your plugin configurations:
.add_systems(Update, distance_culling_system)
.add_systems(Update, vehicle_lod_system)
.add_systems(Update, npc_lod_system)
.add_systems(Update, vegetation_lod_system)
.add_systems(Update, unified_lod_system)
.add_systems(Update, optimized_unified_lod_system)
```

**Add the new unified plugin:**
```rust
use crate::systems::world::unified_distance_culling::UnifiedDistanceCullingPlugin;

app.add_plugins(UnifiedDistanceCullingPlugin);
```

### Step 2: Component Migration

**For Vehicles:**
```rust
// OLD:
commands.entity(entity).insert(Cullable { max_distance: 400.0, is_culled: false });

// NEW:
commands.entity(entity).insert(UnifiedCullable::vehicle());
```

**For NPCs:**
```rust
// OLD:
commands.entity(entity).insert(Cullable { max_distance: 150.0, is_culled: false });

// NEW:
commands.entity(entity).insert(UnifiedCullable::npc());
```

**For Vegetation:**
```rust
// OLD:
commands.entity(entity).insert(Cullable { max_distance: 300.0, is_culled: false });

// NEW:
commands.entity(entity).insert(UnifiedCullable::vegetation());
```

**For Buildings:**
```rust
// OLD:
commands.entity(entity).insert(Cullable { max_distance: 800.0, is_culled: false });

// NEW:
commands.entity(entity).insert(UnifiedCullable::building());
```

**For Map Chunks:**
```rust
// NEW (no direct replacement):
commands.entity(entity).insert(UnifiedCullable::chunk());
```

### Step 3: Custom Configuration

You can create custom configurations for specific entity types:

```rust
let custom_config = DistanceCullingConfig {
    lod_distances: vec![30.0, 100.0, 200.0],
    cull_distance: 300.0,
    hysteresis: 5.0,
    update_interval: 0.2,
    entity_type: "CustomEntity",
};

commands.entity(entity).insert(UnifiedCullable::new(custom_config));
```

### Step 4: Update Entity-Specific Rendering Systems

**Vehicle Rendering System Updates:**
```rust
// Listen for VehicleLODUpdate components instead of checking LOD directly
pub fn vehicle_rendering_system(
    mut commands: Commands,
    vehicle_query: Query<(Entity, &VehicleState, &VehicleLODUpdate)>,
    // ... other parameters
) {
    for (entity, vehicle_state, lod_update) in vehicle_query.iter() {
        // Update vehicle rendering based on lod_update.new_lod
        update_vehicle_lod(entity, vehicle_state, lod_update.new_lod, &mut commands);
        
        // Remove the update component after processing
        commands.entity(entity).remove::<VehicleLODUpdate>();
    }
}
```

**NPC Rendering System Updates:**
```rust
pub fn npc_rendering_system(
    mut commands: Commands,
    npc_query: Query<(Entity, &NPCState, &NPCLODUpdate)>,
    // ... other parameters
) {
    for (entity, npc_state, lod_update) in npc_query.iter() {
        // Update NPC rendering based on lod_update.new_lod
        update_npc_lod(entity, npc_state, lod_update.new_lod, &mut commands);
        
        // Remove the update component after processing
        commands.entity(entity).remove::<NPCLODUpdate>();
    }
}
```

**Vegetation Rendering System Updates:**
```rust
pub fn vegetation_rendering_system(
    mut commands: Commands,
    vegetation_query: Query<(Entity, &mut VegetationLOD, &VegetationLODUpdate)>,
    // ... other parameters
) {
    for (entity, mut vegetation_lod, lod_update) in vegetation_query.iter() {
        // Update vegetation LOD
        vegetation_lod.detail_level = lod_update.new_detail_level;
        vegetation_lod.distance_to_player = lod_update.distance;
        
        // Handle mesh switching, billboard updates, etc.
        update_vegetation_rendering(entity, &vegetation_lod, &mut commands);
        
        // Remove the update component after processing
        commands.entity(entity).remove::<VegetationLODUpdate>();
    }
}
```

### Step 5: Chunk System Integration

**Map System Updates:**
```rust
pub fn chunk_lod_handling_system(
    mut commands: Commands,
    chunk_query: Query<(Entity, &mut MapChunk, &ChunkLODUpdate)>,
    unload_query: Query<Entity, With<ChunkUnloadRequest>>,
) {
    // Handle LOD updates
    for (entity, mut chunk, lod_update) in chunk_query.iter() {
        chunk.lod_level = lod_update.new_lod;
        chunk.distance_to_player = lod_update.distance;
        
        // Update chunk content based on new LOD
        update_chunk_content(entity, &chunk, &mut commands);
        
        commands.entity(entity).remove::<ChunkLODUpdate>();
    }
    
    // Handle unload requests
    for entity in unload_query.iter() {
        // Unload chunk content
        unload_chunk(entity, &mut commands);
        commands.entity(entity).remove::<ChunkUnloadRequest>();
    }
}
```

## Performance Benefits

### Before (7 separate systems):
- 7 different distance calculations per entity type
- Inconsistent update intervals
- Redundant visibility updates
- No distance caching coordination

### After (unified system):
- Single distance calculation using cache
- Consistent, configurable update intervals
- Coordinated visibility and LOD updates
- Batch processing for performance
- Entity-type specific optimizations

## Configuration Options

### Distance Thresholds
- **Vehicles**: 50m, 150m, 300m, cull at 500m
- **NPCs**: 25m, 75m, 100m, cull at 150m
- **Vegetation**: 50m, 150m, 300m, cull at 400m
- **Buildings**: 100m, 300m, 500m, cull at 800m
- **Chunks**: 150m, 300m, 500m, cull at 800m

### Update Intervals
- **Vehicles**: 0.5s
- **NPCs**: 0.3s  
- **Vegetation**: 1.0s
- **Buildings**: 0.8s
- **Chunks**: 0.5s

### Hysteresis Values
- Prevents LOD flickering at distance boundaries
- Configurable per entity type
- Default: 5m for most entities, 3m for NPCs, 10m for vegetation

## Testing and Validation

### Performance Testing
```bash
# Run with performance features enabled
cargo run --features debug-movement,debug-ui

# Monitor FPS and entity counts
# Press F3 for cache performance stats
```

### Verification Commands
```bash
# Check for compilation errors
cargo check

# Run tests
cargo test

# Build with optimizations
cargo build --release
```

### Debug Output
- Entity type distribution
- LOD level distribution  
- Culling statistics
- Cache hit rates
- Processing times

## Rollback Plan

If issues arise, you can temporarily restore old systems:

1. Comment out `UnifiedDistanceCullingPlugin` 
2. Re-enable old system registrations
3. Use migration system to convert `UnifiedCullable` back to `Cullable`

```rust
// Emergency rollback system
pub fn rollback_unified_to_cullable(
    query: Query<(Entity, &UnifiedCullable), Without<Cullable>>,
    mut commands: Commands,
) {
    for (entity, unified_cullable) in query.iter() {
        commands.entity(entity).insert(Cullable {
            max_distance: unified_cullable.config.cull_distance,
            is_culled: unified_cullable.is_culled,
        });
        commands.entity(entity).remove::<UnifiedCullable>();
    }
}
```

## Expected Results

- **Performance**: 60+ FPS maintained with improved frame consistency
- **Memory**: Reduced memory usage due to unified distance cache
- **Maintainability**: Single system to configure and debug
- **Scalability**: Easy to add new entity types with custom configurations
- **Consistency**: All entities use same distance calculation and caching logic
