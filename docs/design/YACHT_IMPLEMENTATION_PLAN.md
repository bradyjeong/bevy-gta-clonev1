# Yacht Walkability & Helipad Fix - Implementation Plan

## Overview
Fix two critical issues:
1. **Cockpit Walkability**: Players can walk through cockpit walls (solid boxes have no character collision)
2. **Helipad Design**: Improve visual design + fix collision detection for helicopter landing

## Prerequisites
- Read file: `src/factories/vehicle_factory.rs` lines 696-965
- Understand: Yacht uses compound physics collider + visual mesh children
- Know: CollisionGroups from `self.config.physics` (vehicle_group, character_group, static_group)

---

## PART 1: Cockpit Collision Walls

### Problem
Current cockpit structures are visual-only meshes:
- **Lower Superstructure** (16×3×20 @ y=6.0, z=-5.0) - line 808
- **Upper Superstructure** (12×2.5×14 @ y=9.0, z=-5.0) - line 828  
- **Bridge** (8×2×8 @ y=12.0, z=-5.0) - line 848

These have NO collision, so characters walk through them.

### Solution
Add thin (0.2m) non-physics wall colliders that ONLY block characters, not physics.

### Implementation Steps

#### Step 1: Add Helper Function
Location: `src/factories/vehicle_factory.rs` - ADD AFTER line 695 (before `spawn_yacht`)

```rust
/// Helper: Spawn thin wall collider that only blocks characters
fn spawn_character_wall(
    commands: &mut Commands,
    parent: Entity,
    position: Vec3,
    half_extents: Vec3,
    collision_groups: CollisionGroups,
    name: &str,
) {
    commands.spawn((
        Transform::from_translation(position),
        Collider::cuboid(half_extents.x, half_extents.y, half_extents.z),
        collision_groups,
        Friction::coefficient(0.0),
        Restitution::coefficient(0.0),
        ChildOf(parent),
        Name::new(name.to_string()),
    ));
}
```

#### Step 2: Add Walls for Lower Superstructure
Location: `src/factories/vehicle_factory.rs` - ADD AFTER line 816 (after Lower Superstructure mesh spawn)

**Important**: Lower Superstructure is 16×3×20 at (0, 6.0, -5.0)
- Half-extents: hx=8.0, hy=1.5, hz=10.0
- Wall thickness: 0.1m
- Leave 3m gap on south side for door

```rust
// Lower Superstructure Walls (16×3×20 centered at y=6.0, z=-5.0)
let lower_wall_groups = CollisionGroups::new(
    self.config.physics.vehicle_group,
    self.config.physics.character_group,
);

// North wall (front) - full width
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(0.0, 6.0, -5.0 + 10.0 - 0.1), // z = -5 + 10 - 0.1 = 4.9
    Vec3::new(8.0, 1.4, 0.1), // 16m wide × 2.8m tall × 0.2m thick
    lower_wall_groups,
    "Lower SS North Wall",
);

// South wall (back) - split for 3m door gap in center
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(-6.5, 6.0, -5.0 - 10.0 + 0.1), // Left section
    Vec3::new(1.5, 1.4, 0.1), // 3m wide section
    lower_wall_groups,
    "Lower SS South Wall Left",
);
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(6.5, 6.0, -5.0 - 10.0 + 0.1), // Right section
    Vec3::new(1.5, 1.4, 0.1), // 3m wide section
    lower_wall_groups,
    "Lower SS South Wall Right",
);

// East wall (right side)
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(8.0 - 0.1, 6.0, -5.0), // x = 7.9
    Vec3::new(0.1, 1.4, 10.0), // 0.2m thick × 2.8m tall × 20m long
    lower_wall_groups,
    "Lower SS East Wall",
);

// West wall (left side)
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(-8.0 + 0.1, 6.0, -5.0), // x = -7.9
    Vec3::new(0.1, 1.4, 10.0),
    lower_wall_groups,
    "Lower SS West Wall",
);
```

#### Step 3: Add Walls for Upper Superstructure
Location: `src/factories/vehicle_factory.rs` - ADD AFTER line 836 (after Upper Superstructure mesh spawn)

**Important**: Upper Superstructure is 12×2.5×14 at (0, 9.0, -5.0)
- Half-extents: hx=6.0, hy=1.25, hz=7.0
- Leave 2m door gap on south side

