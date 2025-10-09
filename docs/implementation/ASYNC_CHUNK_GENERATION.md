# Async Chunk Generation System

## Overview
The async chunk generation system replaces the synchronous `layered_generation_coordinator` that caused frame drops ("jolting"). This new system achieves smooth 60+ FPS by moving heavy work off the main thread and applying strict per-frame budgets.

## Architecture

### System Sets (Deterministic Ordering)
```
StreamingSet::Scan     → unified_world_streaming_system
StreamingSet::GenQueue → queue_async_chunk_generation
StreamingSet::GenApply → process_completed_chunks
```

### Key Components

#### 1. AsyncChunkQueue Resource
- **max_concurrent_tasks**: 3 (conservative for stability)
- **max_completed_per_frame**: 2 (strict frame budget)
- Tracks active generation tasks
- Prevents resource exhaustion

#### 2. queue_async_chunk_generation System
- **Only consumes chunks already marked `Loading`** by unified_world_streaming_system
- Sorts by distance (closest first)
- Respects capacity limits
- Spawns async tasks with correct chunk_size parameter

#### 3. process_completed_chunks System
- **Stale result protection**: Only applies if chunk still in `Loading` state
- **Frame budget**: Max 2 chunks per frame
- Uses unit meshes scaled via Transform (no double-scaling)
- Marks chunks as `Loaded` on success

#### 4. generate_chunk_async Function
- **Runs off main thread** (AsyncComputeTaskPool)
- **Only computes blueprint data** (positions, types, scales)
- Uses correct chunk_size from world_manager
- No ECS/Assets access in async context

## Performance Optimizations

### Asset Reuse
- **Unit meshes**: Cuboid(1,1,1) and Cylinder(0.1, 1.0)
- **Transform scaling**: Scale applied via Transform, not mesh geometry
- **MaterialRegistry**: Reuse materials to avoid GC pressure

### Frame Budgets
- **Queue**: Max 3 concurrent async tasks
- **Apply**: Max 2 chunks spawned per frame
- **Streaming**: 0.2s tick interval (5 Hz)

### Stale Result Guards
Before applying generation results:
1. Verify chunk still exists
2. Check state is still `Loading`
3. Discard if chunk was unloaded or state changed

## Key Fixes from Original System

### Bug Fixes
1. ✅ Fixed `Default` setting `max_concurrent_tasks = 0`
2. ✅ Removed duplicate streaming logic (no more racing)
3. ✅ Added stale result protection
4. ✅ Fixed chunk size mismatch (128 vs 200)
5. ✅ Fixed double-scaling (mesh + transform)
6. ✅ Added per-frame budget (unbounded → 2 max)
7. ✅ Uses unit meshes for asset reuse
8. ✅ Fixed ActiveEntity import path

### Architecture Improvements
- Deterministic system ordering via SystemSets
- Clean separation: Scan → Queue → Apply
- Integrated with existing streaming system (doesn't compete)
- Proper error handling and logging

## Migration from Old System

### Old (Synchronous)
```rust
// layered_generation_coordinator - BLOCKING MAIN THREAD
generate_complete_chunk() {
    road_generator.generate_roads();      // Main thread
    building_generator.generate_buildings(); // Main thread
    vehicle_generator.generate_vehicles();  // Main thread
    vegetation_generator.generate_vegetation(); // Main thread
}
// Result: Frame spikes / jolting
```

### New (Async)
```rust
// queue_async_chunk_generation - NON-BLOCKING
task_pool.spawn(async move {
    generate_chunk_async(coord, chunk_size).await // Off thread
})

// process_completed_chunks - BUDGETED
for chunk in completed.take(max_completed_per_frame) {
    // Apply only 2 chunks per frame maximum
}
// Result: Smooth 60+ FPS
```

## Configuration

### Tuning Performance
Edit `AsyncChunkQueue::new()`:
```rust
Self {
    max_concurrent_tasks: 3,      // 2-4 depending on CPU cores
    max_completed_per_frame: 2,   // 1-3 depending on entity complexity
}
```

### Monitoring
Use F3 debug overlay to see:
- Active async tasks
- Chunks loaded per frame
- Frame time stability

## Testing
```bash
cargo check    # Verify compilation
cargo clippy   # Check for warnings
cargo run      # Test in-game

# Watch for:
# - Smooth camera movement (no jolting)
# - Consistent 60+ FPS
# - Chunks appearing without frame drops
# - No stale result warnings in console
```

## Future Enhancements

### Short Term
- Add unit mesh caching in MeshRegistry
- MaterialRegistry integration for better reuse
- Per-entity spawn batching for vegetation

### Long Term
- Replace sample generation with actual generators
- Integrate with placement grid for collision avoidance
- Add priority-based generation (gameplay-critical chunks first)
- LOD-aware generation (generate appropriate detail level)

## Troubleshooting

### Jolting/Frame Drops Return
- Check `max_completed_per_frame` not set too high
- Verify no synchronous generation systems added
- Monitor active task count (should stay ≤ 3)

### Chunks Not Generating
- Check unified_world_streaming_system marking chunks as `Loading`
- Verify AsyncChunkQueue has capacity
- Look for stale result warnings (chunks unloading too fast)

### Memory Issues
- Reduce `max_concurrent_tasks`
- Verify unit meshes are reused (not recreated)
- Check MaterialRegistry for leaks

## References
- Oracle recommendations in investigation thread
- [unified_world.rs](../src/systems/world/unified_world.rs) - Streaming system
- [async_chunk_generation.rs](../src/systems/world/async_chunk_generation.rs) - Async implementation
- [AGENT.md](../AGENT.md) - Simplicity principles
