Spawn multiple subagents to identify the highest performance optimization opportunities in the codebase.

Each subagent analyzes a specific performance category:

1. **ECS Performance Agent**: Analyze entity-component systems for:
   - Unnecessary system queries and filters
   - Missing ParallelCommands or batch operations
   - Inefficient component access patterns
   - Missing Change detection filters (Changed<T>, Added<T>)
   - System ordering issues causing unnecessary iterations

2. **Physics Performance Agent**: Analyze physics systems for:
   - Unnecessary collision checks or raycasts
   - Missing collision groups or layers
   - Inefficient damping or constraint configurations
   - Velocity clamping issues causing solver overhead
   - Missing CCD or sleeping body optimizations

3. **Rendering Performance Agent**: Analyze rendering systems for:
   - Inefficient mesh/material usage (missing MeshCache)
   - Unnecessary render passes or cameras
   - Missing frustum culling or VisibilityRange
   - Particle system overdraw or excessive spawn rates
   - Inefficient texture/asset loading

4. **Asset/Memory Agent**: Analyze asset management for:
   - Duplicate asset loads or missing handles
   - Large assets loaded but rarely used
   - Missing asset unloading or cleanup
   - Inefficient serialization (RON files)
   - Memory leaks in entity spawning/despawning

5. **Algorithm/Logic Agent**: Analyze game logic for:
   - O(n²) or worse algorithms in hot paths
   - Unnecessary allocations per frame
   - String concatenation or formatting in loops
   - Inefficient data structures (Vec where HashMap better)
   - Missing memoization or caching

Each agent should:
- List top 3-5 performance issues found
- Estimate potential performance impact (High/Medium/Low)
- Suggest specific code changes with file locations
- Prioritize quick wins vs long-term optimizations

Present consolidated findings ranked by impact, with actionable next steps.