```rust
// Upper Superstructure Walls (12×2.5×14 centered at y=9.0, z=-5.0)
let upper_wall_groups = CollisionGroups::new(
    self.config.physics.vehicle_group,
    self.config.physics.character_group,
);

// North wall
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(0.0, 9.0, -5.0 + 7.0 - 0.1), // z = 1.9
    Vec3::new(6.0, 1.15, 0.1), // 12m wide × 2.3m tall × 0.2m thick
    upper_wall_groups,
    "Upper SS North Wall",
);

// South wall - split for 2m door gap
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(-5.0, 9.0, -5.0 - 7.0 + 0.1), // Left section
    Vec3::new(1.0, 1.15, 0.1),
    upper_wall_groups,
    "Upper SS South Wall Left",
);
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(5.0, 9.0, -5.0 - 7.0 + 0.1), // Right section
    Vec3::new(1.0, 1.15, 0.1),
    upper_wall_groups,
    "Upper SS South Wall Right",
);

// East wall
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(6.0 - 0.1, 9.0, -5.0),
    Vec3::new(0.1, 1.15, 7.0),
    upper_wall_groups,
    "Upper SS East Wall",
);

// West wall
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(-6.0 + 0.1, 9.0, -5.0),
    Vec3::new(0.1, 1.15, 7.0),
    upper_wall_groups,
    "Upper SS West Wall",
);
```

#### Step 4: Add Walls for Bridge
Location: `src/factories/vehicle_factory.rs` - ADD AFTER line 856 (after Bridge mesh spawn)

**Important**: Bridge is 8×2×8 at (0, 12.0, -5.0)
- Half-extents: hx=4.0, hy=1.0, hz=4.0
- Fully enclosed (no doors - represents command center)

```rust
// Bridge Walls (8×2×8 centered at y=12.0, z=-5.0) - Fully enclosed
let bridge_wall_groups = CollisionGroups::new(
    self.config.physics.vehicle_group,
    self.config.physics.character_group,
);

// North wall
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(0.0, 12.0, -5.0 + 4.0 - 0.1),
    Vec3::new(4.0, 0.9, 0.1), // 8m wide × 1.8m tall × 0.2m thick
    bridge_wall_groups,
    "Bridge North Wall",
);

// South wall
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(0.0, 12.0, -5.0 - 4.0 + 0.1),
    Vec3::new(4.0, 0.9, 0.1),
    bridge_wall_groups,
    "Bridge South Wall",
);

// East wall
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(4.0 - 0.1, 12.0, -5.0),
    Vec3::new(0.1, 0.9, 4.0),
    bridge_wall_groups,
    "Bridge East Wall",
);

// West wall
spawn_character_wall(
    &mut commands,
    vehicle_entity,
    Vec3::new(-4.0 + 0.1, 12.0, -5.0),
    Vec3::new(0.1, 0.9, 4.0),
    bridge_wall_groups,
    "Bridge West Wall",
);
```

#### Step 5: Import Required Types
Location: `src/factories/vehicle_factory.rs` - TOP OF FILE (around lines 1-20)

**Check if these imports exist**, if NOT, add them:

```rust
use bevy_rapier3d::prelude::{Friction, Restitution};
```

---

## PART 2: Helipad Visual Redesign

### Problem
Current helipad is basic:
- Simple gray square (12×0.5×12)
- Single yellow circle
- No "H" marking
- No corner markings

### Solution
SpaceX drone ship style:
- Hexagonal platform shape
- Light gray surface
- Yellow circle outline with "H" letter inside
- Corner numbers/markers
- Professional appearance

### Implementation Steps

#### Step 1: Replace Helipad Platform Visual
Location: `src/factories/vehicle_factory.rs` - REPLACE lines 868-876

**OLD CODE** (DELETE):
```rust
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(12.0, 0.5, 12.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.3))),
    Transform::from_xyz(0.0, 6.5, 10.0),
    ChildOf(vehicle_entity),
    VisibleChildBundle::default(),
    yacht_visibility(),
    Name::new("Helipad"),
));
```

**NEW CODE** (hexagonal platform):
```rust
// Helipad Platform - Hexagonal shape (14m diameter, light gray)
let helipad_platform_material = materials.add(StandardMaterial {
    base_color: Color::srgb(0.75, 0.75, 0.78), // Light gray
    metallic: 0.2,
    perceptual_roughness: 0.8,
    ..default()
});

// Create hexagon mesh (7m radius, 0.5m height)
let helipad_mesh = meshes.add(Cylinder::new(7.0, 0.5).mesh().resolution(6));

commands.spawn((
    Mesh3d(helipad_mesh),
    MeshMaterial3d(helipad_platform_material),
    Transform::from_xyz(0.0, 6.5, 10.0),
    ChildOf(vehicle_entity),
    VisibleChildBundle::default(),
    yacht_visibility(),
    Name::new("Helipad Platform"),
));
```

#### Step 2: Replace Yellow Circle
Location: `src/factories/vehicle_factory.rs` - REPLACE lines 878-887

