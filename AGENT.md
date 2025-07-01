# AGENT.md

## Commands
- Build: `cargo build` | Check: `cargo check` | Test: `cargo test test_name`
- Lint: `cargo clippy` | Format: `cargo fmt` | Run: `cargo run`
- Features: `cargo run --features debug-movement,debug-audio`

## Project Structure
- Bevy 0.16.1 game using Rust 2024 edition, bevy_rapier3d physics
- Plugin-based: components/, systems/, plugins/, setup/, factories/

## Features
- `debug-movement`, `debug-audio`, `debug-ui`

## Code Style
- snake_case for variables/functions, PascalCase for structs/components
- Import order: external crates, std, bevy prelude, local crate
- Prefer `if let`/`match` over unwrap(), use Bevy constants (Vec3::ZERO)
- Components: `#[derive(Component)]` + `Default`, systems in subdirs
- Safety: Validate physics values, clamp positions/dimensions, use collision groups
- Comments: `//` style, 4-space indent, trailing commas

## Performance
- Target 60+ FPS, entity culling (buildings 300m, vehicles 150m, NPCs 100m)
- System timing intervals: road gen 0.5s, dynamic content 2.0s, culling 0.5s
- Ultra-reduced spawn rates: buildings 8%, vehicles 4%, trees 5%, NPCs 1%
- Distance caching: Avoids repeated calculations (5-frame cache, 2048 entry limit)
- Cache debug: Press F3 or wait 5s for cache performance stats
