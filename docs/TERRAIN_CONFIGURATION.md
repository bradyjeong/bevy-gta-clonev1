# Terrain Configuration

Phase 2 of the terrain system introduces asset-driven configuration using RON files. This document explains how to configure terrain parameters without modifying code.

## Configuration File

The terrain configuration is loaded from `assets/config/terrain.ron`.

## Configuration Structure

### Basic Terrain Settings

```ron
TerrainConfig(
    world_size: 4096.0,        // Total world size in units (creates a 4096x4096 world)
    resolution: 512,           // Heightmap resolution (512x512 grid points)
    base_height: -0.15,        // Base terrain height (matches existing ground detection)
    hill_scale: 15.0,          // Maximum height variation for hills/mountains
    noise_seed: 12345,         // Seed for reproducible terrain generation
    chunk_size: 64.0,          // Size of terrain chunks for LOD and streaming
    // ... additional settings
)
```

### Water Areas

Define water bodies like lakes, ponds, and rivers:

```ron
water_areas: [
    WaterArea(
        center: (100.0, 100.0),    // World position of water center
        radius: 50.0,              // Radius of circular water area
        depth: -5.0,               // Water surface depth (negative values)
        description: "Central lake" // Human-readable description
    ),
    // Add more water areas as needed
],
```

### Performance Settings

Control rendering performance and LOD:

```ron
performance: PerformanceSettings(
    max_terrain_distance: 2000.0,    // Maximum distance for terrain rendering
    lod_levels: 4,                   // Number of Level-of-Detail levels
    enable_frustum_culling: true,    // Cull terrain outside camera view
    chunk_cache_size: 64,            // Number of chunks to keep in memory
),
```

### Generation Settings

Control procedural terrain generation:

```ron
generation: GenerationSettings(
    noise_octaves: 4,          // Number of noise layers (more = more detail)
    noise_frequency: 0.01,     // Base noise frequency (lower = larger features)
    noise_persistence: 0.5,    // How much each octave contributes
    noise_lacunarity: 2.0,     // Frequency multiplier between octaves
    terrain_smoothing: true,   // Apply smoothing filter
    edge_falloff: 100.0,       // Distance over which terrain fades at world edge
),
```

## Parameter Limits

The configuration includes validation to prevent invalid values:

- `world_size`: Must be positive
- `resolution`: Between 32 and 2048 (powers of 2 recommended)
- `chunk_size`: Must be positive
- `hill_scale`: Cannot be negative
- `max_terrain_distance`: Must be positive
- `lod_levels`: At least 1
- `noise_octaves`: At least 1
- `noise_frequency`: Must be positive

## Usage Examples

### Desert Terrain
```ron
TerrainConfig(
    world_size: 2048.0,
    resolution: 256,
    base_height: 0.0,
    hill_scale: 30.0,
    noise_seed: 11111,
    water_areas: [
        WaterArea(
            center: (400.0, -150.0),
            radius: 80.0,
            depth: -8.0,
            description: "Desert oasis"
        ),
    ],
    generation: GenerationSettings(
        noise_octaves: 3,
        noise_frequency: 0.005,
        noise_persistence: 0.6,
        terrain_smoothing: false,
        // ... other settings
    ),
    // ... other settings
)
```

### Mountainous Terrain
```ron
TerrainConfig(
    world_size: 8192.0,
    resolution: 1024,
    base_height: 10.0,
    hill_scale: 100.0,
    noise_seed: 22222,
    generation: GenerationSettings(
        noise_octaves: 6,
        noise_frequency: 0.01,
        noise_persistence: 0.4,
        noise_lacunarity: 2.5,
        terrain_smoothing: true,
        // ... other settings
    ),
    // ... other settings
)
```

## Hot Reloading

During development, you can modify the `terrain.ron` file and the changes will be automatically loaded without restarting the game. Watch the console for loading messages:

```
INFO terrain: Terrain configuration loaded and validated successfully!
INFO terrain: TerrainService: Updated with asset configuration
```

## Debug Controls

Press **F4** during gameplay to display the current terrain configuration in the console:

```
=== LOADED TERRAIN CONFIGURATION ===
World size: 4096x4096
Resolution: 512x512
Base height: -0.15
Hill scale: 15
Water areas: 3
```

## Integration with TerrainService

The `TerrainService` automatically uses the loaded configuration for:

- Height queries (`height_at()`)
- Water detection (`is_water_at()`)
- Chunk calculations (`world_to_chunk()`)
- Spawn position validation

Example usage in code:
```rust
fn system(terrain_service: Res<TerrainService>) {
    let height = terrain_service.height_at(100.0, 200.0);
    let is_water = terrain_service.is_water_at(100.0, 200.0);
    let world_size = terrain_service.get_world_size();
}
```

## Benefits of Asset-Driven Configuration

1. **No Code Changes**: Terrain parameters can be tweaked without recompilation
2. **Runtime Customization**: Players can modify terrain settings
3. **Designer Friendly**: Non-programmers can adjust terrain parameters
4. **Version Control**: Configuration changes are tracked separately from code
5. **Hot Reload**: Immediate feedback during development
6. **Validation**: Automatic validation prevents invalid configurations

## Future Phases

Phase 2 provides the foundation for:
- Phase 3: Procedural heightmap generation using noise
- Phase 4: Advanced features like biomes, roads, and structures
- Phase 5: Dynamic terrain modification and streaming

The asset-driven configuration will continue to support all future features without requiring code changes.
