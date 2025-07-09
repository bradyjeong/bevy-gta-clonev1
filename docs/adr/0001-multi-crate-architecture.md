# ADR-0001: Multi-Crate Architecture

## Status
Accepted

## Context

The game engine needs to be organized in a way that supports:
- Fast compilation times during development
- Clear separation of concerns
- Team collaboration with parallel development
- Modular testing and benchmarking
- Future extensibility for additional systems

A monolithic crate structure would lead to:
- Slow compilation as the codebase grows
- Unclear dependencies between systems
- Difficulty in testing individual components
- Merge conflicts when multiple developers work on different systems

## Decision

We will organize the codebase into multiple focused crates:

```
├─ crates/
│   ├─ amp_core/          # Core error handling and utilities
│   ├─ amp_math/          # Spatial mathematics and Morton encoding  
│   ├─ amp_spatial/       # Hierarchical spatial partitioning
│   ├─ amp_gpu/           # GPU abstraction over wgpu
│   ├─ amp_world/         # ECS world management
│   └─ [future crates]    # amp_physics, amp_ai, amp_render
```

Each crate has a single, well-defined responsibility and follows these rules:
- No circular dependencies
- Clear dependency hierarchy (amp_core ← amp_math ← amp_spatial ← amp_gpu)
- Minimal public APIs
- Comprehensive test coverage per crate

## Consequences

### Positive
- **Faster compilation**: Only changed crates need to be recompiled
- **Parallel development**: Teams can work on different crates without conflicts
- **Clear interfaces**: Public APIs must be well-designed for cross-crate usage
- **Modular testing**: Each crate can be tested independently
- **Easier maintenance**: Bug fixes and features are contained within specific crates

### Negative
- **Initial complexity**: More setup overhead compared to single crate
- **Dependency management**: Need to carefully manage versions and features
- **Cross-crate refactoring**: Changes that span multiple crates require coordination
- **Documentation overhead**: Each crate needs its own documentation

## Implementation Notes

- Use workspace Cargo.toml for shared dependency versions
- Implement strict linting rules across all crates
- Each crate maintains its own README and tests
- Use semantic versioning for inter-crate dependencies
