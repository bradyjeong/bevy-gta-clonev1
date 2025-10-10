# Aggressive Arcade Physics Tuning

## FPS Question - Answered ✅

**Q: Why is FPS so high?**
**A: `cargo run --release` enables all Rust optimizations!**

- Debug build: ~20-40 FPS (slow, for development)
- Release build: 120+ FPS (optimized, for gameplay)
- M4 Max + optimized physics = extremely fast
- VSync is already enabled (will cap to display refresh rate)

## New Controls ✅

### Car Controls (Updated)
- **Arrow Up**: Accelerate (forward only)
- **Arrow Down**: Reverse (backward only)  
- **Arrow Left/Right**: Steer
- **Shift**: Regular Brake (slow down smoothly)
- **Space**: Emergency Brake / Drift (sharp slides!)
- **F**: Enter/Exit Vehicle
- **F3**: Debug Overlay

**Old System**: Arrow Down was brake AND reverse (confusing)
**New System**: 
- Shift = Regular brake (slows you down)
- Space = E-brake for drifting!

## Aggressive Tuning Applied ✅

### Changes Made:
```diff
- steer_gain: 2.5         → 4.5    (80% more responsive!)
- steer_speed_drop: 0.03  → 0.02   (Better high-speed turning)
- stability: 0.8          → 0.6    (Less auto-straighten = sharper)
- drift_grip: 2.5         → 1.8    (28% easier to drift)
- ebrake_yaw_boost: 0.6   → 1.2    (100% more drift rotation!)
```

### What You'll Feel:
1. **Much Sharper Turns**: Car responds 80% faster to steering input
2. **High-Speed Agility**: Less steering penalty at speed
3. **Easier Drifting**: Lower grip when e-braking
4. **Dramatic Slides**: Double the yaw rotation during drifts
5. **Less Stability Assist**: Car won't auto-correct as much

## Files Changed:
- ✅ `assets/config/vehicle_controls.ron` - Space is now e-brake
- ✅ `assets/config/simple_car.ron` - Aggressive physics values
- ✅ `docs/ARCADE_CAR_PHYSICS.md` - Updated documentation

## Testing Notes:

**Try This:**
1. Get in a car (F key near vehicle)
2. Accelerate to high speed (Arrow Up)
3. Turn sharply (Arrow Left/Right) - Notice MUCH faster response!
4. Hit Space while turning - Dramatic drift!
5. Release Space - Car grips and exits drift

**Expected Behavior:**
- Steering feels "snappy" and responsive
- High speeds: car still turns well (was sluggish before)
- E-brake: instant drift with big yaw rotation
- Drifts are easier to initiate and control

## Revert Instructions:

If too aggressive, restore original values in `simple_car.ron`:
```ron
steer_gain: 2.5
steer_speed_drop: 0.03
stability: 0.8
drift_grip: 2.5
ebrake_yaw_boost: 0.6
```

## Performance Impact:
**Zero** - These are just config values, no code changes, same performance!
