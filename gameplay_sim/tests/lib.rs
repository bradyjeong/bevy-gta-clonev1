// Gameplay simulation test suite
// Comprehensive tests for physics, AI behavior, and game rules

// Import all test modules
mod utils;
mod physics;
mod ai_behavior;
mod game_rules;
mod integration;
// Re-export test utilities for use in test modules
pub use utils::*;
// Integration with existing workspace test framework
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_utils_available() {
        // Verify test utilities are working
        let app = create_test_app();
        assert!(app.world().entities().len() > 0, "Test app should have entities");
    }
    fn test_physics_validator() {
        use bevy::prelude::*;
        use bevy_rapier3d::prelude::*;
        
        let velocity = Velocity {
            linvel: Vec3::new(10.0, 0.0, 0.0),
            angvel: Vec3::ZERO,
        };
        let result = PhysicsValidator::validate_velocity(&velocity, 50.0);
        assert!(result.is_ok(), "Valid velocity should pass validation");
        let invalid_velocity = Velocity {
            linvel: Vec3::new(f32::INFINITY, 0.0, 0.0),
        let result = PhysicsValidator::validate_velocity(&invalid_velocity, 50.0);
        assert!(result.is_err(), "Invalid velocity should fail validation");
}