**OLD CODE** (DELETE):
```rust
commands.spawn((
    Mesh3d(meshes.add(Cylinder::new(5.5, 0.1))),
    MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.1))),
    Transform::from_xyz(0.0, 7.0, 10.0)
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ChildOf(vehicle_entity),
    VisibleChildBundle::default(),
    yacht_visibility(),
    Name::new("Helipad Circle"),
));
```

**NEW CODE** (yellow circle outline):
```rust
// Yellow circle outline (5m radius, thin torus-like ring)
let yellow_material = materials.add(StandardMaterial {
    base_color: Color::srgb(0.95, 0.75, 0.05), // Bright yellow
    metallic: 0.1,
    perceptual_roughness: 0.6,
    ..default()
});

commands.spawn((
    Mesh3d(meshes.add(Torus {
        minor_radius: 0.15, // Ring thickness
        major_radius: 5.0,  // Circle radius
    })),
    MeshMaterial3d(yellow_material.clone()),
    Transform::from_xyz(0.0, 7.05, 10.0) // Slightly above platform
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ChildOf(vehicle_entity),
    VisibleChildBundle::default(),
    yacht_visibility(),
    Name::new("Helipad Circle Outline"),
));
```

#### Step 3: Add "H" Letter Marking
Location: `src/factories/vehicle_factory.rs` - ADD AFTER yellow circle code

**NEW CODE** (large "H" made from rectangles):
```rust
// "H" Letter - Made from three rectangles (2 vertical bars + 1 horizontal bar)
let h_material = materials.add(StandardMaterial {
    base_color: Color::srgb(0.05, 0.05, 0.05), // Dark gray/black
    metallic: 0.1,
    perceptual_roughness: 0.9,
    ..default()
});

// Left vertical bar of H
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(0.6, 0.05, 2.5))),
    MeshMaterial3d(h_material.clone()),
    Transform::from_xyz(-1.0, 7.1, 10.0), // Slightly above circle
    ChildOf(vehicle_entity),
    VisibleChildBundle::default(),
    yacht_visibility(),
    Name::new("Helipad H - Left Bar"),
));

// Right vertical bar of H
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(0.6, 0.05, 2.5))),
    MeshMaterial3d(h_material.clone()),
    Transform::from_xyz(1.0, 7.1, 10.0),
    ChildOf(vehicle_entity),
    VisibleChildBundle::default(),
    yacht_visibility(),
    Name::new("Helipad H - Right Bar"),
));

// Horizontal crossbar of H
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(2.0, 0.05, 0.6))),
    MeshMaterial3d(h_material),
    Transform::from_xyz(0.0, 7.1, 10.0),
    ChildOf(vehicle_entity),
    VisibleChildBundle::default(),
    yacht_visibility(),
    Name::new("Helipad H - Crossbar"),
));
```

#### Step 4: Add Corner Markers
Location: `src/factories/vehicle_factory.rs` - ADD AFTER "H" letter code

**NEW CODE** (orientation markers at hexagon corners):
```rust
// Corner markers for orientation (4 small rectangles at cardinal directions)
let marker_material = materials.add(StandardMaterial {
    base_color: Color::srgb(0.1, 0.1, 0.1), // Dark markers
    metallic: 0.1,
    perceptual_roughness: 0.9,
    ..default()
});

let corner_positions = [
    Vec3::new(0.0, 7.08, 10.0 + 6.0),   // North
    Vec3::new(0.0, 7.08, 10.0 - 6.0),   // South
    Vec3::new(6.0, 7.08, 10.0),         // East
    Vec3::new(-6.0, 7.08, 10.0),        // West
];

for (i, pos) in corner_positions.iter().enumerate() {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.8, 0.05, 0.8))),
        MeshMaterial3d(marker_material.clone()),
        Transform::from_translation(*pos),
        ChildOf(vehicle_entity),
        VisibleChildBundle::default(),
        yacht_visibility(),
        Name::new(format!("Helipad Corner Marker {}", i + 1)),
    ));
}
```

---

## PART 3: Helipad Collision & Detection Fixes

### Problem
1. Physics collider too small (6×0.25×6 vs 12×12 visual)
2. Sensor detection volume too small (6×1×6)
3. Height misalignment for landing detection

### Solution
Enlarge both physics and sensor to match new 14m diameter hexagon

### Implementation Steps

#### Step 1: Fix Physics Collider
Location: `src/factories/vehicle_factory.rs` - lines 711-727 (compound collider definition)

**OLD CODE** (line 717-721):
```rust
(
    Vec3::new(0.0, 6.5, 10.0),
    Quat::IDENTITY,
    Collider::cuboid(6.0, 0.25, 6.0),
),
```

**NEW CODE** (REPLACE with):
```rust
(
    Vec3::new(0.0, 7.0, 10.0), // Raised to top of platform
    Quat::IDENTITY,
    Collider::cuboid(7.0, 0.25, 7.0), // Matches 14m hexagon diameter
),
```

