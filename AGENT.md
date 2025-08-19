# AGENT.md

## Table of Contents
- [Code Philosophy: Simplicity First](#code-philosophy-simplicity-first)
- [Event-Driven Architecture: First Principles](#event-driven-architecture-first-principles)
- [Modern ECS Patterns (Bevy 0.16+)](#modern-ecs-patterns-bevy-016)
- [Performance Optimization (Bevy 0.16+)](#performance-optimization-bevy-016)
- [Commands](#commands)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Testing Guidelines](#testing-guidelines)
- [Debugging & Error Handling](#debugging--error-handling)
- [Subagent Context Protocol](#subagent-context-protocol)

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

## Event-Driven Architecture & Module Communication
CORE PRINCIPLE: Events decouple systems while maintaining explicit data flow.

### Communication Rules
- **Cross-Plugin Communication**: Use events for coordination, direct access for computation
- **Plugins communicate via events for coordination, direct access for computation**
- **No direct system-to-system calls** across plugin boundaries
- **Resources for shared state**, not global variables
- **Each plugin owns its components**
- **Direct access allowed within plugins** for high-frequency operations
- **Utility modules** (math, data structures) can be directly imported anywhere

### Event Guidelines
- **Explicit Data Flow**: Every message visible in schedule with clear ordering
- **Lightweight Events**: Keep events small (≤128 bytes), Copy/Clone, no world references
- **One Event Per Concern**: Avoid kitchen-sink generic events requiring runtime casting
- **Documentation**: Each event group in dedicated module with clear purpose

### When to Use Events vs Direct Access
**ALWAYS USE EVENTS FOR:**
- **Cross-plugin boundaries** (architectural enforcement)
- **Entity lifecycle events** (spawn, despawn, state transitions)
- **One-to-many notifications** (damage → multiple UI/audio/effect handlers)
- **Decoupled game logic** (player input → multiple system responses)
- **Error propagation** between plugins

**USE DIRECT ACCESS FOR:**
- **Tight performance loops** (>1000 entities/frame)
- **Single-frame calculations** (transform updates, physics steps)
- **Shared resources** (configs, caches, lookup tables)
- **Utility functions** (math, validation, data structures)
- **Same-plugin high-frequency ops** (animation, movement within vehicle systems)

### Event Implementation Rules
- Events cleared every frame (O(n) performance)
- Multiple readers can consume same event concurrently
- Name systems after events: `handle_spawn_car_event`
- Use `.before()/.after()` for explicit system ordering
- Add debug instrumentation for event counts under debug-ui feature
- Keep stateless builder functions as helpers inside event handlers

### Event Naming Conventions
```rust
// Good: Specific, actionable events
pub struct VehicleEngineStarted { entity: Entity, engine_type: EngineType }
pub struct PlayerEnteredVehicle { player: Entity, vehicle: Entity }
pub struct WeaponFired { weapon: Entity, target: Option<Vec3> }

// Avoid: Generic, kitchen-sink events
pub struct GameEvent { event_type: String, data: HashMap<String, Value> }
```

## Modern ECS Patterns (Bevy 0.16+)
CORE PRINCIPLE: Leverage Bevy's latest ECS features for performance and maintainability.

### Component Design Best Practices
- **Immutable Components**: Use `#[component(immutable)]` for data that shouldn't change after spawn
- **Component Size**: Keep components under 64 bytes for cache efficiency
- **Data-Oriented Design**: Group related data together, avoid complex nested structures
- **Entity Relationships**: Use Bevy's relationship system instead of manual entity references

### Query Optimization
- **Specific Queries**: Use `With<T>` and `Without<T>` to minimize entity iteration
- **Query Filters**: Leverage `Changed<T>`, `Added<T>`, `AssetChanged<T>` for targeted updates
- **Entity Disabling**: Use `Disabled` component for inactive entities vs despawning

### Unified ECS Error Handling
- **System Results**: Return `Result<(), BevyError>` from systems instead of panicking
- **Error Propagation**: Bubble errors up rather than handling immediately
- **Global Handler**: Configure `GLOBAL_ERROR_HANDLER` for development vs production
- **Location Tracking**: Leverage Bevy's enhanced location tracking for debugging

### Entity Spawning Patterns
- **Spawn API**: Use `children!` and `related!` macros for hierarchical spawning
- **Entity Cloning**: Implement `#[derive(Clone)]` on components for entity duplication
- **Relationship Components**: Define bidirectional relationships with `Relationship` and `RelationshipTarget`

### Observer Pattern (New in 0.16)
```rust
// Better than global events for entity-specific logic
app.add_observer(on_vehicle_spawned);

fn on_vehicle_spawned(trigger: Trigger<OnAdd, VehicleComponent>) {
    // Automatic cleanup, validation, initialization
}
```

### Hybrid Event Approaches
- **Observers**: Use for entity-specific events instead of global events
- **Relationships**: Direct entity links for hierarchies (Parent/Child)
- **Query Filters**: `Changed<T>`, `AssetChanged<T>` for reactive updates
- **Commands**: Entity modification requests (better than events for spawning)

### Performance Considerations
- Events are O(n) cleared every frame
- Direct queries with `With<T>` filters are more cache-friendly
- Resource access has no per-frame overhead
- Observer pattern scales better than global events for entity-specific logic

## Performance Optimization (Bevy 0.16+)

### Key Optimizations
- GPU-driven rendering for complex scenes (3x+ performance gains)
- Faster transform propagation with dirty bit optimization
- Distance-based culling (buildings 300m, vehicles 150m, NPCs 100m)
- Asset processing with hot-reloading using `AssetChanged<T>` filters
- `MeshCache` resource for shared geometry

### Performance Targets
- 60+ FPS target with system timing intervals (road gen 0.5s, culling 0.5s)
- Distance caching with 5-frame cache, 2048 entry limit

## Commands
- Build: `cargo build` | Check: `cargo check` | Test: `cargo test test_name`
- Lint: `cargo clippy` | Format: `cargo fmt` | Run: `cargo run`
- Features: `cargo run --features debug-movement,debug-audio,debug-ui`

## Project Structure
- Bevy 0.16.1 game using Rust 2024 edition, bevy_rapier3d 0.30.0 physics
- Core dependencies: bevy 0.16.1, bevy_rapier3d 0.30.0, bytemuck 1.18, rand 0.8, serde 1.0
- Plugin-based: components/, systems/, plugins/, setup/, factories/
- **Bevy 0.16+ Features**: Entity relationships, observers, immutable components, unified error handling

### Architectural Boundaries
- **components/**: Pure data structures, no logic (includes relationships, immutable components)
- **systems/**: Pure functions that operate on components (return `Result<(), BevyError>`)
- **plugins/**: Self-contained modules with clear interfaces (use observers for entity lifecycle)
- **setup/**: One-time initialization, no ongoing state
- **factories/**: Entity creation patterns, stateless (use `children!`/`related!` macros)



## Code Style
- Follow Rust and Bevy best practices
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
- **Event-First**: Use events for coordination, direct access for computation
- **Direct Import Exceptions**: Core engine (Bevy systems), utilities (math, data structures), performance-critical intra-plugin code
- **Avoid circular dependencies** between modules
- **Prefer local imports** over glob imports
- **Keep external dependencies minimal**
- **One module per event group** for discoverability

## Testing Guidelines
- **Framework**: Rust built-in testing with Bevy test utilities
- **Pattern**: Use `App::new().add_plugins(MinimalPlugins)` for Bevy tests
- **Types**: Unit tests (inline modules), integration tests (tests/ dir), performance tests
- **Run**: `cargo test test_name` for specific tests, `cargo test` for all
- **Focus**: Component validation, LOD/culling behavior, physics integration, performance

### ECS Testing Patterns (Enhanced)
- **Component Testing**: Test component data validation and defaults
- **System Testing**: Use `App::new().add_plugins(MinimalPlugins)` for isolated system tests
- **Integration Testing**: Test plugin communication via events and shared resources
- **Performance Testing**: Include frame time and memory usage assertions
- **Observer Testing**: Test entity lifecycle hooks and component change reactions

### Test Organization
- **Unit tests**: Inline `#[cfg(test)]` modules in component/system files
- **Integration tests**: `tests/` directory for cross-plugin scenarios
- **Performance tests**: Separate feature flag `cargo test --features perf-tests`
- **Mock Components**: Create lightweight test doubles for expensive operations

## Debugging & Error Handling
- Clear error messages with context
- Use `expect()` with descriptive messages over `unwrap()`
- Simple debug tools: visual overlays, console output
- Fail fast and clearly, don't hide errors
- Debug features toggleable via features flags

### Development vs Production
- Use `#[cfg(debug_assertions)]` for debug-only code
- Configure `GLOBAL_ERROR_HANDLER` differently for dev/prod
- Feature flags for debug UI, wireframes, performance overlays
- Hot reloading enabled in development builds only

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



## Asset-Driven Control System
Primary control configuration using RON (Rusty Object Notation) files.

### System Architecture
- **Asset Configuration**: `assets/config/vehicle_controls.ron` - Single source of truth for ALL controls
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

### Kinematic Vehicle Physics (Option 2: Arcade Control)
- **Physics Model**: `RigidBody::KinematicVelocityBased` for all vehicles (cars, helicopters, F16)
- **Control Method**: Direct velocity manipulation without fighting physics solver
- **Collision**: Handled automatically by Rapier for kinematic bodies
- **Ground Detection**: Uses existing `GroundDetectionService` for spawning

### Vehicle Movement Systems
- **Car Movement**: `src/systems/movement/vehicles.rs` - Direct velocity control with asset-driven specs
- **Aircraft Movement**: `src/systems/movement/simple_aircraft.rs` - F16 and helicopter physics with RON configs

### Physics Configuration
- **Car Specs**: `assets/config/simple_car.ron` - Speed, rotation, emergency brake settings
- **Helicopter Specs**: `assets/config/simple_helicopter.ron` - Movement speeds and rotation rates  
- **F16 Specs**: `assets/config/simple_f16.ron` - Thrust, lift, and flight parameters

### Key Design Decisions
- **No Dynamic Bodies**: Eliminated Rapier solver conflicts and velocity panics
- **No Manual Ground Collision**: Removed `PhysicsUtilities::apply_ground_collision` calls
- **Direct Velocity Control**: Maintains arcade feel without physics solver interference
- **Collision Handling**: Relies on Rapier's kinematic collision detection

### Benefits
- **No Physics Panics**: Eliminated all Rapier velocity/solver conflicts  
- **Easy to Understand**: Linear velocity mapping instead of complex force calculations
- **Maintainable**: No advanced physics calculations requiring aerospace knowledge
- **Performant**: Fewer calculations per frame, no solver overhead
- **Reliable Landing**: F16 and helicopters can land without minimum height restrictions




