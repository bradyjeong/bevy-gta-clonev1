# Architectural Fix: Road Validation Cross-Plugin Boundaries

## Summary
Fixed architectural violations where `is_on_road_spline` was being called directly across plugin boundaries, violating AGENT.md principles.

## Key Principle (from AGENT.md)
- **Cross-Plugin Communication**: Use events for coordination, direct access for computation
- **No direct system-to-system calls** across plugin boundaries
- **Utility modules** (math, data structures) can be directly imported anywhere

## Changes Made

### 1. Legacy Systems (Not Active)
**Files**: `map_system.rs`, `infinite_streaming.rs`
- These are legacy systems not registered in any plugin
- Changed from `is_on_road_spline()` to `road_network.is_near_road()`
- Added comments explaining they are legacy and documenting the architectural violation

### 2. SpawnValidationPlugin
**File**: `spawn_validation_plugin.rs`
- This is a separate plugin from UnifiedWorldPlugin (where road_generation lives)
- Changed from `is_on_road_spline()` to `road_network.is_near_road()`
- Added comments explaining to use events for cross-plugin coordination

### 3. Factory Modules (Same Plugin - OK)
**Files**: `position_validator.rs`, `entity_factory_unified.rs`, `spawn_utils.rs`
- These are part of UnifiedWorldPlugin (same as road_generation)
- **Direct calls are allowed** within the same plugin per AGENT.md
- Added clarifying comments that direct import is allowed

### 4. Event Handler (Same Plugin - OK)
**File**: `spawn_validation_handler.rs`
- Part of the world/road plugin system
- Already uses `is_on_road_spline` correctly for intra-plugin use
- No changes needed

## Event-Based System (Already Implemented)
The codebase already has a proper event-based validation system:
- `RequestRoadValidation` - Request road checking
- `RoadValidationResult` - Response with validation result
- `ValidationId` - Correlation ID to match requests/responses
- Handler systems process these events with proper ordering

## Architectural Compliance
✅ **Within Plugin**: Direct calls allowed (factories, handlers)
✅ **Cross Plugin**: Must use events (fixed in spawn_validation_plugin)
✅ **Shared Resources**: Can access directly (RoadNetwork)
✅ **Legacy Code**: Documented violations with migration notes

## Migration Path for Active Systems
If legacy systems become active again:
1. Replace synchronous `road_network.is_near_road()` calls with:
   - Send `RequestRoadValidation` event
   - Store correlation ID
   - Handle `RoadValidationResult` in separate system
2. Use system ordering to ensure validation completes before spawn
3. See `spawn_validation_handler.rs` for reference implementation
