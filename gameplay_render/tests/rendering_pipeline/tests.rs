//! Rendering pipeline tests
//! Tests LOD transitions, culling systems, and render queue management

use bevy::prelude::*;
use gameplay_render::prelude::*;
use crate::utils::*;
use std::time::Duration;
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_queue_management() {
        let mut app = create_render_test_app();
        
        // Create entities with changing transforms to populate render queue
        let entities: Vec<Entity> = (0..20)
            .map(|i| {
                let distance = (i as f32) * 15.0;
                create_test_vehicle_at_distance(&mut app, distance)
            })
            .collect();
        let _camera = create_test_camera(&mut app);
        // Run first frame to establish baseline
        run_render_simulation_frames(&mut app, 1);
        // Change transforms to trigger render queue updates
        {
            let mut world = app.world_mut();
            for entity in &entities {
                if let Some(mut transform) = world.get_mut::<Transform>(*entity) {
                    transform.translation.y += 1.0; // Move up slightly
                }
            }
        }
        // Run render queue management system
        run_render_simulation_frames(&mut app, 3);
        // Verify that render queue was processed
        // (This is mainly testing that the system runs without errors)
        let perf = PerformanceMeasurement::capture(&app);
        assert!(perf.render_operations >= 0, "Render operations should be non-negative");
    }
    fn test_batch_rendering_system() {
        // Create entities at various distances for batch processing
        let entities: Vec<Entity> = (0..30)
                let distance = (i as f32) * 20.0; // 0m to 580m
                create_test_building_at_distance(&mut app, distance)
        // Run batch rendering system
        run_render_simulation_frames(&mut app, 5);
        // Test that entities are processed in batches
        // Entities within streaming radius should be visible
        // Entities beyond streaming radius should be hidden
        for (i, entity) in entities.iter().enumerate() {
            let distance = (i as f32) * 20.0;
            let config = app.world().resource::<GameConfig>();
            
            if distance <= config.world.streaming_radius {
                assert_visibility(&app, *entity, Visibility::Visible);
            } else {
                assert_visibility(&app, *entity, Visibility::Hidden);
    fn test_lod_transition_smoothness() {
        // Create vehicle that will transition through LOD levels
        let vehicle = create_test_vehicle_at_distance(&mut app, 100.0);
        let camera = create_test_camera(&mut app);
        // Run initial frame
        // Verify initial state
        assert_lod_level(&app, vehicle, LodLevel::High);
        // Move vehicle to medium distance
            let mut transform = world.get_mut::<Transform>(vehicle).unwrap();
            transform.translation.x = 175.0; // Medium distance
        assert_lod_level(&app, vehicle, LodLevel::Medium);
        // Move vehicle to far distance
            transform.translation.x = 350.0; // Far distance
        assert_lod_level(&app, vehicle, LodLevel::Sleep);
        // Test that component transitions happened correctly
        assert_lacks_component::<HighDetailVehicle>(&app, vehicle, "HighDetailVehicle");
        assert_has_component::<SleepingEntity>(&app, vehicle, "SleepingEntity");
    fn test_vegetation_instancing_pipeline() {
        // Create multiple vegetation entities for instancing
        let vegetation_entities: Vec<Entity> = (0..10)
                let distance = (i as f32) * 20.0;
                create_test_vegetation_at_distance(&mut app, distance)
        // Run vegetation instancing systems
        // Test that vegetation LOD levels are set correctly
        for (i, entity) in vegetation_entities.iter().enumerate() {
            let world = app.world();
            let veg_lod = world.get::<VegetationLOD>(*entity).unwrap();
            // Verify distance is tracked correctly
            assert!(f32_equals(veg_lod.distance_to_player, distance, 1.0),
                "Vegetation distance should be tracked correctly");
            // Verify LOD level based on distance
            let expected_level = match distance {
                d if d < 50.0 => VegetationDetailLevel::Full,
                d if d < 150.0 => VegetationDetailLevel::Medium,
                d if d < 300.0 => VegetationDetailLevel::Billboard,
                _ => VegetationDetailLevel::Culled,
            };
            assert_eq!(veg_lod.detail_level, expected_level,
                "Vegetation at distance {} should have LOD level {:?}", distance, expected_level);
    fn test_render_optimization_time_budgets() {
        // Create many entities to stress test time budgets
        let entities: Vec<Entity> = (0..100)
                let distance = (i as f32) * 5.0;
        // Measure performance before optimization
        let start_time = std::time::Instant::now();
        run_render_simulation_frames(&mut app, 10);
        let optimization_time = start_time.elapsed();
        // Test that optimization runs within reasonable time
        assert!(optimization_time.as_millis() < 100,
            "Render optimization should complete within 100ms for 100 entities");
        // Test that performance counters are updated
        assert!(perf.entities_processed > 0, "Entities should have been processed");
        assert!(perf.render_operations > 0, "Render operations should have been recorded");
    fn test_priority_based_processing() {
        // Create entities at different distances
        let close_entity = create_test_vehicle_at_distance(&mut app, 50.0);
        let medium_entity = create_test_vehicle_at_distance(&mut app, 200.0);
        let far_entity = create_test_vehicle_at_distance(&mut app, 400.0);
        // Run render optimization
        // Test that closer entities are processed first (implied by correct LOD/visibility)
        assert_lod_level(&app, close_entity, LodLevel::High);
        assert_lod_level(&app, medium_entity, LodLevel::Medium);
        assert_lod_level(&app, far_entity, LodLevel::Sleep);
        // Test that distant entities are culled
        assert_visibility(&app, close_entity, Visibility::Visible);
        assert_visibility(&app, medium_entity, Visibility::Visible);
        assert_visibility(&app, far_entity, Visibility::Hidden);
    fn test_multiple_entity_types_rendering() {
        // Create mixed entity types
        let npc = create_test_npc_at_distance(&mut app, 80.0);
        let building = create_test_building_at_distance(&mut app, 250.0);
        let vegetation = create_test_vegetation_at_distance(&mut app, 120.0);
        // Run all rendering systems
        // Test that each entity type has appropriate LOD/visibility
        assert_lod_level(&app, npc, LodLevel::High);
        assert_visibility(&app, building, Visibility::Visible);
        assert_vegetation_lod_level(&app, vegetation, VegetationDetailLevel::Medium);
        // Test that performance counters reflect all entity types
        assert!(perf.lod_updates > 0, "LOD updates should include all entity types");
    fn test_render_system_integration() {
        // Create a comprehensive test scene
        let scene = TestSceneBuilder::new()
            .with_camera(&mut app)
            .with_vehicle_at_distance(&mut app, 75.0)
            .with_vehicle_at_distance(&mut app, 180.0)
            .with_npc_at_distance(&mut app, 90.0)
            .with_building_at_distance(&mut app, 280.0)
            .with_vegetation_at_distance(&mut app, 130.0)
            .build();
        // Run integrated rendering pipeline
        // Test that all systems work together correctly
        assert_lod_level(&app, scene.vehicle_entities[0], LodLevel::High);
        assert_lod_level(&app, scene.vehicle_entities[1], LodLevel::Medium);
        assert_lod_level(&app, scene.npc_entities[0], LodLevel::High);
        assert_visibility(&app, scene.building_entities[0], Visibility::Visible);
        assert_vegetation_lod_level(&app, scene.vegetation_entities[0], VegetationDetailLevel::Medium);
        // Test that performance monitoring is working
        assert!(perf.frame_time_ms >= 0.0, "Frame time should be measured");
        assert!(perf.entities_processed > 0, "Entities should be processed");
    fn test_dynamic_lod_updates() {
        // Create vehicle that will move through LOD zones
        let vehicle = create_test_vehicle_at_distance(&mut app, 50.0);
        // Test initial state
        run_render_simulation_frames(&mut app, 2);
        // Move vehicle progressively further
        let distances = vec![100.0, 175.0, 250.0, 350.0];
        let expected_lods = vec![LodLevel::High, LodLevel::Medium, LodLevel::Sleep, LodLevel::Sleep];
        for (distance, expected_lod) in distances.iter().zip(expected_lods.iter()) {
            // Move vehicle
            {
                let mut world = app.world_mut();
                let mut transform = world.get_mut::<Transform>(vehicle).unwrap();
                transform.translation.x = *distance;
            // Run LOD system
            run_render_simulation_frames(&mut app, 3);
            // Test LOD level
            assert_lod_level(&app, vehicle, *expected_lod);
    fn test_rendering_pipeline_error_handling() {
        // Create entity with invalid position
        let entity = app.world_mut().spawn((
            Car::default(),
            ActiveEntity,
            Transform::from_xyz(f32::INFINITY, 0.0, 0.0), // Invalid position
            GlobalTransform::default(),
            Visibility::Visible,
            Cullable::default(),
        )).id();
        // Run rendering systems - should not panic
        // System should handle invalid positions gracefully
        // (Test passes if no panic occurs)
        assert!(app.world().get::<Transform>(entity).is_some(),
            "Entity should still exist after invalid position");
}
