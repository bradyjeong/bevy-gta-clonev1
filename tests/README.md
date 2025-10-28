# Test Infrastructure

Comprehensive testing utilities for Bevy GTA-clone game.

## Structure

```
tests/
├── common/
│   ├── mod.rs              # Core test helpers and app setup
│   └── test_entities.rs    # Vehicle spawning utilities
├── test_infrastructure.rs  # Infrastructure validation tests
└── README.md
```

## Quick Start

### Running Tests

```bash
cargo test test_name          # Run specific test
cargo test --test test_infrastructure  # Run infrastructure tests
cargo test                    # Run all tests
```

### Basic Usage

```rust
mod common;

use bevy::prelude::*;
use common::*;

#[test]
fn test_car_physics() {
    let mut app = setup_test_app_with_rapier();
    let car = test_entities::spawn_test_car(&mut app, Vec3::new(0.0, 5.0, 0.0));
    
    test_entities::set_test_control_state(&mut app, car, 1.0, 0.0, 0.0);
    run_app_updates(&mut app, 60); // Simulate 1 second
    
    let velocity = get_velocity(&app, car);
    assert!(velocity.length() > 0.0, "Car should be moving");
}
```

## Test Helpers

### App Setup Functions

- **`setup_test_app()`** - Minimal Bevy app without physics
  - Use for: Component/system tests that don't need physics
  
- **`setup_test_app_with_rapier()`** - Full physics-enabled app
  - Use for: Vehicle physics, collision, movement tests
  - Includes: RapierPhysicsPlugin, ScenePlugin, 60 FPS fixed timestep

- **`run_app_updates(app, frames)`** - Simulate frame updates
  - Example: `run_app_updates(&mut app, 60)` simulates 1 second at 60 FPS

### Entity Spawning

- **`spawn_test_car(app, position)`** - Basic car with physics
  - Mass: 1200kg, Collider: cuboid, CCD enabled
  
- **`spawn_test_helicopter(app, position)`** - Helicopter
  - Mass: 2000kg, Collider: capsule, higher damping
  
- **`spawn_test_f16(app, position)`** - F16 fighter jet
  - Mass: 12000kg, Collider: capsule, low damping
  
- **`spawn_test_ground(app, position, size)`** - Static ground plane
  - For collision testing

### Control & State Helpers

- **`set_test_control_state(app, entity, throttle, brake, steering)`**
  - Set car controls (0.0 to 1.0 for throttle/brake, -1.0 to 1.0 for steering)
  
- **`set_test_vertical_control(app, entity, vertical)`**
  - Aircraft vertical control (-1.0 down, 1.0 up)
  
- **`set_test_flight_controls(app, entity, pitch, yaw, roll)`**
  - Full aircraft controls
  
- **`apply_test_velocity(app, entity, velocity)`**
  - Directly set velocity (bypasses movement systems)

### Query Helpers

- **`get_velocity(app, entity)`** - Get entity velocity as Vec3
- **`get_position(app, entity)`** - Get entity position as Vec3
- **`entity_exists(app, entity)`** - Check if entity still exists

### Assertions

- **`assert_velocity_near(app, entity, expected, epsilon)`**
  - Assert velocity within tolerance
  
- **`assert_position_near(app, entity, expected, epsilon)`**
  - Assert position within tolerance

## Constants

```rust
TEST_TIMEOUT: Duration         // 30 seconds
TEST_DELTA_TIME: f32           // 16.67ms (60 FPS)
TEST_SPAWN_POSITION: Vec3      // Vec3::new(0.0, 5.0, 0.0)
VELOCITY_EPSILON: f32          // 0.01 m/s
POSITION_EPSILON: f32          // 0.1 meters
```

## Best Practices

1. **Use `setup_test_app_with_rapier()` for vehicle tests** - Physics is required
2. **Spawn ground for collision tests** - `spawn_test_ground()`
3. **Run updates to see physics effects** - `run_app_updates(&mut app, frames)`
4. **Use assertions with epsilon** - Floating point comparisons need tolerance
5. **Check entity exists before querying** - `entity_exists()` prevents panics

## Examples

### Testing Car Movement
```rust
#[test]
fn test_car_acceleration() {
    let mut app = setup_test_app_with_rapier();
    let ground = test_entities::spawn_test_ground(&mut app, Vec3::ZERO, 100.0);
    let car = test_entities::spawn_test_car(&mut app, TEST_SPAWN_POSITION);
    
    test_entities::set_test_control_state(&mut app, car, 1.0, 0.0, 0.0);
    run_app_updates(&mut app, 30);
    
    let vel = get_velocity(&app, car);
    assert!(vel.length() > 0.0, "Car should accelerate");
}
```

### Testing Helicopter Hover
```rust
#[test]
fn test_helicopter_hover() {
    let mut app = setup_test_app_with_rapier();
    let heli = test_entities::spawn_test_helicopter(&mut app, Vec3::new(0.0, 20.0, 0.0));
    
    test_entities::set_test_vertical_control(&mut app, heli, 0.5);
    run_app_updates(&mut app, 60);
    
    let pos = get_position(&app, heli);
    assert!(pos.y >= 20.0, "Helicopter should maintain/gain altitude");
}
```

### Testing Collision
```rust
#[test]
fn test_ground_collision() {
    let mut app = setup_test_app_with_rapier();
    let ground = test_entities::spawn_test_ground(&mut app, Vec3::ZERO, 100.0);
    let car = test_entities::spawn_test_car(&mut app, Vec3::new(0.0, 10.0, 0.0));
    
    run_app_updates(&mut app, 120); // Let car fall and settle
    
    let pos = get_position(&app, car);
    assert!(pos.y < 3.0 && pos.y > 0.0, "Car should rest on ground");
}
```
