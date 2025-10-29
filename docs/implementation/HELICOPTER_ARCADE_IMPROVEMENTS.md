# Helicopter Arcade Improvements - Implementation Summary

## Overview
Simplified helicopter controls by replacing complex PD controller with F16-style direct angular velocity control + GTA San Andreas-inspired per-axis stabilization.

## Research Phase

### Librarian Findings
Discovered **[gta-reversed](https://github.com/gta-reversed/gta-reversed)** - complete reverse-engineered GTA San Andreas source code with full helicopter implementation:
- `tFlyingHandlingData` - 21 flight physics parameters
- Per-axis stabilization (`YawStab`, `RollStab`, `PitchStab`)
- Directional resistance vectors (`turn_res`, `speed_res`)
- Direct torque control with strong damping (NOT PD controllers)

### Key GTA SA Insights
1. **Per-axis stabilization** - Independent damping for pitch/roll/yaw
2. **Direct control mapping** - What you input is what you get
3. **Multiplicative damping** - Applied only when inputs are neutral
4. **Mission vehicle perks** - Wind immunity for playability
5. **Simple architecture** - No complex PD loops or attitude targets

## Implementation Changes

### 1. Removed Unnecessary Code (63 lines, 60% reduction)

**Deleted from SimpleHelicopterSpecs (12 fields):**
- `lateral_speed`, `forward_speed` - Not used in force-based system
- `linear_lerp_factor`, `drag_factor` - Not used
- `angular_stab` - Replaced with per-axis stab
- `attitude_rate_deg_s`, `attitude_kp`, `attitude_kd`, `yaw_rate_kp` - PD controller removed
- `pitch_torque`, `roll_torque`, `yaw_torque` - Using velocity control

**Deleted from HelicopterRuntime (4 fields):**
- `target_pitch_deg`, `target_roll_deg` - No attitude targets
- `heading_hold_yaw`, `prev_yaw_active` - No heading hold

**Deleted from simple_helicopter_movement (105 lines → 42 lines):**
- 5 helper functions: `apply_deadzone`, `move_towards`, `shortest_angle_deg`, `extract_pitch_roll_deg`, `extract_yaw_deg`
- Complex attitude target calculations
- PD torque computation
- Heading-hold state machine

### 2. Added GTA SA Features

**New per-axis stabilization fields:**
```ron
pitch_stab: 0.97,  // Moderate auto-leveling
roll_stab: 0.96,   // Slightly faster than pitch
yaw_stab: 0.98,    // Slower, allows drift but prevents spin
```

**Control pattern (matches F16):**
```rust
// Direct angular velocity control
let local_target_ang = Vec3::new(pitch_cmd, yaw_cmd, roll_cmd) * rpm_eff * dmg_scale;
velocity.angvel = safe_lerp(velocity.angvel, world_target_ang, dt * angular_lerp_factor);

// Per-axis multiplicative damping (GTA SA style)
let pf = if pitch_input_abs < dz { pitch_stab.powf(dt) } else { 1.0 };
let rf = if roll_input_abs < dz { roll_stab.powf(dt) } else { 1.0 };
let yf = if yaw_input_abs < dz { yaw_stab.powf(dt) } else { 1.0 };
```

### 3. Bug Fixes (Oracle Recommendations)

**Hover-hold deadzone gate:**
- Before: `vertical.abs() <= 0.001` (magic number, chatter risk)
- After: `vertical_input_abs < dz` (consistent with other axes)

**RON config cleanup:**
- Removed 12 unused parameter lines
- Kept only fields that exist in SimpleHelicopterSpecs
- Added clear comments for each section

## Technical Details

### Physics Model
**Kept (force-based lift):**
- RPM-based lift calculation with spool-up/down
- Cyclic tilt for directional control
- Collective for vertical movement
- Horizontal drag force
- Damage scaling

**Changed (attitude control):**
- ~~PD controller with torques~~ → Direct angular velocity control
- ~~Attitude targets + heading hold~~ → Simple per-axis damping
- ~~Complex state machine~~ → Stateless deadzone + lerp

### Control Flow
```
Input → Deadzone → Angular Rate Command → Scale by RPM/Damage 
→ Lerp Velocity → Per-Axis Damping → Physics Integration
```

### Benefits
1. **Simpler**: 60% less code, no PD tuning needed
2. **Consistent**: Same pattern as F16 (proven arcade feel)
3. **Maintainable**: Stateless, data-driven, clear flow
4. **GTA-inspired**: Per-axis stabilization for arcade playability
5. **Physics-friendly**: Force-based lift preserves helicopter feel

## Configuration

### Current Parameters
```ron
// Rotation rates
yaw_rate: 1.8,    pitch_rate: 1.0,    roll_rate: 1.0

// Smoothing
angular_lerp_factor: 4.0

// Stabilization (GTA SA style)
pitch_stab: 0.97,   roll_stab: 0.96,   yaw_stab: 0.98

// Lift & drag
hover_bias: 0.03,   collective_gain: 0.55,   horiz_drag: 0.12

// RPM dynamics
spool_up_rate: 0.70,   spool_down_rate: 0.30,   min_rpm_for_lift: 0.30
```

### Tuning Recommendations (from Oracle)
- **Snappier roll**: Lower `roll_stab` to 0.94
- **Faster yaw**: Raise `yaw_rate` to 2.0
- **Quicker response**: Raise `angular_lerp_factor` to 6.0
- **Reduce slide**: Raise `horiz_drag` to 0.3-0.6

## Testing Results
✅ `cargo check` - Clean compilation
✅ `cargo clippy -- -D warnings` - No warnings
✅ `cargo test` - All 27 tests pass

## Known Behavior Differences

### From PD Controller Implementation
- **No heading hold**: Yaw damping instead of locked heading
- **No attitude hold**: Helicopters may hold bank if released while tilted
- **Simpler hover**: Altitude hold works, but horizontal drift possible from bank

### If Needed: Optional Enhancements
1. **Additive horizon leveling** - Small auto-level when inputs neutral
2. **Thrust falloff** - Reduce lift at high horizontal speeds
3. **Ground effect** - Increase lift near ground

## Files Modified
1. `src/components/vehicles.rs` - Removed/added spec fields, cleaned runtime
2. `src/systems/movement/simple_aircraft.rs` - Replaced PD with direct control
3. `assets/config/simple_helicopter.ron` - Cleaned config file

## Next Steps
1. **Playtest** helicopter controls for arcade feel
2. **Monitor** for persistent bank/drift issues
3. **Consider** additive horizon leveling if needed
4. **Tune** per-axis stab values based on feedback

---

**Result**: Helicopter controls now match F16's proven arcade pattern while incorporating GTA SA's per-axis stabilization wisdom. 60% code reduction with cleaner, more maintainable architecture.
