# STRATEGIC SHIFT: Bevy 0.16.1 Migration Plan

## Summary

Oracle-guided strategic shift from bevy_ecs 0.13 + micro-crates to Bevy 0.16.1 + strategic modularity. Current architecture fights ecosystem, creates unnecessary complexity.

## Problems with Current Architecture

### 1. Ecosystem Misalignment
- **bevy_ecs 0.13** instead of full Bevy 0.16.1 stack
- **Custom RON loaders** instead of Bevy asset pipeline
- **wgpu abstractions** instead of Bevy rendering
- **Missing integration** with Bevy plugins, examples, community

### 2. Development Overhead
- **Cross-crate compilation** dominates CI time (40%+)
- **6+ micro-crates** create coordination tax
- **Version conflicts** between workspace members
- **Test complexity** from mocked ECS vs integrated App

### 3. Future Risk
- **Bevy 0.17+ upgrades** require multi-month re-integration
- **Ecosystem drift** as community moves forward
- **Maintenance burden** for custom solutions

## Oracle's Target Architecture

```
├─ crates/
│   ├─ amp_core/          # Pure Rust utilities, error handling (no Bevy deps)
│   ├─ amp_math/          # glam re-exports, Morton, AABB (no Bevy deps)  
│   ├─ amp_engine/        # Bevy 0.16.1 dependency, engine plugins
│   ├─ amp_gameplay/      # Game systems, components, prefabs
│   └─ amp_tools/         # xtask, build pipeline helpers (optional)
├─ examples/              # city_demo.rs
└─ docs/adr/              # Architecture Decision Records
```

## Migration Roadmap (10-14 Days)

### Days 1-2: Branch & Lock Versions
- `cargo add bevy@0.16.1 --features bevy_winit,bevy_gltf` in amp_engine
- Freeze current branch for rollback safety
- Update to Rust 2024 edition

### Days 3-4: Crate Consolidation  
- Move amp_spatial, amp_gpu, amp_world → amp_engine modules
- Delete empty crates, fix mod paths
- Preserve git history with `git mv`

### Days 5-6: Asset Pipeline Integration
- Delete custom RON loader
- Register AssetLoader plugin using Bevy's RON support
- Update asset loading to use Bevy Handle<T> system

### Days 7-9: Test Modernization
- Rewrite integration tests: `App::new().add_plugins(DefaultPlugins)`
- Update coverage thresholds for reduced LOC
- Re-enable examples/ in workspace

### Days 10-14: Stabilization
- Create ADR-007 documenting shift
- Update all documentation
- 30-minute playtest validation
- Tag v0.2.0-alpha

## Expected Benefits

### Immediate
- ✅ **30-40% faster builds** from reduced cross-crate overhead
- ✅ **Ecosystem access** to Bevy plugins, examples, community
- ✅ **Test reliability** from integrated App instances
- ✅ **Development velocity** from standard patterns

### Long-term  
- ✅ **Future-proofing** for Bevy 0.17+ upgrades
- ✅ **Amp productivity** with clear boundaries, no coordination tax
- ✅ **Community alignment** with standard Bevy patterns
- ✅ **Reduced maintenance** burden from custom solutions

## Status

- ✅ Oracle consultation complete
- ✅ ADR-007 created
- ✅ Agent.md updated  
- ✅ Documentation aligned
- 🔄 Ready for implementation

**This strategic shift addresses architectural debt and aligns with ecosystem best practices for sustainable development.**
