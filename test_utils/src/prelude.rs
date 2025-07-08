// Re-export commonly used testing utilities
pub use crate::minimal_app::MinimalBevyApp;
pub use crate::world_helpers::{
    spawn_test_world, EntityBuilder, ScenarioBuilder, TestRng,
    validate_world_state, create_test_vehicle, create_test_building, create_test_ground,
};
pub use crate::screenshot::{
    compare_screenshot, ScreenshotConfig, ComparisonResult, GoldenFrameTest,
    create_test_pattern, save_screenshot, ensure_golden_frame_dir,
};

// Re-export commonly used Bevy types for testing
pub use bevy::prelude::*;
pub use bevy_rapier3d::prelude::*;
pub use rand::prelude::*;

// Disambiguate Real type to use Bevy's version  
pub use bevy::time::Real;

// Common test constants
/// Default test seed for reproducible tests
pub const DEFAULT_TEST_SEED: u64 = 42;
/// Default screenshot width
pub const DEFAULT_SCREENSHOT_WIDTH: u32 = 800;
/// Default screenshot height
pub const DEFAULT_SCREENSHOT_HEIGHT: u32 = 600;
/// Default tolerance for comparisons
pub const DEFAULT_TOLERANCE: f32 = 0.01;

/// Macro for creating simple test scenarios
#[macro_export]
macro_rules! test_scenario {
    ($name:ident, $seed:expr_2021, $setup:expr_2021) => {
        #[test]
        fn $name() {
            let mut world = spawn_test_world($seed);
            let entities = $setup(&mut world);
            validate_world_state(&world).expect("World validation failed");
        }
    };
}

/// Macro for creating golden frame tests
#[macro_export]
macro_rules! golden_frame_test {
    ($name:ident, $render_fn:expr_2021) => {
        #[test]
        fn $name() {
            let mut app = MinimalBevyApp::with_rendering();
            let image = $render_fn(&mut app);
            
            let test = GoldenFrameTest::new(stringify!($name));
            let result = test.run_test(&image).expect("Golden frame test failed");
            
            assert!(result.matches, 
                "Golden frame test failed: {}% difference (max: {}%)", 
                result.difference_percentage * 100.0,
                result.max_pixel_difference * 100.0
            );
        }
    };
}

/// Macro for creating physics simulation tests
#[macro_export]
macro_rules! physics_test {
    ($name:ident, $seed:expr_2021, $setup:expr_2021, $validate:expr_2021) => {
        #[test]
        fn $name() {
            let mut app = MinimalBevyApp::with_physics();
            let entities = $setup(app.world_mut());
            
            // Run simulation for a few frames
            for _ in 0..60 {
                app.app.update();
            }
            
            $validate(app.world(), &entities);
            validate_world_state(app.world()).expect("World validation failed");
        }
    };
}

/// Helper for creating deterministic test environments
pub fn create_deterministic_test_env(seed: u64) -> MinimalBevyApp {
    let mut app = MinimalBevyApp::with_physics();
    app.app.world_mut().insert_resource(TestRng(StdRng::seed_from_u64(seed)));
    app
}

/// Helper for creating test camera
pub fn create_test_camera(world: &mut World, position: Vec3, target: Vec3) -> Entity {
    world.spawn((
        Camera3d::default(),
        Transform::from_translation(position).looking_at(target, Vec3::Y),
    )).id()
}

/// Helper for creating test light
pub fn create_test_light(world: &mut World, position: Vec3) -> Entity {
    world.spawn((
        DirectionalLight::default(),
        Transform::from_translation(position),
    )).id()
}

/// Common test assertions
pub mod assertions {
    use super::*;
    
    /// Assert that an entity exists and has the expected components
    pub fn assert_entity_has_component<T: Component>(world: &World, entity: Entity) {
        assert!(world.entity(entity).contains::<T>(), 
            "Entity {:?} missing component {}", entity, std::any::type_name::<T>());
    }
    
    /// Assert that a transform is within expected bounds
    pub fn assert_transform_in_bounds(transform: &Transform, min: Vec3, max: Vec3) {
        assert!(transform.translation.x >= min.x && transform.translation.x <= max.x,
            "Transform X {} not in bounds [{}, {}]", transform.translation.x, min.x, max.x);
        assert!(transform.translation.y >= min.y && transform.translation.y <= max.y,
            "Transform Y {} not in bounds [{}, {}]", transform.translation.y, min.y, max.y);
        assert!(transform.translation.z >= min.z && transform.translation.z <= max.z,
            "Transform Z {} not in bounds [{}, {}]", transform.translation.z, min.z, max.z);
    }
    
    /// Assert that a velocity is within expected bounds
    pub fn assert_velocity_in_bounds(velocity: &Velocity, max_linear: f32, max_angular: f32) {
        assert!(velocity.linvel.length() <= max_linear,
            "Linear velocity {} exceeds maximum {}", velocity.linvel.length(), max_linear);
        assert!(velocity.angvel.length() <= max_angular,
            "Angular velocity {} exceeds maximum {}", velocity.angvel.length(), max_angular);
    }
    
    /// Assert that two Vec3 values are approximately equal
    pub fn assert_vec3_approx_eq(a: Vec3, b: Vec3, tolerance: f32) {
        let diff = (a - b).length();
        assert!(diff <= tolerance,
            "Vec3 values not approximately equal: {:?} vs {:?} (diff: {})", a, b, diff);
    }
}
