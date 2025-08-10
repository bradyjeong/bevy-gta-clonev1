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

## Event-Driven Architecture: First Principles
CORE PRINCIPLE: Events decouple systems while maintaining explicit data flow.

### Event-Driven Guidelines
- **Cross-Plugin Communication**: Always use Bevy events between plugins
- **Explicit Data Flow**: Every message visible in schedule with clear ordering
- **Lightweight Events**: Keep events small (≤128 bytes), Copy/Clone, no world references
- **One Event Per Concern**: Avoid kitchen-sink generic events requiring runtime casting
- **Documentation**: Each event group in dedicated module with clear purpose

### When to Use Events vs Direct Access
**USE EVENTS FOR:**
- Cross-plugin communication (mandatory per architectural boundaries)
- Entity lifecycle (spawn, despawn, state changes)
- Game logic triggers (damage, interactions, achievements)
- User actions (button presses, menu selections)
- System coordination (phase transitions, mode changes)

**USE DIRECT ACCESS FOR:**
- Core engine systems (renderer, physics, audio, input primitives)
- Performance-critical tight loops (movement updates, collision detection)
- Simple utility functions (math, string processing, data structures)
- Intra-plugin high-frequency data (position updates, animation frames)
- Read-only shared data (configurations, constants, lookup tables)

### Event Implementation Rules
- Events cleared every frame (O(n) performance)
- Multiple readers can consume same event concurrently
- Name systems after events: `handle_spawn_car_event`
- Use `.before()/.after()` for explicit system ordering
- Add debug instrumentation for event counts under debug-ui feature
- Keep stateless builder functions as helpers inside event handlers

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
- **Plugins communicate via Bevy events only** (except performance-critical cases below)
- **No direct system-to-system calls** across plugin boundaries
- **Resources for shared state**, not global variables
- **Each plugin owns its components**
- **Direct access allowed within plugins** for high-frequency operations
- **Utility modules** (math, data structures) can be directly imported anywhere

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
- **Event-First**: Use events for cross-plugin communication before considering direct imports
- **Direct Import Exceptions**: Core engine (Bevy systems), utilities (math, data structures), performance-critical intra-plugin code
- **Avoid circular dependencies** between modules
- **Prefer local imports** over glob imports
- **Keep external dependencies minimal**
- **One module per event group** for discoverability

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

## Asset-Driven Control System
Primary control configuration using RON (Rusty Object Notation) files following simplicity principles.

### System Architecture (Simplified)
- **Asset Configuration**: `assets/config/vehicle_controls.ron` - Single source of truth for ALL controls
- **Asset Processing**: `src/systems/input/asset_based_controls.rs` - Loads RON → ControlState component
- **Control State**: `src/components/control_state.rs` - Pure data component for all input
- **Plugin Registration**: `src/plugins/input_plugin.rs` - Single asset-based input system

### Simplicity Benefits
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

### RON Configuration Structure
```ron
VehicleControlsConfig(
    vehicle_types: {
        Car: VehicleControls(
            name: "Car",
            description: "Standard vehicle controls",
            primary_controls: [
                (action: Forward, key: ArrowUp, description: "Accelerate"),
                (action: TurnLeft, key: ArrowLeft, description: "Steer left"),
            ],
            secondary_controls: [
                (action: Turbo, key: Space, description: "Turbo boost"),
            ],
            meta_controls: [
                (action: Interact, key: KeyF, description: "Exit vehicle"),
            ],
        ),
    }
)
```

### Debug Commands
- `F3`: Display loaded control configuration debug info
- Asset reloading: Automatic when RON file changes during development

## Simplified Physics Systems
Following simplicity principles, complex physics have been replaced with maintainable alternatives.

### Vehicle Physics Options
- **Simple Vehicle Physics**: `src/systems/movement/simple_vehicle_physics.rs` - Direct force application (DEFAULT)
- **Realistic Vehicle Physics**: `src/systems/movement/realistic_vehicle_physics.rs` - Complex tire/aerodynamics (AVAILABLE)

### Aircraft Physics Options
- **Simple Aircraft**: `src/systems/movement/simple_aircraft.rs` - Direct control mapping (DEFAULT)
- **Complex Aircraft**: `src/systems/movement/aircraft.rs` - Advanced aerodynamics (AVAILABLE)

### Simplicity Benefits
- **Easy to Understand**: Linear force/rotation mapping instead of complex formulas
- **Maintainable**: No advanced physics calculations requiring aerospace knowledge
- **Performant**: Fewer calculations per frame
- **Flight Feel Preserved**: Responsive controls maintain aircraft experience
- **AGENT.md Compliant**: Single responsibility, clear data flow, minimal coupling

### Switching Physics Systems
To use complex physics, modify `src/plugins/vehicle_plugin.rs`:
```rust
// Change from:
simple_f16_movement.run_if(in_state(GameState::Jetting)),
// To:
f16_movement.run_if(in_state(GameState::Jetting)),
```

## Entry Points & Usage
- Main game: `cargo run` (starts full GTA-style open world game)
- Debug features: `cargo run --features debug-movement,debug-audio,debug-ui`
- Examples:
  - Vehicle control: `cargo run --example vehicle_control_example`
  - Physics integration: `cargo run --example physics_utils_integration`
  - Vegetation demo: `cargo run --example vegetation_instancing_demo`
