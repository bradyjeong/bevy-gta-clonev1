# AGENT.md

## Code Philosophy: Simplicity First
CORE PRINCIPLE: Simplicity is the key to this codebase.

### What Simplicity Means
- NOT fewer features - we can have rich functionality
- NO tangled/interweaved code - avoid complex interdependencies
- Clear separation of concerns - each module has one clear purpose
- Minimal coupling - components should work independently when possible
- Straightforward data flow - easy to trace how data moves through the system

### Complexity Limits
- Avoid deep inheritance hierarchies
- Prefer composition over complex inheritance
- Keep functions focused on single responsibilities
- Minimize cross-module dependencies
- Use clear, direct APIs between components
- Avoid clever code that requires deep understanding

### When Adding Features
- Can this be implemented without tangling existing code?
- Does this maintain clear boundaries between modules?
- Is the data flow still easy to follow?
- Would a new developer understand this quickly?

## Commands
- Build: `cargo build` | Check: `cargo check` | Test: `cargo test test_name`
- Lint: `cargo clippy` | Format: `cargo fmt` | Run: `cargo run`
- Features: `cargo run --features debug-movement,debug-audio,debug-ui`

## Project Structure
- Bevy 0.16.1 game using Rust 2024 edition, bevy_rapier3d 0.30.0 physics
- Core dependencies: bevy 0.16.1, bevy_rapier3d 0.30.0, bytemuck 1.18, rand 0.8, serde 1.0
- Plugin-based: components/, systems/, plugins/, setup/, factories/

### Architectural Boundaries
- **components/**: Pure data structures, no logic
- **systems/**: Pure functions that operate on components
- **plugins/**: Self-contained modules with clear interfaces
- **setup/**: One-time initialization, no ongoing state
- **factories/**: Entity creation patterns, stateless

### Module Communication Rules
- Plugins communicate via Bevy events only
- No direct system-to-system calls
- Resources for shared state, not global variables
- Each plugin owns its components

## Code Style
- snake_case for variables/functions, PascalCase for structs/components
- Import order: external crates, std, bevy prelude, local crate
- Prefer `if let`/`match` over unwrap(), use Bevy constants (Vec3::ZERO)
- Components: `#[derive(Component)]` + `Default`, systems in subdirs
- Safety: Validate physics values, clamp positions/dimensions, use collision groups
- Comments: `//` style, 4-space indent, trailing commas

### Simplicity Rules
- Prefer explicit over implicit (no magic)
- Max 4-5 function parameters (use structs for more)
- Avoid nested Option/Result chains
- Keep structs under 10 fields
- Single responsibility per function
- Clear, descriptive names over clever ones

### Dependency Guidelines
- Only import what you use
- Avoid circular dependencies between modules
- Prefer local imports over glob imports
- Keep external dependencies minimal

## Performance
- Target 60+ FPS, entity culling (buildings 300m, vehicles 150m, NPCs 100m)
- System timing intervals: road gen 0.5s, dynamic content 2.0s, culling 0.5s
- Ultra-reduced spawn rates: buildings 8%, vehicles 4%, trees 5%, NPCs 1%
- Distance caching: Avoids repeated calculations (5-frame cache, 2048 entry limit)
- Cache debug: Press F3 or wait 5s for cache performance stats

## Testing Guidelines
- **Framework**: Rust built-in testing with Bevy test utilities
- **Pattern**: Use `App::new().add_plugins(MinimalPlugins)` for Bevy tests
- **Types**: Unit tests (inline modules), integration tests (tests/ dir), performance tests
- **Run**: `cargo test test_name` for specific tests, `cargo test` for all
- **Focus**: Component validation, LOD/culling behavior, physics integration, performance

## Debugging & Error Handling
- Clear error messages with context
- Use `expect()` with descriptive messages over `unwrap()`
- Simple debug tools: visual overlays, console output
- Fail fast and clearly, don't hide errors
- Debug features toggleable via features flags

## Memory & Asset Management
- Entity culling: Buildings 300m, vehicles 150m, NPCs 100m
- Mesh caching via `MeshCache` resource
- Asset loading: Lazy load, unload distant assets
- Entity limits via `EntityLimits` resource
- Distance caching: 5-frame cache, 2048 entry limit

## Bevy-Specific Patterns
- Systems: Pure functions, single responsibility
- Resources: Shared state, avoid global variables
- Events: Cross-plugin communication only
- States: Use `GameState` for major game modes
- Plugins: Self-contained, clear interfaces
- System ordering: Use `.after()` and `.before()` explicitly

## Subagent Context Protocol
CRITICAL: Always pass AGENT.md context to subagents for consistency.

### When Using Task Tool
- Always include this AGENT.md file in the context parameter
- Reference specific sections relevant to the subagent's task
- Ensure subagents follow the same protocols (version verification, commands, architecture)
- Include any recent changes or updates that affect the task

### Example Task Call
Task tool with context: "Read AGENT.md for project structure, commands, and protocols. Follow the version management protocol strictly."

This ensures all subagents maintain consistency with:
- Version management protocols
- Architectural boundaries
- Code philosophy
- Command usage
- Project structure understanding

## Never Assume - Always Verify
CRITICAL: Never assume package versions. Always verify from official sources first.

- Package versions change frequently
- Security vulnerabilities are discovered regularly
- Always check official package registries before making version claims
- Update this file when versions are verified

## Entry Points & Usage
- Main game: `cargo run` (starts full GTA-style open world game)
- Debug features: `cargo run --features debug-movement,debug-audio,debug-ui`
- Examples:
  - Vehicle control: `cargo run --example vehicle_control_example`
  - Physics integration: `cargo run --example physics_utils_integration`
  - Vegetation demo: `cargo run --example vegetation_instancing_demo`
