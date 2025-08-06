# Vegetation GPU Instancing System

This document describes the GPU instancing system implemented for vegetation rendering in the Bevy GTA clone.

## Overview

The vegetation instancing system reduces rendering overhead by batching similar vegetation entities into instanced draws. Instead of rendering 1000+ individual vegetation entities, the system groups them by type and renders them in batches of up to 1024 instances per draw call.

## Architecture

### Components (`src/components/instanced_vegetation.rs`)

#### Core Components
- **`InstanceData`**: Contains transform, color, scale, and animation data for each instance
- **`InstancedPalmFrond`**: Batches palm frond instances (max 256 per batch)
- **`InstancedLeafCluster`**: Batches leaf cluster instances (max 512 per batch) 
- **`InstancedTreeTrunk`**: Batches tree trunk instances (max 128 per batch)
- **`InstancedBush`**: Batches bush instances (max 384 per batch)

#### Configuration
- **`VegetationInstancingConfig`**: Resource containing batch sizes, culling distances, and update intervals
- **`VegetationBatchable`**: Marker component identifying vegetation entities for instancing
- **`VegetationType`**: Enum defining vegetation categories for batching

### Systems (`src/systems/rendering/vegetation_instancing.rs`)

#### Core Systems
1. **`collect_vegetation_instances_system`**: Groups vegetation by type and creates instanced batches
2. **`update_vegetation_instancing_system`**: Updates dirty instanced entities with new meshes/materials
3. **`animate_vegetation_instances_system`**: Applies wind animation to vegetation instances
4. **`vegetation_instancing_metrics_system`**: Reports performance metrics

#### Integration Systems (`src/systems/vegetation_instancing_integration.rs`)
- **`integrate_vegetation_with_instancing_system`**: Converts existing vegetation to use instancing
- **`spawn_test_vegetation_system`**: Creates test vegetation for demonstration

### Dirty Flag Integration (`src/components/dirty_flags.rs`)

The system integrates with the existing dirty flag batching system:
- **`DirtyVegetationInstancing`**: Marks vegetation instances needing updates
- **`mark_vegetation_instancing_dirty_system`**: Automatically marks changed vegetation

## Performance Impact

### Before Instancing
- 1000 vegetation entities = 1000 draw calls
- High CPU overhead from individual entity processing
- GPU state changes for each entity

### After Instancing  
- 1000 vegetation entities = ~50 instanced draw calls
- Reduced CPU overhead through batching
- Efficient GPU instanced rendering

### Expected Performance Gains
- **Draw Calls**: Reduced by 95% (1000 → 50)
- **CPU Usage**: 60-70% reduction in vegetation rendering overhead
- **Memory**: More efficient through shared meshes and materials
- **Frame Rate**: 15-25% improvement in vegetation-heavy scenes

## Configuration

Default configuration in `VegetationInstancingConfig`:
```rust
VegetationInstancingConfig {
    palm_frond_batch_size: 256,
    leaf_cluster_batch_size: 512, 
    tree_trunk_batch_size: 128,
    bush_batch_size: 384,
    max_instances_per_draw: 1024,
    culling_distance: 500.0,        // Cull vegetation beyond 500 units
    update_interval: 0.5,           // Update every 0.5 seconds
}
```

## Usage

### Basic Setup
1. Add the `VegetationInstancingConfig` resource
2. Register the vegetation instancing systems
3. Mark vegetation entities with `VegetationBatchable` component

### Integration with Existing Vegetation
```rust
// Convert existing vegetation to use instancing
commands.entity(vegetation_entity).insert(VegetationBatchable {
    vegetation_type: VegetationType::PalmFrond,
    mesh_id: None,
    material_id: None,
});
```

### System Ordering
```rust
.add_systems(Update, (
    // 1. Mark dirty entities
    mark_vegetation_instancing_dirty_system,
    
    // 2. Collect and batch instances  
    collect_vegetation_instances_system,
    
    // 3. Update rendering
    update_vegetation_instancing_system,
    
    // 4. Animate instances
    animate_vegetation_instances_system,
))
```

## Animation Features

The system includes built-in wind animation:
- **Sway Animation**: Vegetation sways based on wind simulation
- **Variation**: Each instance has unique sway timing and amplitude
- **Performance**: Animation runs on instance data, not individual entities

## Culling Integration

Vegetation instancing integrates with the distance culling system:
- Vegetation beyond `culling_distance` is excluded from batches
- Only visible vegetation is instanced and rendered
- Automatic integration with `Cullable` component

## Monitoring

The system provides real-time metrics:
- Total instances per vegetation type
- Number of draw calls
- Performance statistics logged every 5 seconds

Example output:
```
Vegetation Instancing - Total Instances: 1247 | Draw Calls: 43 | PF:256 LC:512 TT:128 B:351
```

## Best Practices

1. **Batch Sizes**: Tune batch sizes based on your vegetation density
2. **Update Interval**: Increase interval for better performance in static scenes
3. **Culling Distance**: Adjust based on view distance requirements
4. **Vegetation Types**: Group similar vegetation into the same type for better batching

## Future Enhancements

- GPU-side frustum culling for instances
- Level-of-detail for individual instances
- Procedural generation integration
- Advanced wind simulation
- Shadow cascading for instanced vegetation
