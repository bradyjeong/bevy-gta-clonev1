# Arcade Car Physics Implementation

## Overview
Professional-quality arcade racing physics for GTA-style handling with drift, traction loss, and realistic game feel.

## Controls (Default)
- **Arrow Up**: Accelerate
- **Arrow Down**: Reverse
- **Arrow Left/Right**: Steer
- **Shift**: Regular Brake (slow down)
- **Space**: Emergency Brake / Drift
- **F**: Enter/Exit Vehicle
- **F3**: Debug Overlay

## Critical Fixes Applied

### 1. **Coordinate System Correction** ✅
- **Issue**: Code was treating +Z as forward, but Bevy uses -Z as forward direction
- **Fix**: 
  - Changed `conjugate()` to `inverse()` for robust quaternion operations
  - Inverted Z-axis calculations: `current_speed = (-v_local.z).abs()`
  - Corrected acceleration/braking signs to match Bevy's coordinate system
- **Impact**: Cars now accelerate forward and brake properly

### 2. **Local-Space Velocity Control** ✅
- **What**: Separates forward/lateral motion for realistic physics
- **How**: Converts world velocity to car local space, applies physics, converts back
- **Benefit**: Enables proper drifting, sliding, and traction loss

### 3. **Speed-Based Steering** ✅
- **Formula**: `steer_gain / (1.0 + steer_speed_drop * current_speed)`
- **Prevents**: Instant 180° turns at high speed
- **Result**: GTA V-style handling that feels realistic

### 4. **Lateral Grip System** ✅
- **Normal Grip**: `8.0` - Strong traction during normal driving
- **Drift Grip**: `2.5` - Reduced traction when e-braking
- **Downforce**: Increases grip at high speeds (`0.3` scale factor)
- **Application**: `v_local.x = safe_lerp_f32(v_local.x, 0.0, dt * effective_grip)`

### 5. **Stability & Auto-Straightening** ✅
- **Term**: `-v_local.x * specs.stability`
- **Function**: Prevents spin-outs by aligning car with velocity
- **Tunable**: `stability: 0.8` in config

### 6. **E-Brake Drifting** ✅
- **Grip Reduction**: Switches from `grip` to `drift_grip`
- **Yaw Boost**: `control_state.steering.signum() * ebrake_yaw_boost`
- **Result**: Controlled powerslides and drifts

## New Configuration Parameters

### Extended `SimpleCarSpecs`
```rust
// Acceleration/Braking
accel_lerp: 5.0           // Acceleration smoothing rate
brake_lerp: 6.0           // Braking smoothing rate

// Grip & Traction
grip: 8.0                 // Normal lateral grip (higher = sticks better)
drift_grip: 1.8           // Grip during e-brake (lower = drifts more) - AGGRESSIVE
downforce_scale: 0.3      // High-speed grip boost

// Steering (AGGRESSIVE TUNING)
steer_gain: 4.5           // Base steering responsiveness - INCREASED
steer_speed_drop: 0.02    // Speed-based steering reduction - REDUCED for agility

// Stability (RESPONSIVE TUNING)
stability: 0.6            // Auto-straightening torque - REDUCED for sharper feel
ebrake_yaw_boost: 1.2     // Extra yaw when drifting - INCREASED for sharp drifts
```

### RON Configuration (`assets/config/simple_car.ron`)
```ron
SimpleCarConfig(
    // ... existing fields ...
    
    // Arcade physics tuning (AGGRESSIVE PRESET)
    accel_lerp: 5.0,
    brake_lerp: 6.0,
    grip: 8.0,
    drift_grip: 1.8,          // Lower for easier drifts
    steer_gain: 4.5,          // Higher for aggressive turns
    steer_speed_drop: 0.02,   // Less speed penalty
    stability: 0.6,           // Lower for sharper response
    ebrake_yaw_boost: 1.2,    // Higher for dramatic drifts
    downforce_scale: 0.3,
)
```

## Physics Configuration Updates

### Damping Adjustments (`vehicle_factory.rs`)
```rust
Damping {
    linear_damping: 0.2,  // Reduced from 1.0 to avoid double-damping
    angular_damping: 2.0, // Reduced from 5.0 for better responsiveness
}
Friction::coefficient(0.2), // Low friction for custom grip model
```

