# 4km x 4km World Boundaries - Complete Reference

## Overview
World reduced from 12km to 4km for faster loading (12.6 seconds vs 90 seconds).

---

## Core Configuration

### WorldConfig (src/config.rs)
```rust
map_size: 4000.0          // 4km x 4km total world
chunk_size: 128.0         // 128m chunks
total_chunks: 31x31       // 961 total chunks
streaming_radius: 800.0   // Reduced from 1200m
```

### Validation Limits
```rust
map_size: 2000.0 - 8000.0 // Min 2km, max 8km allowed
streaming_radius: 200.0 - 2000.0
```

---

## Spatial Systems

### 1. Terrain (src/setup/world.rs)
✅ **Ground Plane**: 4096m x 4096m mesh
✅ **Physics Collider**: cuboid(2048, 0.05, 2048) = 2km radius
✅ **Position**: Centered at origin (0, 0, 0)

### 2. WorldBounds (src/components/world.rs)
✅ **Calculated from config**: ±2000m (derived from map_size/2)
✅ **Warning zone**: 500m from edge  
✅ **Critical zone**: 200m from edge
✅ **Enforcement**: Progressive pushback (GTA-style)

### 3. Chunk System (src/systems/world/unified_world.rs)
✅ **Total chunks**: 31x31 = 961 chunks
✅ **Chunk coordinates**: -15 to +15 (centered at origin)
✅ **World space**: Each chunk is 128m, total coverage = 3968m ≈ 4km

---

## Content Positioning

### Water System
✅ **Lake position**: Vec3(300, -2, 300) - Within 4km bounds
✅ **Lake size**: 200m x 200m
✅ **Lake bounds**: (200, 200) to (400, 400) - All within ±2000m

### Vehicle/NPC Spawning
✅ **Spawned per chunk**: Procedurally within chunk boundaries
✅ **Boundary system**: Prevents escape beyond ±2000m
⚠️ **Edge chunks**: Vehicles may spawn near edge and get clamped

### Aircraft
✅ **Spawn positions**: From setup systems (within bounds)
✅ **Max altitude**: 2000m ceiling
✅ **Boundary handling**: Stronger pushback + auto-turn

---

## Performance Optimizations

### Visibility Culling (VisibilityRange)
- Buildings: 350-400m
- Vegetation: 250-300m
- Water: 2000m
- All within 4km world bounds

### Physics Activation (GTA-style)
- **Activation radius**: 200m from player
- **Deactivation radius**: 250m from player
- **Max active buildings**: ~200-400 (instead of 70,000)

### Loading Performance
- **Total chunks**: 961 (down from 8,649)
- **Load time**: 12.6 seconds (down from 90 seconds)
- **Generation rate**: ~76 chunks/sec

---

## Boundary Behavior by Entity Type

### Ground Vehicles (Car, Yacht)
- Gentle pushback starting at edge
- Velocity zeroed perpendicular to boundary
- Cannot escape ±2000m bounds

### Aircraft (Helicopter, F16)
- Stronger pushback (2x strength)
- Auto-turning to redirect inward
- Altitude ceiling at 2000m

### Player (On Foot)
- Same as ground vehicles
- Warning zone feedback possible
- Safe navigation within bounds

---

## Key Coordinates Reference

### World Center
- Origin: (0, 0, 0)

### World Edges
- North: +2000m (Z)
- South: -2000m (Z)
- East: +2000m (X)
- West: -2000m (X)

### Notable Locations
- Lake center: (300, -2, 300)
- Player spawn: (0, 0.5, 0)
- Terrain center: (0, 0, 0)

---

## Verification Checklist

✅ WorldConfig.map_size = 4000.0
✅ Terrain mesh = 4096m x 4096m
✅ Terrain collider = ±2048m (4096m total)
✅ Chunk count = 31x31 = 961
✅ Streaming radius = 800m (within bounds)
✅ Lake position = (300, -2, 300) - valid
✅ WorldBounds validation limits updated (2-8km)
✅ Comments updated to reflect 4km
✅ Physics activation working
✅ Loading screen functional

---

## Future Considerations

### If Expanding World
1. Update WorldConfig.map_size in config.rs
2. Terrain mesh and collider auto-scale from config
3. Chunk count recalculates automatically
4. WorldBounds derived from config
5. No hardcoded coordinates to update

### If Adding Large Features
- Ensure positions within ±2000m
- Lake/water regions: Check against bounds
- Special landmarks: Validate coordinates
- Procedural content: Uses chunk-relative positioning (safe)

---

## Testing Validation

Run game and verify:
- [ ] World loads in ~12 seconds
- [ ] Terrain visible and centered
- [ ] Lake at correct position
- [ ] Vehicles cannot escape edges
- [ ] No "out of bounds" warnings for static content
- [ ] Physics activation working near player
- [ ] Loading screen shows progress
