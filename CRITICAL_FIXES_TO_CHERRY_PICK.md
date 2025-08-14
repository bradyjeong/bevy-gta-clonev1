# Critical Fixes to Cherry-Pick to Main Branch

## 1. RoadNetwork Implementation Fix ⚠️ CRITICAL
**Location**: `src/world/road_network.rs` (NEW FILE) + `src/systems/world/road_network.rs`
**Issue**: Main branch was using a broken 4-node demo RoadNetwork with u16 coordinates
**Fix**: Replace with proper HashMap-based implementation with:
- Proper coordinate types (f32 not u16)
- No 4-node limit
- Working `is_near_road()` and `get_nearest_road_point()` methods
- Support for negative coordinates

**How to apply**:
```bash
# Copy the fixed RoadNetwork to main
git checkout main
git checkout <this-branch> -- src/world/road_network.rs
git checkout <this-branch> -- src/systems/world/road_network.rs
```

## 2. Chunk Size Consistency ⚠️ IMPORTANT
**Location**: `src/constants.rs` + uses of `UNIFIED_CHUNK_SIZE`
**Issue**: Multiple conflicting chunk sizes (256.0 vs 100.0)
**Fix**: Unified to use `UNIFIED_CHUNK_SIZE` (100.0) everywhere

**Key changes**:
- `src/constants.rs`: Re-export UNIFIED_CHUNK_SIZE as CHUNK_SIZE
- All `ChunkCoord::from_world_pos()` calls use consistent size

## 3. Coordinate Overflow Fix ⚠️ CRITICAL
**Location**: `src/systems/world/layered_generation.rs`
**Issue**: Casting negative coordinates to u16 caused wraparound
**Fix**: Changed to use f32 coordinates:
```rust
// BEFORE (BROKEN):
let x = (chunk_center.x - half_size + i as f32 * node_spacing) as u16;
let z = (chunk_center.z - half_size + j as f32 * node_spacing) as u16;

// AFTER (FIXED):
let x = chunk_center.x - half_size + i as f32 * node_spacing;
let z = chunk_center.z - half_size + j as f32 * node_spacing;
```

## 4. Collision Detection Fix ⚠️ IMPORTANT
**Location**: `src/systems/world/dynamic_content.rs`
**Issue**: Roads with 15m radius blocked ALL entity spawning
**Fix**: Exclude roads from collision checks:
```rust
let mut existing_content: Vec<(Vec3, ContentType, f32)> = content_query.iter()
    .filter(|(_, _, dynamic_content)| dynamic_content.content_type != ContentType::Road)
    // ... rest of code
```

## 5. Spawn Rates Fix
**Location**: `src/systems/world/dynamic_content.rs`
**Issue**: Ultra-low spawn rates (~17% total chance)
**Fix**: Proper probability distribution:
```rust
let roll = rng.gen_range(0.0..1.0);
let content_type = if roll < 0.20 {
    Some(EventContentType::Building)  // 20%
} else if roll < 0.35 {
    Some(EventContentType::Vehicle)    // 15%
} else if roll < 0.55 {
    Some(EventContentType::Tree)       // 20%
} else if roll < 0.70 {
    Some(EventContentType::NPC)        // 15%
} else {
    None  // 30% chance to spawn nothing
};
```

## 6. Node Spacing Fix
**Location**: `src/systems/world/layered_generation.rs`
**Issue**: Only one road node per chunk (node_spacing == CHUNK_SIZE)
**Fix**: Proper grid spacing:
```rust
let node_spacing = 50.0; // Proper spacing for road network
let nodes_per_side = ((UNIFIED_CHUNK_SIZE / node_spacing) + 1.0) as u8;
```

## What NOT to Take

❌ Event-driven architecture (observers, validation events, etc.)
❌ Complex event routing
❌ Double-hop validation pipeline
❌ Observer patterns for spawning

## Recommended Approach

1. **First Priority**: Fix the RoadNetwork implementation (it's fundamentally broken in main)
2. **Second Priority**: Fix coordinate overflow and chunk size consistency
3. **Third Priority**: Fix collision detection and spawn rates
4. **Skip**: All the event-driven architectural changes

## Test After Cherry-Picking

```bash
cargo run
# Should see:
# - Roads generated: 600+
# - NPCs/Vehicles spawning
# - No coordinate overflow errors
# - Entities visible in world
```
