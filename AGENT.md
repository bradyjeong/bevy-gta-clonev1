# AGENT.md

## Commands
- Build: `cargo build --workspace` | Check: `cargo check --workspace` | Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets --all-features` | Format: `cargo fmt --all`
- Rustdoc: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- Coverage: `cargo llvm-cov --workspace --all-features` | Coverage Gate: Minimum 70%
- Run Example: `cargo run --example city_demo`
- Dev Tools: `cargo xtask ci` (full CI pipeline), `./scripts/pre-commit-check.sh` (before commits)

## Development Workflow
- **During Development**: `cargo watch -c` (continuous compilation)
- **Before Committing**: `./scripts/pre-commit-check.sh` (full CI simulation)
- **Auto-format**: `cargo fmt --all` (run frequently)
- **Golden Rule**: Never commit without running pre-commit checks

## Project Vision
**AAA-Level Open World Game** - GTA-style game built with Bevy 0.16.1 using Rust 2021 edition
- **Target**: Professional game development with Amp-optimized workflow
- **Focus**: Ecosystem alignment, fast iteration, clear boundaries

## Architecture Strategy
**Oracle-Guided Strategic Shift** - Bevy 0.16.1 + Strategic 4-5 Crate Structure

### Oracle's Strategic Workspace Structure
```
├─ crates/
│   ├─ amp_core/          # Pure Rust utilities, error handling (no Bevy deps)
│   ├─ amp_math/          # glam re-exports, Morton, AABB (no Bevy deps)  
│   ├─ amp_engine/        # Bevy 0.16.1 dependency, engine plugins
│   ├─ amp_gameplay/      # Game systems, components, prefabs
│   └─ amp_tools/         # xtask, build pipeline helpers (optional)
├─ examples/              # Example applications (city_demo.rs)
├─ docs/adr/              # Architecture Decision Records
└─ .github/workflows/     # CI/CD pipeline
```

### Key Principles
- **Ecosystem Alignment**: Use full Bevy 0.16.1, don't fight the ecosystem
- **Strategic Modularity**: 4-5 crates max, clear domain boundaries
- **Amp Optimized**: Focused surfaces for parallel agent development
- **Compile Speed**: Incremental builds, minimal cross-crate dependencies

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
🔄 **STRATEGIC SHIFT IN PROGRESS** - Oracle-Guided Architecture Change
- **Decision**: Moving from bevy_ecs 0.13 + micro-crates to Bevy 0.16.1 + strategic modularity
- **Reason**: Current setup fights Bevy ecosystem, creates unnecessary complexity
- **Target**: 4-5 crate structure with full Bevy 0.16.1 for ecosystem alignment
- **Timeline**: 10-14 day migration following Oracle's roadmap
- **Status**: Documentation updated, migration planning complete

## Oracle Guidance
- **Strategic Decisions**: Documented in [Oracle Consultations](docs/oracle-consultations.md)
- **Architecture Decisions**: Captured in [ADR system](docs/adr/README.md)
- **Key Principle**: Follow Oracle's strategic shift to Bevy 0.16.1 strictly
- **Weekly Verification**: Consult Oracle for milestone checkpoints

## Maintenance & Live Documentation

### Files Requiring Regular Updates
These files must be kept current and reviewed during every strategic change:

**Core Documentation:**
- `Agent.md` - Commands, architecture, status (THIS FILE)
- `README.md` - Public face, quick start, features  
- `STRATEGIC_SHIFT.md` - Current migration status and roadmap
- `CONTRIBUTING.md` - Development workflow, code style, commit guidelines

**Architecture Records:**
- `docs/adr/README.md` - Index of all architectural decisions
- `docs/oracle-consultations.md` - Oracle guidance and strategic decisions
- Latest ADR (currently ADR-0007) - Active architectural strategy

**Configuration Files:**
- `Cargo.toml` - Workspace dependencies, edition, version
- `examples/Cargo.toml` - Example dependencies and structure  
- `CODEOWNERS` - Ownership aligned with current crate structure
- `.github/workflows/ci.yml` - CI pipeline matching current architecture

**Status Tracking:**
- `IMPLEMENTATION_SUMMARY.md` - Current implementation status
- Test counts and coverage metrics in CI
- Performance benchmarks and targets

### Dead Weight Prevention
**Red Flags for Cleanup:**
- Documentation referencing removed crates (amp_spatial, amp_gpu, amp_world)
- Cargo.toml dependencies not used by any crate
- Examples that don't compile or run
- CI workflows testing non-existent targets
- README features that don't exist
- ADRs marked "Superseded" without clear replacement

**Maintenance Schedule:**
- **Every commit**: Verify Agent.md status reflects reality
- **Every architectural change**: Update all docs in this list
- **Every Oracle consultation**: Update oracle-consultations.md + create ADR if needed
- **Every milestone**: Verify README.md features match implementation

## Next Steps (Migration Implementation)
- Follow Oracle's 10-14 day migration plan in STRATEGIC_SHIFT.md
- Update all maintenance files during each migration phase
- Verify no dead weight accumulates during restructuring