#### Step 2: Fix Sensor Detection Volume
Location: `src/factories/vehicle_factory.rs` - lines 927-934

**OLD CODE**:
```rust
commands.spawn((
    Transform::from_xyz(0.0, 6.5, 10.0),
    Collider::cuboid(6.0, 1.0, 6.0),
    Sensor,
    ChildOf(vehicle_entity),
    crate::components::Helipad,
    Name::new("HelipadVolume"),
));
```

**NEW CODE** (REPLACE with):
```rust
commands.spawn((
    Transform::from_xyz(0.0, 7.5, 10.0), // Center of detection zone above platform
    Collider::cuboid(7.0, 2.0, 7.0), // Larger detection (14m × 4m height × 14m)
    Sensor,
    ChildOf(vehicle_entity),
    crate::components::Helipad,
    Name::new("HelipadVolume"),
));
```

---

## PART 4: Validation & Testing

### Pre-Commit Checks

**CRITICAL: Run these commands BEFORE committing**

```bash
# 1. Check for compile errors
cargo check

# 2. Check for warnings
cargo clippy -- -D warnings

# 3. Format code
cargo fmt

# 4. Run tests (if any yacht tests exist)
cargo test yacht
```

### Runtime Testing

1. **Launch game**:
   ```bash
   cargo run --features debug-ui
   ```

2. **Press F3** to enable debug overlay with collider visualization

3. **Test Cockpit Walls**:
   - Walk yacht to yacht
   - Exit to deck with `F` key
   - Try walking into Lower Superstructure - should be BLOCKED
   - Walk around perimeter - should hit invisible walls
   - Try to pass through door gaps - should be ABLE to enter

4. **Test Helipad**:
   - Visual check: Should see hexagon with yellow circle + "H" + corner markers
   - Get helicopter (spawn or find one)
   - Fly to yacht helipad (forward section at z=+10)
   - Hover over helipad and descend slowly
   - Should see "LandedOnYacht" component added (check debug UI F3)
   - Press `F` while in yacht - should transfer to helicopter if landed

5. **Check for Issues**:
   - No z-fighting (flickering textures)
   - Smooth collision (no snagging on corners)
   - Proper landing detection (5m distance + velocity < 2.0)

### Common Errors

**Error: "cannot find type `Friction` in this scope"**
- Fix: Add import `use bevy_rapier3d::prelude::{Friction, Restitution};`

**Error: "mismatched types, expected `Vec3` found `(f32, f32, f32)`"**
- Fix: Use `Vec3::new(x, y, z)` not `(x, y, z)`

**Error: Wall colliders not blocking character**
- Fix: Check collision groups mask includes `character_group`

**Error: Helicopter not detecting landing**
- Fix: Verify HelipadVolume sensor is large enough (7×2×7) and at correct height (y=7.5)

**Visual: Helipad looks flat/wrong**
- Fix: Check hexagon mesh has resolution(6) for 6 sides
- Fix: Verify "H" bars are at y=7.1 (above platform surface)

---

## File Summary

**Files Modified**: 1
- `src/factories/vehicle_factory.rs`

**Approximate Changes**:
- Add 1 helper function (~15 lines)
- Add cockpit walls (~80 lines)
- Replace helipad visuals (~100 lines)
- Fix physics collider (1 line change)
- Fix sensor volume (1 line change)

**Total**: ~200 lines added/modified

---

## Final Checklist

- [ ] Added `spawn_character_wall` helper function
- [ ] Added 4-5 walls for Lower Superstructure with door gap
- [ ] Added 4-5 walls for Upper Superstructure with door gap
- [ ] Added 4 walls for Bridge (fully enclosed)
- [ ] Replaced helipad platform with hexagon (resolution 6)
- [ ] Replaced yellow circle with torus outline
- [ ] Added "H" letter (3 rectangles)
- [ ] Added 4 corner markers
- [ ] Fixed physics collider size (7×0.25×7) and height (y=7.0)
- [ ] Fixed sensor volume (7×2×7) and height (y=7.5)
- [ ] Added required imports (Friction, Restitution)
- [ ] Ran `cargo check` successfully
- [ ] Ran `cargo clippy` with no warnings
- [ ] Ran `cargo fmt`
- [ ] Tested in-game with debug-ui enabled
- [ ] Verified walls block character movement
- [ ] Verified door gaps allow entry
- [ ] Verified helipad looks professional
- [ ] Verified helicopter landing detection works

---

## Time Estimate
- Implementation: 30-45 minutes
- Testing: 15-20 minutes
- Debugging (if needed): 10-30 minutes
- **Total**: 1-1.5 hours
