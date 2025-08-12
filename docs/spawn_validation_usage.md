# Spawn Validation Plugin Usage Guide

## Overview

The `SpawnValidationPlugin` provides pure stateless functions for validating entity spawn positions. This replaces the scattered validation logic previously found across multiple files.

## Key Benefits

- **Pure Functions**: No side effects, deterministic behavior
- **Utility Module**: Can be called directly by other plugins (per AGENT.md §42)
- **Centralized Logic**: All validation rules in one place
- **Performance**: Optimized algorithms with minimal allocations

## Basic Usage

### 1. Import the Validation Functions

```rust
use crate::plugins::spawn_validation_plugin::{SpawnValidation, AdvancedSpawnValidation};
```

### 2. Position Validation

```rust
// Check if a position is valid for spawning a specific content type
let is_valid = SpawnValidation::is_spawn_position_valid(
    position,
    ContentType::Building,
    max_world_coord,
    Some(&road_network),
);
```

### 3. Collision Detection

```rust
// Check for collisions with existing entities
let has_collision = SpawnValidation::has_content_collision(
    position,
    ContentType::Vehicle,
    &existing_content, // &[(Vec3, ContentType, f32)]
);
```

### 4. Water Area Checking

```rust
// Check if position is in water
let in_water = SpawnValidation::is_in_water_area(position);
```

### 5. Road Proximity Checking

```rust
// Check if position is on/near roads
let on_road = SpawnValidation::is_on_road(
    position,
    &road_network,
    tolerance, // Distance in meters
);
```

## Advanced Usage

### Find Safe Spawn Position

```rust
// Find a safe position near preferred location
let safe_position = AdvancedSpawnValidation::find_safe_spawn_position(
    preferred_position,
    ContentType::Vehicle,
    max_world_coord,
    Some(&road_network),
    &existing_content,
    30.0, // max_search_radius
    20,   // max_attempts
);

match safe_position {
    Some(pos) => {
        // Spawn at safe position
    }
    None => {
        // No safe position found
    }
}
```

### Batch Position Validation

```rust
// Validate multiple positions at once
let valid_positions = AdvancedSpawnValidation::validate_spawn_positions(
    &candidate_positions,
    ContentType::Tree,
    max_world_coord,
    Some(&road_network),
    &existing_content,
);
```

### Entity-to-Entity Collision

```rust
// Check collision between two specific entities
let collision = AdvancedSpawnValidation::check_entity_collision(
    pos1, type1, radius1,
    pos2, type2, radius2,
);

if let Some(overlap_distance) = collision {
    // Entities would overlap by `overlap_distance` meters
}
```

## Migration from Old Validation Code

### Replace Old Factory Methods

**Before:**
```rust
// Old scattered validation
if unified_factory.has_content_collision(pos, content_type, existing) {
    // Handle collision
}

if position_validator.is_spawn_position_valid(pos, content_type, roads) {
    // Handle valid position
}
```

**After:**
```rust
// New centralized validation
if SpawnValidation::has_content_collision(pos, content_type, existing) {
    // Handle collision
}

if SpawnValidation::is_spawn_position_valid(pos, content_type, max_coord, Some(roads)) {
    // Handle valid position
}
```

### Replace Utility Functions

**Before:**
```rust
// Old spawn utils
if SpawnValidation::is_position_valid(pos, content_type, Some(roads)) {
    // Handle validation
}
```

**After:**
```rust
// New plugin functions
if SpawnValidation::is_spawn_position_valid(pos, content_type, max_coord, Some(roads)) {
    // Handle validation
}
```

## Performance Notes

- All functions are stateless and thread-safe
- Collision detection uses distance-squared optimization where possible
- Spatial queries are optimized for typical game scenarios
- No memory allocations in hot path functions

## Content Type Rules

### Road Constraints
- **Vehicles**: REQUIRE roads (negative tolerance)
- **Buildings/Trees**: AVOID roads (positive tolerance)
- **NPCs**: No road constraints

### Collision Distances
- **Buildings**: 35m minimum distance
- **Vehicles**: 25m minimum distance
- **Trees**: 10m minimum distance
- **NPCs**: 5m minimum distance

### Water Restrictions
- All content types avoid water except vehicles
- Lake center: (300, -2, 300) with 200m radius + 20m buffer

## Integration with Existing Systems

The plugin is designed to work alongside existing systems:

1. **Entity Factories**: Use validation before spawning
2. **World Generation**: Validate positions during content placement
3. **Dynamic Spawning**: Check positions before runtime spawning
4. **Save/Load**: Validate loaded positions for consistency

## Testing

The validation functions are pure and easily testable:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_validation() {
        let position = Vec3::new(100.0, 0.0, 100.0);
        let valid = SpawnValidation::is_spawn_position_valid(
            position,
            ContentType::Building,
            1000.0,
            None,
        );
        assert!(valid);
    }
}
```

This plugin follows AGENT.md principles by providing simple, focused utility functions that other plugins can use directly without complex dependencies.
