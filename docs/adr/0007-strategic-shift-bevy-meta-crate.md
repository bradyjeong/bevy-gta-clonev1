# ADR-0007: Strategic Shift to Bevy 0.16.1 Meta-Crate

## Status
Accepted

## Context
The current architecture uses `bevy_ecs 0.13` with 6+ micro-crates (`amp_core`, `amp_math`, `amp_spatial`, `amp_gpu`, `amp_world`, etc.). This approach has created several critical issues:

1. **Ecosystem Misalignment**: Fighting Bevy's integrated systems, losing asset pipeline, rendering, input handling
2. **Development Tax**: Reinventing RON loaders, wgpu wrappers, and other solved problems
3. **Compilation Overhead**: Cross-crate dependencies dominate CI time (40%+), cause version conflicts
4. **Test Complexity**: Fragile mocked ECS worlds instead of integrated Bevy App instances
5. **Upgrade Path Risk**: Future Bevy 0.17+ upgrades require multi-month re-integration

## Decision
Adopt Oracle's recommended **Bevy 0.16.1 + Strategic 4-5 Crate Structure**:

```
├─ crates/
│   ├─ amp_core/          # Pure Rust utilities, error handling (no Bevy deps)
│   ├─ amp_math/          # glam re-exports, Morton, AABB (no Bevy deps)  
│   ├─ amp_engine/        # Bevy 0.16.1 dependency, engine plugins
│   ├─ amp_gameplay/      # Game systems, components, prefabs
│   └─ amp_tools/         # xtask, build pipeline helpers (optional)
```

### Migration Strategy (10-14 Days)
1. **Days 1-2**: Branch & lock Bevy 0.16.1 versions
2. **Days 3-4**: Consolidate `amp_spatial`, `amp_gpu`, `amp_world` into `amp_engine`
3. **Days 5-6**: Replace custom RON loader with Bevy's asset pipeline
4. **Days 7-9**: Rewrite tests to use `App::new().add_plugins(DefaultPlugins)`
5. **Days 10-14**: Documentation, stabilization, playtest

### Key Changes
- **Full Bevy Stack**: Use `bevy = "0.16.1"` instead of `bevy_ecs = "0.13"`
- **Rust 2024 Edition**: Upgrade to modern language features
- **Crate Consolidation**: Reduce from 6+ to 4-5 strategic crates
- **Asset Pipeline**: Use Bevy's integrated RON/GLTF loaders
- **Plugin Architecture**: Move customizations to Bevy plugins within `amp_engine`

## Consequences

### Positive
- **Ecosystem Leverage**: Full access to Bevy plugins, examples, community support
- **Compile Performance**: 30-40% faster builds with reduced cross-crate overhead
- **Test Reliability**: Integrated App-based testing instead of mocked components
- **Future-Proofing**: Bevy 0.17+ upgrades become `cargo upgrade` + minor fixes
- **Amp Productivity**: Clear module boundaries without micro-crate coordination tax

### Negative
- **Migration Effort**: 10-14 day investment to restructure codebase
- **Dependency Size**: Larger binary from full Bevy vs. just ECS
- **Learning Curve**: Team must adapt to Bevy conventions for plugins/systems

### Mitigation
- Archive current branch for rollback safety
- Maintain module-level boundaries within larger crates
- Use feature flags to minimize unused Bevy components
- Follow Bevy schedule conventions for system organization

## Implementation Notes
- Keep `amp_core` and `amp_math` dependency-free for fast compilation
- Only `amp_engine` and `amp_gameplay` depend on Bevy
- Use `pub mod prelude` pattern for clean imports
- Document cross-module contracts in ADRs immediately
- Maintain coverage >70% throughout migration

## References
- [Oracle Consultation: Architecture Strategy](../oracle-consultations.md)
- [Bevy 0.16.1 Migration Guide](https://bevyengine.org/learn/migration-guides/0-15-to-0-16/)
- [Strategic Workspace Structure](../Agent.md#architecture-strategy)