**Rationale**: 
- Prevents double-damping conflict between Rapier and custom grip system
- Allows custom lateral grip to dominate handling feel
- Low collider friction avoids interference with physics model

## Debug UI Integration

### F3 Overlay Now Shows
```
Performance Debug (F3)
FPS: 60.0 | Frame: 16.67ms
Entities: 1234
Chunks: 25 (Active: 8)

Vehicle:
Speed: 45.2 m/s (163 km/h)
Steering: -0.85 | E-Brake: OFF
Grip: 8.0 | Drift Grip: 2.5
Stability: 0.8
```

## Implementation Details

### Physics Flow
1. **World → Local**: `inv_rotation = transform.rotation.inverse()`
2. **Extract Components**: 
   - Forward speed: `-v_local.z`
   - Lateral velocity: `v_local.x`
3. **Apply Physics**:
   - Speed-based steering calculation
   - Lateral grip decay
   - Forward/backward acceleration
   - Downforce effect
4. **Local → World**: `world_velocity = transform.rotation * v_local`
5. **Apply**: Update `velocity.linvel.x/z`, preserve Y for gravity

### Key Formulas

**Speed-Based Steering**:
```rust
let steer_gain = specs.steer_gain / (1.0 + specs.steer_speed_drop * current_speed);
```

**Stability Term**:
```rust
let stability_term = -v_local.x * specs.stability;
```

**Effective Grip (with Downforce)**:
```rust
let speed_factor = (current_speed / specs.base_speed).min(1.0);
let effective_grip = grip * (1.0 + specs.downforce_scale * speed_factor);
```

**E-Brake Drift**:
```rust
let grip = if control_state.emergency_brake {
    target_yaw += control_state.steering.signum() * specs.ebrake_yaw_boost;
    specs.drift_grip
} else {
    specs.grip
};
```

## Coordinate System Reference

### Bevy Coordinate System (Right-Handed Y-Up)
- **X-axis**: Left (-) / Right (+)
- **Y-axis**: Down (-) / Up (+)  
- **Z-axis**: Forward (-) / Backward (+)

### Local Space Mapping
- `v_local.x`: Lateral velocity (left/right slide)
- `v_local.y`: Vertical velocity (gravity, preserved)
- `v_local.z`: Forward velocity (negative = forward motion)

## Tuning Guide

### For More Responsive Steering
- Increase `steer_gain` (e.g., `3.0`)
- Decrease `steer_speed_drop` (e.g., `0.02`)

### For Better Drifting
- Decrease `drift_grip` (e.g., `2.0`)
- Increase `ebrake_yaw_boost` (e.g., `0.8`)

### For More Stability
- Increase `stability` (e.g., `1.0`)
- Increase `grip` (e.g., `10.0`)
- Increase `downforce_scale` (e.g., `0.5`)

### For Snappier Acceleration
- Increase `accel_lerp` (e.g., `7.0`)
- Increase `brake_lerp` (e.g., `8.0`)

## Testing Checklist

- [x] Cars accelerate forward correctly
- [x] Braking works in correct direction
- [x] Steering turns car appropriately
- [x] E-brake enables drifting
- [x] High-speed steering feels reduced
- [x] Car auto-straightens during slides
- [x] Downforce improves high-speed stability
- [x] Debug UI displays vehicle info
- [x] No physics panics or explosions
- [x] 60+ FPS maintained

## Known Limitations

1. **Slopes**: Cars don't pitch to match terrain (visual-only rotation locked)
2. **Reverse Steering**: No reverse steering logic (left = left, regardless of direction)
3. **Tire Model**: Simplified grip model, no per-wheel physics
4. **ABS/TC**: No anti-lock brakes or traction control systems

## Future Enhancements (Optional)

- Visual-only pitch/roll based on acceleration (weight transfer feel)
- Per-surface grip variation (asphalt, dirt, ice)
- Simple traction circle (limit combined accel + lateral forces)
- Reverse steering inversion for realistic reversing
- Raycast suspension for better terrain following

## References

- Oracle Analysis: Confirmed coordinate system fix and physics correctness
- AGENT.md: Simplicity-first philosophy maintained
- GTA V / Need for Speed: Arcade handling reference targets
