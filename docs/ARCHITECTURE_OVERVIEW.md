# Architecture Overview

This document provides a high-level overview of the GTA game project architecture, following the AI-Optimized Restructuring Plan.

## Project Structure

The project is organized as a Cargo workspace with domain-separated crates following a layered architecture:

```
gta_game/
├── engine_core/       # Pure utilities, no Bevy deps
├── engine_bevy/       # Bevy adapters & abstractions  
├── gameplay_sim/      # Physics, AI, rules
├── gameplay_render/   # LOD, culling, effects
├── gameplay_ui/       # HUD, menus, debug
├── game_core/         # Legacy compatibility
├── game_bin/          # main.rs, app wiring
├── test_utils/        # Testing utilities
└── tests/            # Integration tests
```

## Dependency Flow

![Crate Dependencies](graphs/crate_dependencies.dot)

Dependencies flow upward through the layers:
- **Engine Layer** → **Gameplay Layer** → **Game Layer**
- No circular dependencies between layers
- Clear separation of concerns

## Layer Responsibilities

### Engine Layer
- **engine_core**: Pure Rust utilities, math, configuration
- **engine_bevy**: Bevy framework adapters and abstractions

### Gameplay Layer  
- **gameplay_sim**: Physics simulation, AI, game rules
- **gameplay_render**: Rendering optimizations, LOD, culling
- **gameplay_ui**: User interface, HUD, debug panels

### Game Layer
- **game_core**: Legacy compatibility and integration
- **game_bin**: Application entry point and wiring

## Key Design Principles

### 1. Isolation Barriers
- **Public API Surface**: Only `prelude.rs` modules are publicly exported
- **Lint Walls**: Every crate has `#![deny(clippy::all, clippy::pedantic)]`
- **Feature Guards**: Conditional compilation for experimental features
- **System Sets**: Clear scheduling boundaries (PRE_PHYSICS, PHYSICS, POST_PHYSICS, RENDER_PREP)

### 2. Testing Foundation
- **Unit Tests**: Deterministic World tests for each system
- **Property Tests**: Validation of physics constraints and ranges
- **Golden Frame Tests**: Render hash comparison for visual regression
- **Save-Load Roundtrip**: World serialization integrity

### 3. Performance Optimization
- **Batch Processing**: Parallel job system with 300%+ performance gains
- **Unified Culling**: Distance-based entity management
- **LOD System**: Progressive quality reduction based on distance
- **GPU-Ready**: Prepared for compute shader implementation

### 4. Data-Driven Configuration
- **RON-based Config**: All game parameters configurable via files
- **Entity Factory**: Single source of truth for entity creation
- **Component Templates**: Reusable entity definitions

## System Architecture

### System Headers
All systems follow a standardized header format:
```rust
//! ───────────────────────────────────────────────
//! System:   system_name
//! Purpose:  Brief description
//! Schedule: When it runs (Update, FixedUpdate, etc.)
//! Reads:    Components/resources it reads
//! Writes:   Components/resources it writes
//! Invariants: Key constraints maintained
//! Owner:    Responsible team
//! ───────────────────────────────────────────────
```

### Ownership Model
- **@core-team**: Engine layer and critical infrastructure
- **@simulation-team**: Physics, AI, and game mechanics
- **@render-team**: Rendering and visual systems
- **@ui-team**: User interface and debug panels
- **@qa-team**: Testing and validation

## Performance Characteristics

### Target Metrics
- **Frame Rate**: 60+ FPS consistently
- **Entity Culling**: Buildings 300m, vehicles 150m, NPCs 100m
- **System Timing**: Road gen 0.5s, dynamic content 2.0s, culling 0.5s
- **Spawn Rates**: Ultra-reduced (buildings 8%, vehicles 4%, trees 5%, NPCs 1%)

### Optimization Strategies
- **Distance Caching**: 5-frame cache with 2048 entry limit
- **Batch Processing**: Parallel processing of similar entities
- **LOD Management**: Progressive quality reduction
- **Memory Pooling**: Reuse of allocated objects

## Development Guidelines

### Code Style
- **Naming**: snake_case for variables/functions, PascalCase for structs
- **Import Order**: External crates, std, bevy prelude, local crate
- **Error Handling**: Prefer `if let`/`match` over `unwrap()`
- **Constants**: Use Bevy constants (Vec3::ZERO, Transform::IDENTITY)

### Safety Requirements
- **Physics Validation**: Clamp positions, velocities, and forces
- **Collision Groups**: Proper group assignment for all entities
- **Bounds Checking**: Prevent entities from leaving valid game area
- **Finite Value Checks**: Ensure all math operations produce valid results

## Future Roadmap

### Phase 5: Advanced Features
- **Compute Shaders**: GPU-based culling and batch processing
- **Streaming**: Dynamic world loading and unloading
- **Networking**: Multiplayer support with authority model
- **Scripting**: Lua integration for gameplay logic

### Phase 6: Production Readiness
- **Asset Pipeline**: Automated asset processing and optimization
- **Performance Profiling**: Detailed metrics and optimization tools
- **Platform Support**: Multi-platform deployment
- **Content Tools**: Level editor and asset management

## Resources

- **[RFC Index](RFC_INDEX.md)**: Complete list of design documents
- **[Dependency Graphs](graphs/)**: Visual architecture diagrams
- **[CODEOWNERS](../CODEOWNERS)**: Team ownership assignments
- **[AGENT.md](../AGENT.md)**: Development environment setup
