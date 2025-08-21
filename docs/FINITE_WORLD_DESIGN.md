# Finite World Design - Implementation Plan

## World Bounds
- **Size**: 4096m × 4096m (4km²)
- **Center**: (0, 0) to maintain existing spawn points
- **Bounds**: -2048m to +2048m on X and Z axes
- **Precision**: f32 precision excellent at these coordinates (~0.01mm accuracy)

## Edge Behavior Design

### Ground Vehicles (Cars)
- **Ocean Boundary**: Water extends 100m beyond world edge
- **Invisible Barriers**: Gentle physics pushback at -2000m/+2000m 
- **Visual**: Endless ocean horizon with distant islands (skybox)

### Aircraft (Helicopters, F16)
- **Altitude Limits**: Max height 2000m above ground
- **Boundary Behavior**: Gentle turn-back forces at edges
- **Visual**: Atmospheric effects (clouds, haze) obscure distant areas

### Watercraft (Yachts, Boats)  
- **Ocean Area**: 500m ocean border around landmass
- **Boundary**: Storm/weather effects create natural turn-back zones
- **Gameplay**: Treasure/cargo spawns to encourage exploration within bounds

### Walking/On-foot
- **Beach Boundaries**: Natural coastline prevents further travel
- **Visual Feedback**: Clear horizon line, distant terrain details fade

## Technical Implementation

### Phase 1: Remove Floating Origin (CURRENT TASK)
1. Delete `src/systems/floating_origin.rs`
2. Remove `WorldOffset` from all components
3. Update coordinate math to use direct world positions
4. Remove `WorldRoot` parent requirement

### Phase 2: Implement Boundaries  
1. Add `WorldBounds` resource with min/max coordinates
2. Create boundary collision system for vehicles
3. Add ocean/atmosphere visual boundaries
4. Implement gentle physics pushback (no hard walls)

### Phase 3: Content Pre-generation
1. Generate all chunks at startup (20×20 = 400 chunks)
2. Serialize procedural content to asset files
3. Replace runtime generation with scene loading
4. Cache road networks, buildings, vegetation as GLTF scenes

## Benefits
- **-1000 lines**: Remove entire floating origin system
- **Better Performance**: No coordinate transforms, pre-generated content
- **Easier Debugging**: True world coordinates everywhere  
- **Professional Feel**: Matches GTA, racing games, open-world RPGs
- **Simpler Physics**: Direct Rapier usage without offset complications

## Migration Strategy
- Backward compatibility during transition
- Feature flags for testing
- Gradual rollout of boundary systems
- Performance benchmarking at each step
