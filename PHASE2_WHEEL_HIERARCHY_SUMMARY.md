# Phase 2: Visual Wheel Hierarchy Implementation

## Summary
Successfully implemented Phase 2 of the visual wheel system by modifying the vehicle factory to spawn a proper wheel hierarchy under VisualRigRoot.

## Changes Made

### 1. Modified vehicle_factory.rs Imports
Added wheel-related component imports:
- `CarWheelsConfig`
- `WheelMesh`
- `WheelPos`
- `WheelSteerPivot`
- `WheelsRoot`

### 2. Replaced Old Wheel Spawning (Lines 179-199)
Replaced simple wheel spawning with hierarchical system.

### 3. New Hierarchy Structure
```
CarRoot (vehicle physics entity)
└─ VisualRigRoot (Phase 3 body lean)
   ├─ Body meshes (chassis, cabin, windshield, hood)
   └─ WheelsRoot (Phase 2)
      ├─ FL_SteerPivot
      │  └─ FL_Wheel (WheelMesh component)
      ├─ FR_SteerPivot
      │  └─ FR_Wheel (WheelMesh component)
      ├─ RL_Wheel (WheelMesh component, no pivot)
      └─ RR_Wheel (WheelMesh component, no pivot)
```

## Implementation Details

### WheelsRoot Creation
- Spawned as child of VisualRigRoot
- Contains all wheel-related entities
- Groups wheels for easier management

### CarWheelsConfig Attachment
- Loaded from SimpleCarSpecs default values
- Attached to vehicle entity (not rig_root)
- Contains:
  - `max_steer_rad`: Maximum steering angle in radians (from max_steer_deg)
  - `wheel_radius`: Wheel radius for animation calculations

### Front Wheels (Steerable)
- FL (Front Left) and FR (Front Right)
- Two-level hierarchy:
  1. **SteerPivot**: Rotation point for steering
     - Position from `wheel_positions[0]` and `wheel_positions[1]`
     - Has `WheelSteerPivot` component with position marker
  2. **WheelMesh**: Visual wheel under pivot
     - Has `WheelMesh` component with:
       - `pos`: WheelPos enum (FL or FR)
       - `radius`: From SimpleCarSpecs
       - `roll_angle`: 0.0 initial
       - `roll_dir`: 1.0 for axis correction

### Rear Wheels (Non-Steerable)
- RL (Rear Left) and RR (Rear Right)
- Single-level hierarchy:
  - Attached directly to WheelsRoot (no pivot needed)
  - Has `WheelMesh` component with position and radius
  - Position from `wheel_positions[2]` and `wheel_positions[3]`

## Configuration Source

### Wheel Positions (from simple_car.ron)
```ron
wheel_positions: [
    (0.85, -0.32, 1.40),   // FL: Front Left
    (-0.85, -0.32, 1.40),  // FR: Front Right
    (0.85, -0.32, -1.40),  // RL: Rear Left
    (-0.85, -0.32, -1.40), // RR: Rear Right
],
```

### Wheel Config (from simple_car.ron)
- `max_steer_deg`: 28.0° → converted to radians
- `wheel_radius`: 0.33 meters

## Mesh Creation
- Uses existing `MeshFactory::create_sports_wheel()` for visual mesh
- Dark color material (0.1, 0.1, 0.12) for tire appearance
- Rotated 90° around Z-axis for proper wheel orientation

## Component Attachment Pattern
1. **Vehicle Entity**: Gets `CarWheelsConfig` for global wheel settings
2. **SteerPivot Entities**: Get `WheelSteerPivot` component for steering animation
3. **Wheel Entities**: Get `WheelMesh` component for roll animation

## Benefits of This Hierarchy

### 1. Clean Separation of Concerns
- Steering: Controlled by pivot rotation
- Rolling: Controlled by wheel mesh rotation
- Position: Defined by RON config, not hardcoded

### 2. Efficient Animation
- Front wheels: Rotate pivot for steering, rotate mesh for rolling
- Rear wheels: Only rotate mesh for rolling (no steering overhead)

### 3. Data-Driven Configuration
- All wheel specs loaded from `simple_car.ron`
- Easy to adjust without code changes
- Consistent with other vehicle specs (YachtSpecs pattern)

### 4. Proper Parenting
- Wheels inherit visual rig transformations (body lean)
- Clear hierarchy visible in inspector (F3)
- Easy to debug with named entities

## Verification

### Commands Run
```bash
cargo check        # ✓ Passed
cargo clippy       # ✓ Passed (fixed inline format args)
cargo fmt          # ✓ Formatted
```

### Expected Entity Names in Inspector
- `SuperCar` (root)
  - `CarRigRoot`
    - `WheelsRoot`
      - `FL_SteerPivot`
        - `FL_Wheel`
      - `FR_SteerPivot`
        - `FR_Wheel`
      - `RL_Wheel`
      - `RR_Wheel`

## Next Steps (Phase 3)

Phase 3 will implement the animation systems:
1. **Steering animation**: Rotate `WheelSteerPivot` based on control input
2. **Roll animation**: Rotate `WheelMesh` based on vehicle velocity
3. **Ground contact detection**: Adjust wheel height for suspension
4. **Asset loading**: Update config when RON file loads

## Notes

- Used `ChildOf()` for proper Bevy 0.16 hierarchy
- All wheels get proper visibility components for rendering
- Wheel radius from config (0.33m) used for WheelMesh component
- Default specs used during spawning (actual asset loads later)
- Clean separation: factory creates structure, systems will animate it
