# Factory-World Interaction Architecture Audit

## Executive Summary
The factory system has several violations of the event-driven architecture rules specified in AGENT.md. While factories are allowed direct resource access for computation, several factories are performing **coordination and scheduling decisions** that must use events.

## Critical Violations Found

### 1. EntityLimitManager - Scheduling Decision Violation ❌
**File:** `src/factories/entity_limit.rs`
**Issue:** Makes autonomous despawn decisions based on capacity limits
```rust
// Line 47-49: Direct despawn without event coordination
if let Some((oldest_entity, _)) = self.building_entities.first().copied() {
    commands.entity(oldest_entity).despawn();
    self.building_entities.remove(0);
}
```
**Violation:** This is a **scheduling decision** (when to despawn entities) and **cross-plugin coordination** (affecting entity lifecycle)
**Required Fix:** Emit `RequestEntityDespawn` event and let a dedicated system handle the despawn

### 2. UnifiedEntityFactory - Cross-Plugin Resource Access ⚠️
**File:** `src/factories/entity_factory_unified.rs`
**Issue:** Takes `Option<&RoadNetwork>` which appears to be cross-plugin access
**Analysis:** 
- Both `UnifiedEntityFactory` and `RoadNetwork` are owned by `UnifiedWorldPlugin`
- This is **same-plugin access** and therefore ALLOWED per AGENT.md
- However, the pattern is fragile if plugin boundaries change
**Status:** Currently compliant but architecturally fragile

### 3. MaterialFactory - Global Resource Mutation ⚠️
**File:** `src/factories/material_factory.rs`
**Issue:** Directly modifies `Assets<StandardMaterial>` and inserts itself as a resource
**Analysis:**
- Asset modification is a **utility function** (computation)
- Resource insertion happens during initialization only
**Status:** Compliant - utility functions are allowed direct access

### 4. content_spawn_handler - Event Emission ✅
**File:** `src/systems/world/event_handlers/content_spawn_handler.rs`
**Issue:** Emits `DynamicContentSpawned` events after factory spawning
**Analysis:** This is correct event-driven coordination
**Status:** Compliant

## Architectural Boundaries

### UnifiedWorldPlugin Owns:
- `UnifiedEntityFactory`
- `RoadNetwork`  
- `ChunkTracker`, `ChunkTables`, `PlacementGrid`, `WorldCoordinator`

Since both factory and road network are in the same plugin, the current access pattern is technically allowed. However, this creates tight coupling.

## Required Fixes

### Priority 1: EntityLimitManager Event Refactor
```rust
// CURRENT (VIOLATION)
pub fn enforce_limit(&mut self, commands: &mut Commands, content_type: ContentType, entity: Entity, timestamp: f32) {
    if self.building_entities.len() >= self.max_buildings {
        if let Some((oldest_entity, _)) = self.building_entities.first().copied() {
            commands.entity(oldest_entity).despawn(); // ❌ Direct despawn
        }
    }
}

// PROPOSED (EVENT-DRIVEN)
pub fn enforce_limit(&mut self, event_writer: &mut EventWriter<RequestEntityDespawn>, content_type: ContentType, entity: Entity, timestamp: f32) -> Option<Entity> {
    if self.building_entities.len() >= self.max_buildings {
        if let Some((oldest_entity, _)) = self.building_entities.first().copied() {
            event_writer.send(RequestEntityDespawn {
                entity: oldest_entity,
                reason: DespawnReason::CapacityLimit(content_type),
            });
            self.building_entities.remove(0);
            return Some(oldest_entity);
        }
    }
    self.building_entities.push((entity, timestamp));
    None
}
```

### Priority 2: Document Plugin Boundaries
Add to AGENT.md:
```markdown
## Plugin Ownership Matrix
- UnifiedWorldPlugin:
  - Resources: UnifiedEntityFactory, RoadNetwork, ChunkTracker, etc.
  - Internal access allowed between these resources
- Factories should ONLY:
  - Create entities (computation)
  - Read resources for validation (computation)
  - NOT make scheduling/lifecycle decisions
```

## Legitimate Direct Access Patterns Found

### Allowed per AGENT.md:
1. **Validation Reads**: Factories reading RoadNetwork for position validation (computation)
2. **Asset Creation**: MaterialFactory creating materials (utility function)
3. **Cache Access**: GroundHeightCache shared between factories (performance optimization)
4. **Config Reading**: GameConfig resource access (read-only shared state)

## Recommendations

1. **Immediate:** Fix EntityLimitManager to use events for despawn coordination
2. **Short-term:** Create explicit `EntityLifecycleEvents` module for spawn/despawn coordination
3. **Medium-term:** Consider splitting UnifiedWorldPlugin if it becomes too large
4. **Long-term:** Add compile-time plugin boundary enforcement via module visibility

## Summary

The factory system is mostly compliant with AGENT.md's event-driven architecture, with one critical violation in EntityLimitManager making autonomous despawn decisions. The UnifiedEntityFactory's use of RoadNetwork is technically allowed since both are in the same plugin, but this should be documented to prevent future violations if plugin boundaries change.
