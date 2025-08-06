# Entity Processing Batching with Dirty Flags

## Overview

This implementation adds optimized entity processing batching with dirty flags to reduce unnecessary processing for inactive entities and improve overall game performance.

## Key Components

### 1. Dirty Flag Components

- **`DirtyTransform`** - Marks entities whose position/rotation changed
- **`DirtyVisibility`** - Marks entities whose visibility state changed  
- **`DirtyPhysics`** - Marks entities with physics state changes
- **`DirtyLOD`** - Marks entities needing LOD recalculation

Each dirty flag includes:
- `marked_frame`: When the entity was marked dirty
- `priority`: Processing priority (Low, Normal, High, Critical)
- Additional context (e.g., `last_distance` for LOD)

### 2. Batching Systems

#### Core Marking Systems
- `mark_transform_dirty_system()` - Auto-marks when Transform changes
- `mark_visibility_dirty_system()` - Marks when visibility/cullable changes
- `mark_physics_dirty_system()` - Marks when physics components change
- `movement_based_lod_marking_system()` - Marks LOD changes based on movement

#### Batch Processing Systems
- `batch_transform_processing_system()` - Processes transform updates in batches
- `batch_physics_processing_system()` - Processes physics updates in batches
- `batch_lod_processing_system()` - Processes LOD calculations in batches
- `batch_culling_system()` - Processes visibility checks in batches

#### Optimized Replacement Systems
- `optimized_unified_lod_system()` - LOD system using dirty flags
- `optimized_distance_culling_system()` - Culling system using dirty flags
- `optimized_npc_lod_system()` - NPC LOD system with batching

### 3. Configuration

**NOTE: The batching subsystem and BatchingConfig have been removed.** The batch processing configuration is no longer available:

```rust
pub struct BatchingConfig {
    pub transform_batch_size: usize,        // 75 - Transform updates per frame
    pub visibility_batch_size: usize,       // 100 - Visibility checks per frame
    pub physics_batch_size: usize,          // 50 - Physics updates per frame
    pub lod_batch_size: usize,              // 80 - LOD calculations per frame
    pub max_processing_time_ms: f32,        // 8.0 - Max processing time per system
    pub priority_boost_frames: u64,         // 10 - Frames before priority boost
    pub cleanup_interval: f32,              // 5.0 - Dirty flag cleanup interval
    pub lod_distance_threshold: f32,        // 25.0 - Min distance change for LOD update
    pub transform_change_threshold: f32,    // 0.1 - Min transform change for marking dirty
    pub max_stale_frames: u64,              // 60 - Max frames before forced processing
    pub cleanup_stale_flags: bool,          // true - Clean up stale flags automatically
}
```

## System Architecture

### Processing Flow

1. **PreUpdate**: Mark dirty flags based on component changes
2. **Update**: Process dirty entities in configurable batches
3. **PostUpdate**: Clean up processed flags and monitor performance

### Priority System

Entities are processed based on priority:
- **Critical**: Physics changes, immediate threats
- **High**: Close entities, important visibility changes
- **Normal**: Standard updates
- **Low**: Distant entities, periodic updates

### Round-Robin Processing

The system uses round-robin scheduling to ensure all entities eventually get processed:
- Maintains offset counters for each batch type
- Processes different subset of entities each frame
- Prevents starvation of low-priority entities

## Performance Optimizations

### 1. LOD System Optimization

**Before**: All entities checked every frame
```rust
// Old system - processes all entities
for entity in all_entities.iter() {
    let distance = calculate_distance(entity, player);
    update_lod(entity, distance);
}
```

**After**: Only dirty entities processed
```rust
// New system - only processes dirty entities
for entity in dirty_lod_entities.iter().take(batch_size) {
    if entity.distance_changed_significantly() {
        update_lod(entity);
    }
}
```

### 2. Culling System Optimization

**Before**: Distance calculations every frame for all entities
**After**: Only recalculate when entity moves or visibility changes

