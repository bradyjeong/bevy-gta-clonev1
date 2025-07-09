# AGENT.md

## Commands
- Build: `cargo build --workspace` | Check: `cargo check --workspace` | Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets --all-features` | Format: `cargo fmt --all`
- Rustdoc: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features` (lint-enabled)
- Coverage: `cargo llvm-cov --workspace --all-features` | Coverage Gate: Minimum 65% (will increase to 70%)
- Run Example: `cargo run --bin minimal`
- Dev Tools: `cargo xtask ci` (full CI pipeline), `cargo xtask fmt`, `cargo xtask test`
- Documentation: `cargo xtask doc` (generate), `cargo xtask doc-validate` (validate)

## Project Vision
**AAA-Level Open World Game** - GTA-style game built with Bevy 0.16.1 using modern Rust 2024 edition
- **Target**: Professional game development with Amp-optimized workflow
- **Focus**: Clean architecture, fast iteration, team scalability

## Architecture Strategy
**8-Week Extraction-Based Restart** - Clean workspace preserving proven systems

### Current Workspace Structure (Week 1 Complete)
```
├─ crates/
│   ├─ amp_core/          # Core error handling and utilities
│   ├─ amp_math/          # Spatial mathematics and Morton encoding  
│   ├─ amp_spatial/       # Hierarchical spatial partitioning
│   ├─ amp_gpu/           # GPU abstraction over wgpu
│   ├─ amp_world/         # ECS world management (basic)
│   └─ [future crates]    # amp_physics, amp_ai, amp_render
├─ examples/              # Example applications (minimal.rs)
├─ tools/xtask/           # Development automation
├─ docs/                  # Organized documentation
│   ├─ architecture/      # Technical architecture docs
│   ├─ guides/            # Development guides  
│   ├─ api/               # Auto-generated API docs
│   └─ adr/               # Architecture Decision Records
└─ .github/workflows/     # CI/CD pipeline
```

### Key Principles
- **Domain Boundaries**: Clear separation between engine and game logic
- **Bevy Integration**: Leverage Bevy's ECS, don't fight it
- **Extract, Don't Rebuild**: Port proven algorithms, rebuild only interfaces
- **Amp Optimized**: Fast compile times, parallel development, clean CI

## Development Workflow
- **Weekly Checkpoints**: Prevent scope creep with deliverable demos
- **60 FPS Target**: Performance gates at each milestone
- **Test Coverage**: 78 tests passing, comprehensive coverage
- **CI Time**: Full workspace build <20 seconds
- **PR Size**: ≤500 LOC per merge

## Code Style
- snake_case for variables/functions, PascalCase for structs/components
- Import order: external crates, std, bevy prelude, local crate
- Prefer `if let`/`match` over unwrap(), use Bevy constants (Vec3::ZERO)
- Components: `#[derive(Component)]` + `Default`, systems in subdirs
- Safety: Validate physics values, clamp positions/dimensions, use collision groups
- Comments: `//` style, 4-space indent, trailing commas

## Performance Targets
- **Target**: 60+ FPS on desktop, stable frame times
- **Culling**: Distance-based (buildings 300m, vehicles 150m, NPCs 100m)
- **Memory**: Object pools, per-frame arenas, minimal allocations
- **Profiling**: Built-in counters, frame analysis, bottleneck detection

## Technical Systems Implemented
### amp_core
- Engine-wide error handling with thiserror
- Result<T> alias for consistent error handling
- 11 unit tests covering all error variants

### amp_math
- Morton 3D encoding for efficient spatial indexing
- AABB and Sphere bounding volume implementations
- Transform utilities with builder patterns
- 40 unit tests with comprehensive coverage

### amp_spatial  
- RegionId with Morton-encoded spatial identifiers
- Hierarchical clipmap for multi-level detail management
- Async streaming provider interface
- 22 unit tests covering all functionality

### amp_gpu
- wgpu context and surface management
- GPU abstraction with error handling
- Purple screen rendering example
- 3 unit tests for core functionality

### amp_world
- Basic ECS world management wrapper
- Future integration point for Bevy systems
- 2 unit tests for world creation

## Current Status
✅ **WEEK 1 COMPLETE** - Foundation established with 78 passing tests
- All crates compile cleanly with `-Dwarnings`
- CI/CD pipeline operational
- Example application runs successfully
- Oracle's Week 1 success criteria met

🚀 **WEEK 2 - DAY 1 IN PROGRESS** - CI Quality Gates & Hygiene
- ✅ Coverage gate added (65% minimum, 82.11% current)
- ✅ Rustdoc linting enabled (no warnings found)
- ✅ Publishing hygiene fixed (categories/keywords added)
- ✅ CI workflow updated with quality gates
- Following Oracle's strict Week 2 strategy

## Oracle Guidance
- **Strategic Decisions**: Documented in [Oracle Consultations](docs/oracle-consultations.md)
- **Architecture Decisions**: Captured in [ADR system](docs/adr/README.md)
- **Key Principle**: Follow Oracle's 8-week extraction plan strictly
- **Weekly Verification**: Consult Oracle for milestone checkpoints

## Next Steps (Week 2)
- Port RON configuration system → config_core crate
- Extract unified entity factory → gameplay_factory crate
- Begin LOD and batch processing systems
- Maintain test coverage above 70%
