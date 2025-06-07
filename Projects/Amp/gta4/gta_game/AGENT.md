# AGENT.md

## Commands
- Build: `cargo build`
- Check (fast compile): `cargo check` 
- Test all: `cargo test` (no tests currently exist)
- Test single: `cargo test test_name`
- Lint: `cargo clippy`
- Format: `cargo fmt`
- Run: `cargo run`
- Run with features: `cargo run --features weather,debug-weather`

## Project Structure
- Bevy 0.16.1 game using Rust 2024 edition
- Dependencies: bevy, bevy_rapier3d, rand
- Plugin-based architecture with modular organization:
  - **components/**: Player, Vehicles, World, Weather, Effects, Water
  - **systems/**: Movement, Camera, Interaction, Effects, Weather, World, Audio, UI, Sky, Vehicles
  - **plugins/**: PlayerPlugin, VehiclePlugin, WorldPlugin, UIPlugin, WeatherPlugin
  - **setup/**: World, Vehicles, Environment, Weather initialization

## Features
- `weather`: Weather system functionality
- `debug-weather`: Weather debugging (includes weather)
- `debug-movement`: Movement system debugging
- `debug-audio`: Audio system debugging
- `debug-ui`: UI system debugging

## Performance Profile
- **Target FPS**: 60+ FPS (achieved: 42-80 FPS range)
- **Entity count**: ~875-900 entities (optimized from 1300+)
- **Frame time**: 12.5-35ms (target: <16.7ms for 60 FPS)
- **Memory usage**: Optimized through aggressive culling

## Performance Optimizations Applied
1. **System Timing**: Road generation (0.5s intervals), dynamic content (2.0s intervals), culling (0.5s intervals)
2. **Entity Culling**: Buildings (300m), vehicles (150m), trees (200m), NPCs (100m)
3. **Spawn Rates**: Ultra-reduced - Buildings (8%), vehicles (4%), trees (5%), NPCs (1%)
4. **Chunk Sizes**: Roads (400m chunks), dynamic content (300m radius, 80m density)
5. **Physics**: Default Rapier configuration with VSync enabled
6. **Debug Spam**: Eliminated road generation debug messages that were causing frame drops

## Code Style & Conventions
- Use snake_case for variables, functions, modules
- Use PascalCase for structs, enums, traits, components  
- Import order: external crates, std, bevy prelude, local crate
- Use wildcard imports for prelude modules, specific for individual items
- Prefer `if let` and `match` for error handling over unwrap()
- Use `Vec3::ZERO`, `Transform::from_xyz()`, `Quat::IDENTITY`
- Components use `#[derive(Component)]` and implement `Default`
- Systems organized in subdirectories by functionality
- Comments: use `//` style with descriptive inline comments
- 4-space indentation, trailing commas in multi-line expressions