### 3. Physics System Optimization

**Before**: All physics entities validated every frame
**After**: Only entities with physics changes are processed

## Integration

### Adding to Game

The batching system is integrated via the `BatchingPlugin`:

```rust
app.add_plugins(BatchingPlugin)
```

### Manual Entity Marking

Entities can be manually marked as dirty:

```rust
use crate::components::MarkDirty;

// Mark specific entity as needing transform update
commands.entity(entity_id).mark_transform_dirty(
    DirtyPriority::High, 
    current_frame
);

// Mark for all updates
commands.entity(entity_id).insert(DirtyFlagsBundle::default());
```

## Expected Performance Gains

### For 1000+ Entities

- **LOD System**: 60-80% reduction in processing time
  - Only processes entities that moved significantly
  - Reduces distance calculations from N to ~50-80 per frame

- **Culling System**: 70-85% reduction in processing time
  - Only checks visibility for entities with state changes
  - Eliminates redundant distance calculations

- **Physics System**: 40-60% reduction in processing time
  - Only validates entities with actual physics changes
  - Reduces constraint checking overhead

- **Overall Frame Time**: 20-40% improvement
  - Depends on entity count and activity level
  - More significant gains with higher entity counts

### Memory Usage

- **Dirty Flags**: ~16-32 bytes per entity (only when dirty)
- **Batch State**: ~100 bytes total
- **Metrics**: ~200 bytes
- **Total Overhead**: <1KB for most scenarios

## Testing and Validation

### Debug Controls

In debug builds, use these keys for testing:
- **F10**: Stress test (mark all entities dirty)
- **F11**: Clean up test entities

### Performance Monitoring

The system automatically reports performance metrics every 5 seconds:
```
Dirty Flags - Marked: T:45 V:23 P:12 L:67 | Processed: T:45 V:23 P:12 L:67 | Time: T:2.1ms V:1.8ms P:1.2ms L:3.1ms
```

### Automatic Testing

Test systems automatically:
- Spawn entities with various dirty flags
- Simulate real gameplay patterns
- Compare performance against baseline
- Report improvement percentages

## Migration from Old Systems

### Replacing Existing LOD Systems

1. Remove old LOD systems from schedule
2. Add `BatchingPlugin` 
3. Entities automatically get dirty flags when components change
4. Existing `Cullable` and `NPCState` components work seamlessly

### Backward Compatibility

- All existing components remain functional
- Old systems can run alongside new ones during transition
- No breaking changes to existing entity spawning code

## Best Practices

### When to Use

✅ **Good for**:
- High entity count scenarios (100+ entities)
- Entities that change infrequently
- Systems with expensive calculations (LOD, culling)
- Performance-critical applications

❌ **Avoid for**:
- Very low entity counts (<50 entities)
- Entities that change every frame
- Simple calculations that are already fast

### Configuration Tuning

- **High entity count**: Increase batch sizes, reduce processing time limit
- **Low-end hardware**: Decrease batch sizes, increase processing time spread
- **60fps target**: Keep `max_processing_time_ms` under 8ms per system
- **30fps target**: Can increase to 15ms per system

### Monitoring

Watch for these warning signs:
- Processing time consistently hitting limits
- Large numbers of stale dirty flags
- Entities not getting processed for many frames
- Frame time regression compared to baseline

## Future Enhancements

### Planned Improvements

1. **Spatial Partitioning**: Prioritize entities by distance to player
2. **Load Balancing**: Distribute processing across multiple frames more evenly
3. **System Dependencies**: Ensure physics processes before rendering
4. **Adaptive Batching**: Automatically adjust batch sizes based on performance

### Potential Extensions

- **Multi-threading**: Process different batch types in parallel
- **GPU Culling**: Move culling calculations to GPU for massive entity counts
- **Predictive Marking**: Mark entities as dirty based on predicted movement
- **Component Batching**: Batch similar component updates together
