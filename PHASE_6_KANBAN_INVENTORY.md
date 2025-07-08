# Phase 6: Decommission Safety Nets - Kanban Inventory

## LEGACY_API Dependencies
- [x] `game_bin/Cargo.toml:27` - Remove `features = ["legacy_api"]` from gameplay_sim dependency
- [x] `gameplay_sim/Cargo.toml:8` - Remove `legacy_api = []` feature definition

## Legacy API Imports to Migrate
- [x] `game_bin/src/lib.rs:15` - `pub use gameplay_sim::compat::{config, factories, services};`
- [x] `game_bin/src/systems/mod.rs:3` - `pub use gameplay_sim::compat::*;`
- [x] `game_bin/src/plugins/mod.rs:2` - `pub use gameplay_sim::compat::plugins::*;`

## Compat Layer Stub Functions to Replace
- [x] `new_unified_distance_culling_system()` - Create real implementation
- [x] `new_unified_lod_system()` - Create real implementation  
- [x] `new_unified_batch_spawning_system()` - Create real implementation
- [x] `new_unified_cleanup_system()` - Create real implementation
- [x] `new_unified_performance_monitoring_system()` - Create real implementation
- [x] `new_unified_cache_system()` - Create real implementation
- [x] `new_unified_vegetation_system()` - Create real implementation
- [x] `new_unified_traffic_system()` - Create real implementation
- [x] `new_unified_npc_system()` - Create real implementation
- [x] `new_unified_audio_system()` - Create real implementation
- [x] `new_unified_ui_system()` - Create real implementation
- [x] `new_unified_save_system()` - Create real implementation
- [x] `new_unified_control_system()` - Create real implementation
- [x] `new_unified_camera_system()` - Create real implementation

## Warning Suppressions to Remove
- [x] `game_bin/src/lib.rs:3` - Remove `#![cfg_attr(not(test), allow(dead_code, unused_imports, unused_variables, unused_mut))]`
- [x] `game_bin/src/lib.rs:7` - Remove extensive allow list
- [x] `gameplay_render/src/lib.rs:134` - Remove `#![cfg_attr(not(test), allow(unexpected_cfgs))]`
- [x] `gameplay_sim/src/lib.rs:7` - Remove extensive allow list
- [x] `gameplay_ui/src/lib.rs:6` - Remove extensive allow list

## Disabled Artifacts to Re-enable
- [x] `gameplay_render/Cargo.toml` - Change `test = false` to `test = true`
- [x] `Cargo.toml` - Re-add `gameplay_ui` to workspace members
- [x] `game_bin/Cargo.toml` - Re-enable gameplay_ui dependency
- [x] `game_bin/src/plugins/game_plugin.rs` - Re-enable GameplayUIPlugin

## Manifest Hygiene
- [x] Remove unused manifest key warning
- [x] Add `[workspace.metadata]` documentation section

## Module Re-exports to Formalize
- [x] `config` module - Move from compat to proper public API
- [x] `factories` module - Move from compat to proper public API  
- [x] `services` module - Move from compat to proper public API
- [x] `plugins` module - Move from compat to proper public API

## Status Summary
- **Total Items**: 32
- **Completed**: 0
- **In Progress**: 0
- **Todo**: 32
