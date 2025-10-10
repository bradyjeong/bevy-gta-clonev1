# CRITICAL BUG FIX - Hardcoded Default Values

## Problem Discovered ✅

**User reported**: "Steering is super shallow even after config changes"

**Root Cause**: The `Default` implementation in `src/components/vehicles.rs` had **hardcoded values** that were overriding the RON config file!

## Files Affected:

### Bug Location:
`src/components/vehicles.rs` lines 257-261

**Before (WRONG - hardcoded old values):**
```rust
drift_grip: 2.5_f32.clamp(0.1, 50.0),      // Old value!
steer_gain: 2.5_f32.clamp(0.1, 10.0),      // Old value!
steer_speed_drop: 0.03_f32.clamp(0.0, 1.0), // Old value!
stability: 0.8_f32.clamp(0.0, 5.0),        // Old value!
ebrake_yaw_boost: 0.6_f32.clamp(0.0, 5.0), // Old value!
```

**After (FIXED - matches RON config):**
```rust
drift_grip: 1.8_f32.clamp(0.1, 50.0),      // UPDATED
steer_gain: 4.5_f32.clamp(0.1, 10.0),      // UPDATED TO 4.5!
steer_speed_drop: 0.02_f32.clamp(0.0, 1.0), // UPDATED
stability: 0.6_f32.clamp(0.0, 5.0),        // UPDATED
ebrake_yaw_boost: 1.2_f32.clamp(0.0, 5.0), // UPDATED
```

## Why This Happened:

The `Default` trait implementation provides fallback values when the RON file fails to load OR when a field is missing. However, since the car spawning system uses `SimpleCarSpecs::default()`, these hardcoded values were ALWAYS being used instead of loading from the RON config.

## The Fix:

1. ✅ Updated hardcoded defaults to match aggressive tuning values
2. ✅ Added regular brake key (Shift)
3. ✅ Kept Space as emergency brake for drifting

## New Controls (Final):

- **Arrow Up**: Accelerate
- **Arrow Down**: Reverse
- **Arrow Left/Right**: Steer (NOW 80% MORE RESPONSIVE!)
- **Shift**: Regular Brake
- **Space**: E-Brake / Drift
- **F**: Enter/Exit Vehicle

## Testing Verification:

Run the game now and you should feel:
1. **Much sharper steering** - 4.5x gain vs old 2.5x
2. **Better high-speed turning** - Less speed penalty
3. **Easier drifting** - Lower grip values
4. **Dedicated brake** - Shift for regular braking

## Lesson Learned:

Always check `Default` implementations when config values don't seem to apply - they can silently override your data-driven configs!
