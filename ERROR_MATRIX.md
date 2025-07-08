# Error Matrix - Compilation Error Classification

## Total Errors: 18

### Category A: Dependency / Version / Feature Mismatches
**Count: 0**
- No external dependency version conflicts found
- All crates use Bevy 0.16.1 consistently

### Category B: Public API Drift Between Internal Crates
**Count: 4**
- `gameplay_ui/src/debug/distance_cache_debug.rs:2` - `use crate::systems::distance_cache::DistanceCache;` (unresolved import)
- `game_bin/src/main.rs` - `gameplay_ui::input` module is private
- Several modules referencing private interfaces

### Category C: Purely Local Code Defects
**Count: 14**
- `gameplay_render` - Various missing imports in test files
- `gameplay_ui` - Missing module declarations and imports 
- `game_core` - Missing components, game_state, config modules
- Typos, missing use statements, wrong paths

### Category D: TODO-Stub (Explicit stubs)
**Count: 0**
- No `todo!()` or `unimplemented!()` macros causing compilation failures

### Category E: FFI / Platform Specific Build Failures
**Count: 0**
- No platform-specific compilation issues

## Resolution Priority Order:
1. **Phase 2:** Resolve Category B (API drift) - 4 errors
2. **Phase 4:** Resolve Category C (local defects) per crate - 14 errors

## Per-Crate Breakdown:
- `gameplay_render`: ~6 errors (missing imports in tests)
- `gameplay_ui`: ~8 errors (missing modules, private interfaces)  
- `game_core`: ~3 errors (missing module references)
- `game_bin`: ~1 error (private module access)
