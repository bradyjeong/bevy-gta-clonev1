# AGENT.md

## Table of Contents
- [Code Philosophy: Simplicity First](#code-philosophy-simplicity-first)
- [Module Communication](#module-communication)
- [ECS Patterns](#ecs-patterns)
- [Performance Optimization](#performance-optimization)
- [Commands](#commands)
- [Git Safety & Pre-commit Rules](#git-safety--pre-commit-rules)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Testing Guidelines](#testing-guidelines)
- [Debugging & Error Handling](#debugging--error-handling)
- [Asset-Driven Control System](#asset-driven-control-system)
- [Simplified Physics Systems](#simplified-physics-systems)

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

## Module Communication
CORE PRINCIPLE: Keep communication simple and direct.

### Basic Guidelines
- **Use judgment**: Direct calls when simpler, events when needed for decoupling
- **Resources for shared state**, not global variables  
- **Each plugin owns its components**
- **Utility modules** can be directly imported anywhere

### When to Use Events
- **One-to-many notifications** (damage → multiple handlers)
- **Decoupled game logic** (input → multiple system responses)
- **Entity lifecycle events** (spawn, despawn, state changes)

### Event Design
- Keep events simple and focused
- Use descriptive names for events and handlers
- Avoid overly generic events requiring runtime casting

## ECS Patterns
- Use Bevy's `commands.spawn().with_children()` for hierarchies
- Create helper functions for complex entity spawning
- Use `expect()` with descriptive messages over `unwrap()`
- Handle errors locally with `tracing::error!` for logging

## Performance Optimization
- `MeshCache` resource for shared geometry
- 60+ FPS target

## Commands
- Build: `cargo build` | Check: `cargo check` | Test: `cargo test test_name`
- Lint: `cargo clippy` | Format: `cargo fmt` | Run: `cargo run`
- Features: `cargo run --features debug-movement,debug-audio,debug-ui`

## Git Safety & Pre-commit Rules
CRITICAL safety rules for version control and code quality.

### Git Safety Rules
- **CRITICAL: NEVER use `git push --force` on main branch**
- **CRITICAL: NEVER auto-commit on main without explicit user instruction**
- Use `git push --force-with-lease` only on feature branches after verification
- Always verify current branch with `git branch` before any force push operation

### Pre-commit Verification
**ALWAYS run before any commit:**
```bash
cargo check && cargo clippy -- -D warnings && cargo test
```
- Fix all compilation errors and warnings before committing
- Run tests when code changes affect functionality
- Use `git status` to verify only intended files are staged
- Never commit broken or unformatted code

## Project Structure
- Bevy 0.16.1 game using Rust 2024 edition, bevy_rapier3d 0.30.0 physics
- Core dependencies: bevy 0.16.1, bevy_rapier3d 0.30.0, bytemuck 1.18, rand 0.8, serde 1.0
- Plugin-based: components/, systems/, plugins/, setup/, factories/

### Directories
- **components/**: Data structures
- **systems/**: Game logic functions  
- **plugins/**: Self-contained modules
- **setup/**: Initialization
- **factories/**: Entity creation helpers



## Code Style
- Follow Rust and Bevy best practices
- snake_case for variables/functions, PascalCase for structs/components
- Import order: external crates, std, bevy prelude, local crate
- Prefer `if let`/`match` over unwrap(), use Bevy constants (Vec3::ZERO)
- Components: `#[derive(Component)]` + `Default`, systems in subdirs
- Safety: Validate physics values, clamp positions/dimensions, use collision groups
- Comments: `//` style, 4-space indent, trailing commas
- **No emojis allowed** in code, comments, or documentation

### Code Formatting Discipline
- **ALWAYS run `cargo fmt` after any code modification**
- **NEVER commit unformatted code** - formatting inconsistencies cause merge conflicts
- Format all modified files before staging changes
- Use consistent indentation and spacing throughout the codebase



### Simplicity Rules
- Prefer explicit over implicit (no magic)
- Single responsibility per function
- Clear, descriptive names over clever ones

## Testing Guidelines
- **Framework**: Rust built-in testing with Bevy test utilities
- **Pattern**: Use `App::new().add_plugins(MinimalPlugins)` for Bevy tests
- **Run**: `cargo test test_name` for specific tests, `cargo test` for all

## Debugging & Error Handling
- Use `expect()` with descriptive messages over `unwrap()`
- Debug features toggleable via features flags
- Fail fast and clearly, don't hide errors



## Asset-Driven Control System
Primary control configuration using RON (Rusty Object Notation) files.

### System Architecture
- **Asset Configuration**: Split RON files in `assets/config/controls/` - Vehicle-specific control configs
- **Asset Processing**: `src/systems/input/asset_based_controls.rs` - Loads RON → ControlState component
- **Control State**: `src/components/control_state.rs` - Pure data component for all input
- **Plugin Registration**: `src/plugins/input_plugin.rs` - Single asset-based input system

### Benefits
- **Single Input Path**: Only asset-based controls (removed simple_input_mapping complexity)
- **No Code Changes**: Add new vehicles/controls by editing RON file only
- **Runtime Customization**: Players can modify controls without recompilation  
- **Clean Data Flow**: RON → ControlState → Movement Systems
- **Unified Interface**: All vehicles use same ControlState component

### Supported Vehicles
- **Walking**: Arrow keys movement, Shift run, F interact
- **Car/SuperCar**: Arrow keys throttle/brake/steering, Space turbo
- **Helicopter**: Arrow keys pitch/yaw, Shift/Ctrl vertical, F exit
- **F16**: Arrow keys pitch/roll, WASD throttle/yaw, Space afterburner
- **Yacht**: IJKL movement (configurable), Space boost, F exit



## Debug Commands
- `F3`: Toggle debug overlay (control configuration, cache performance stats)
- Asset reloading: Automatic when RON file changes during development

## Simplified Physics Systems

### Dynamic Arcade Physics (Final Solution)
- **Physics Model**: `RigidBody::Dynamic` for all vehicles (cars, helicopters, F16)
- **Control Method**: Direct velocity manipulation with automatic gravity/collision
- **Collision**: Handled automatically by Rapier physics solver
- **Ground Detection**: Automatic through Rapier gravity and contact resolution

### Vehicle Movement Systems
- **Car Movement**: `src/systems/movement/vehicles.rs` - Direct velocity control with asset-driven specs
- **Aircraft Movement**: `src/systems/movement/simple_aircraft.rs` - F16 and helicopter physics with RON configs

### Physics Configuration
- **Car Specs**: `assets/config/simple_car.ron` - Speed, rotation, emergency brake settings
- **Helicopter Specs**: `assets/config/simple_helicopter.ron` - Movement speeds and rotation rates  
- **F16 Specs**: `assets/config/simple_f16.ron` - Thrust, lift, and flight parameters

### Key Design Decisions
- **Dynamic Bodies**: Use Rapier's automatic gravity, collision, and contact resolution
- **High Damping**: Arcade feel with `linear_damping: 2.0-3.0, angular_damping: 8.0-10.0`
- **Direct Velocity Control**: Instant response without force calculations
- **No Manual Physics**: Removed `PhysicsUtilities::apply_ground_collision` - let Rapier handle it
- **Velocity Clamping**: Use `PhysicsUtilities::clamp_velocity` to prevent solver panics

### Benefits
- **No Physics Panics**: Proper velocity clamping prevents solver conflicts
- **Automatic Collision**: Vehicles properly land, collide, and stay on ground
- **Easy to Understand**: Direct velocity control with automatic physics handling
- **Maintainable**: No manual gravity, collision, or complex force calculations
- **Performant**: Rapier handles optimization, minimal per-frame calculations
- **Reliable Physics**: Vehicles behave predictably with proper collision response




