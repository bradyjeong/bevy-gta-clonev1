# Physics Optimization System

## Overview
Implemented proper physics optimization using Rapier's built-in features following professional game development practices.

## Key Principles (How Pro Games Do It)

### 1. **Automatic Sleep System** ✅
- Bodies automatically sleep when velocity < threshold for ~1 second
- Wake on collision/joint interaction automatically
- Zero CPU cost for sleeping bodies
- **No manual distance-based culling** (causes jolts)

### 2. **Fixed Timestep Decoupling** ✅
- Physics runs at 60Hz via `FixedUpdate` schedule
- Independent of render FPS
- Prevents jolts from frame rate changes

### 3. **Spatial Islands** ✅
- Rapier handles internally
- Disconnected groups process independently
- No manual management needed

## Implementation

### RapierConfiguration Setup
**Location**: `src/plugins/game_core.rs`

```rust
.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
.add_systems(
    Startup,
    |mut rapier_config: Query<&mut RapierConfiguration>| {
        if let Ok(mut config) = rapier_config.single_mut() {
            config.gravity = Vec3::new(0.0, -9.81, 0.0);
        }
    },
)
```

### Sleep Configuration by Entity Type

#### Vehicles (Cars, Helicopters, F16)
- **Sleep Mode**: `Sleeping::default()` - Enabled with Rapier defaults
- **Behavior**: Auto-sleep when parked → saves CPU
- **Wake**: Automatic on collision/interaction
- **Location**: `src/factories/vehicle_factory.rs`, `src/factories/generic_bundle.rs`

```rust
// Helicopter
Sleeping::default(),  // Will sleep when stationary

// F16
Sleeping::default(),  // Will sleep when landed

// Cars (via DynamicVehicleBundle)
sleeping: Sleeping::default(),
```

#### Player & NPCs
- **Sleep Mode**: `Sleeping::disabled()` - Always active
- **Behavior**: Never sleep → always responsive
- **Location**: `src/bundles.rs` (Player), `src/factories/npc_factory.rs` (NPCs)

```rust
// Player (in PlayerPhysicsBundle)
sleeping: Sleeping::disabled(),  // Always responsive

// NPCs (in spawn functions)
Sleeping::disabled(),  // Always responsive to player
```

## Benefits

### Performance
- Parked vehicles auto-sleep → no wasted CPU
- NPCs/Player never sleep → instant response
- Rapier's internal spatial islands optimize solver
- Fixed 60Hz physics prevents FPS-based performance variance

### Gameplay
- No pop-in/pop-out physics behavior
- No jolts from culling transitions
- Collisions always work correctly
- Vehicles naturally "settle" when parked

### Maintenance
- No manual distance culling systems to maintain
- Rapier handles all optimization internally
- Clean, simple configuration
- Follows professional game engine patterns

## Technical Details

### Why NOT Distance-Based Physics Culling?

**Problems with Distance Culling:**
- Bodies pop in/out of simulation → visual jolts
- Collision state lost when culled
- Complex re-initialization when un-culled
- Performance overhead from culling checks

**Rapier's Automatic Sleep:**
- Smooth transition to sleep (no pop)
- Maintains all collision state
- Perfect wake-up on interaction
- Zero overhead when sleeping

### System Ordering
Physics systems run in proper order (from `game_core.rs`):

1. **FixedUpdate (Before Physics)**:
   - Movement systems apply velocities
   - Systems run `.before(PhysicsSet::SyncBackend)`

2. **Rapier Physics Step**:
   - Solver processes forces/collisions
   - Sleep states updated automatically

3. **FixedUpdate (After Physics)**:
   - Safeguards clean up edge cases
   - Systems run `.after(PhysicsSet::Writeback)`

## Comparison: Before vs After

### Before (Problematic)
- ❌ No sleep configuration
- ❌ All bodies always active
- ❌ Wasted CPU on stationary objects
- ❌ Player had `Sleeping::disabled()` correctly, but vehicles didn't use sleep

### After (Optimized)
- ✅ Vehicles use `Sleeping::default()` (auto-sleep enabled)
- ✅ Player/NPCs use `Sleeping::disabled()` (always responsive)
- ✅ Rapier gravity configured explicitly
- ✅ Fixed 60Hz timestep maintained
- ✅ Zero manual culling - all automatic

## References
- Rapier Physics Documentation: https://rapier.rs/
- Bevy Rapier Plugin: https://docs.rs/bevy_rapier3d/
- GTA-style physics: Automatic sleep for props, manual control for gameplay entities
