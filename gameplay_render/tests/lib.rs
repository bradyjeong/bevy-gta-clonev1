#![cfg(feature = "heavy_tests")]
#![allow(unexpected_cfgs)]
//! Comprehensive test suite for gameplay_render crate
//! 
//! Tests LOD/culling correctness, rendering pipeline, performance validation,
//! and visual quality systems following the oracle's guidance.

// Import all test modules
mod utils;
mod lod_culling;
mod rendering_pipeline;
mod performance_validation;
mod visual_quality;
mod property_tests;
// Re-export test utilities
pub use utils::*;
// Integration with existing workspace test framework
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_utils_available() {
        // Verify test utilities are working
        let app = create_render_test_app();
        assert!(app.world().entities().len() > 0, "Test app should have entities");
    }
    
    #[test]
    fn test_lod_distances_config() {
        let config = create_test_game_config();
        
        // Verify LOD distances match oracle requirements
        assert_eq!(config.world.lod_distances[0], 150.0, "Vehicle LOD distance should be 150m");
        assert_eq!(config.npc.update_intervals.close_distance, 100.0, "NPC close distance should be 100m");
        assert_eq!(config.world.streaming_radius, 300.0, "Streaming radius should be 300m for buildings");
    }
    
    #[test]
    fn test_rendering_components() {
        use bevy::prelude::*;
        use gameplay_render::prelude::*;
        let mut app = create_render_test_app();
        // Test LOD components
        let entity = app.world_mut().spawn((
            LodLevel::High,
            HighDetailVehicle,
            Transform::default(),
            Visibility::Visible,
        )).id();
        let world = app.world();
        assert!(world.get::<LodLevel>(entity).is_some(), "LOD component should be present");
        assert!(world.get::<HighDetailVehicle>(entity).is_some(), "High detail marker should be present");
    }
}
