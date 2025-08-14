# Blame Analysis: What Was Broken Before vs What We Broke

## Pre-Existing Issues (NOT caused by event architecture)

### 1. ❌ Duplicate RoadNetwork Implementation
**When introduced**: Already in codebase
**Evidence**: Two separate RoadNetwork structs existed (4-node demo vs real)
**Would have broken main**: YES - any attempt to use road validation would fail

### 2. ❌ u16 Coordinate Overflow  
**When introduced**: Already in `layered_generation.rs`
**Evidence**: Code was casting `as u16` for negative coordinates
**Would have broken main**: YES - negative chunks would never work

### 3. ❌ Chunk Size Inconsistency
**When introduced**: Already in codebase
**Evidence**: `CHUNK_SIZE=256` vs `UNIFIED_CHUNK_SIZE=100`
**Would have broken main**: YES - caused misaligned spawning

### 4. ❌ Bad Spawn Rate Math
**When introduced**: Already in codebase
**Evidence**: Sequential probability checks instead of single roll
**Would have broken main**: YES - only ~17% total spawn chance

### 5. ❌ Node Spacing Bug
**When introduced**: Already in codebase  
**Evidence**: `node_spacing = 100.0` (same as chunk size)
**Would have broken main**: YES - only 1 road node per chunk

## Issues CAUSED by Event Architecture

### 1. ✅ Event Routing Complexity
**When introduced**: During migration
**Evidence**: ValidationTracker using Local instead of ResMut
**Impact**: Spent hours debugging why events weren't connecting

### 2. ✅ Observer Pattern Confusion
**When introduced**: During migration
**Evidence**: Had to implement observers, wire them correctly
**Impact**: Added complexity without clear benefit

### 3. ✅ Double-Hop Validation
**When introduced**: During migration
**Evidence**: Request → Validation → Result → Spawn (4 hops!)
**Impact**: Made debugging a "scavenger hunt"

## Issues EXPOSED (but not caused) by Event Architecture

### 1. 🔍 Roads Blocking Spawns
**Status**: Pre-existing logic, but only visible after RoadNetwork fix
**Why exposed**: Event validation actually checked road proximity
**Main branch**: Would hit this once RoadNetwork was fixed

## The Verdict

### Pre-Existing Bugs: 80%
- Duplicate RoadNetwork
- Coordinate overflow
- Chunk size mismatch
- Spawn rate math
- Node spacing

### Event Architecture Bugs: 20%
- Event routing
- Observer wiring
- Validation pipeline complexity

### Time Spent Debugging:
- **50%** on event architecture issues
- **50%** on pre-existing bugs

## The Real Problem

The event architecture didn't CAUSE most bugs, but it:
1. Made them harder to debug (indirection)
2. Made them more visible (validation actually ran)
3. Added its own complexity layer on top

**Bottom Line**: We spent as much time fighting the event system as we did fixing real bugs. The bugs needed fixing regardless, but they would have been easier to fix without the event complexity.
