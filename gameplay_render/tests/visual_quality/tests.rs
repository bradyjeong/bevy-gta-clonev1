#![cfg(feature = "heavy_tests")]
//! Visual quality tests
//! Tests lighting, shadows, material systems, and rendering quality

use bevy::prelude::*;
use gameplay_render::prelude::*;
use crate::utils::*;
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lod_visual_quality_consistency() {
        let mut app = create_render_test_app();
        
        // Create vehicles at different LOD distances
        let high_detail_vehicle = create_test_vehicle_at_distance(&mut app, 50.0);
        let medium_detail_vehicle = create_test_vehicle_at_distance(&mut app, 175.0);
        let low_detail_vehicle = create_test_vehicle_at_distance(&mut app, 350.0);
        let _camera = create_test_camera(&mut app);
        // Run LOD systems
        run_render_simulation_frames(&mut app, 5);
        // Test that LOD levels are set correctly
        assert_lod_level(&app, high_detail_vehicle, LodLevel::High);
        assert_lod_level(&app, medium_detail_vehicle, LodLevel::Medium);
        assert_lod_level(&app, low_detail_vehicle, LodLevel::Sleep);
        // Test visual quality markers
        assert_has_component::<HighDetailVehicle>(&app, high_detail_vehicle, "HighDetailVehicle");
        assert_lacks_component::<HighDetailVehicle>(&app, medium_detail_vehicle, "HighDetailVehicle");
        assert_has_component::<SleepingEntity>(&app, low_detail_vehicle, "SleepingEntity");
        // Test that transforms are maintained regardless of LOD
        let world = app.world();
        assert!(world.get::<Transform>(high_detail_vehicle).is_some());
        assert!(world.get::<Transform>(medium_detail_vehicle).is_some());
        assert!(world.get::<Transform>(low_detail_vehicle).is_some());
    }
    fn test_vegetation_visual_quality_transitions() {
        // Create vegetation at different quality levels
        let full_detail_veg = create_test_vegetation_at_distance(&mut app, 25.0);
        let medium_detail_veg = create_test_vegetation_at_distance(&mut app, 100.0);
        let billboard_veg = create_test_vegetation_at_distance(&mut app, 200.0);
        let culled_veg = create_test_vegetation_at_distance(&mut app, 350.0);
        // Run vegetation systems
        // Test vegetation quality levels
        assert_vegetation_lod_level(&app, full_detail_veg, VegetationDetailLevel::Full);
        assert_vegetation_lod_level(&app, medium_detail_veg, VegetationDetailLevel::Medium);
        assert_vegetation_lod_level(&app, billboard_veg, VegetationDetailLevel::Billboard);
        assert_vegetation_lod_level(&app, culled_veg, VegetationDetailLevel::Culled);
        // Test visual quality components
        assert_has_component::<FullDetailVegetation>(&app, full_detail_veg, "FullDetailVegetation");
        assert_has_component::<BillboardVegetation>(&app, billboard_veg, "BillboardVegetation");
        assert_has_component::<CulledVegetation>(&app, culled_veg, "CulledVegetation");
        // Test that medium detail doesn't have specific quality markers
        assert_lacks_component::<FullDetailVegetation>(&app, medium_detail_veg, "FullDetailVegetation");
        assert_lacks_component::<BillboardVegetation>(&app, medium_detail_veg, "BillboardVegetation");
        assert_lacks_component::<CulledVegetation>(&app, medium_detail_veg, "CulledVegetation");
    fn test_lighting_system_integration() {
        // Create entities with different lighting requirements
        let close_vehicle = create_test_vehicle_at_distance(&mut app, 80.0);
        let far_vehicle = create_test_vehicle_at_distance(&mut app, 250.0);
        // Add lighting components for testing
        {
            let mut world = app.world_mut();
            
            // Add directional light
            world.spawn((
                DirectionalLight {
                    color: Color::srgb(1.0, 1.0, 0.9),
                    illuminance: 10000.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
            ));
            // Add point light near close vehicle
                PointLight {
                    color: Color::srgb(1.0, 0.8, 0.6),
                    intensity: 2000.0,
                Transform::from_xyz(85.0, 5.0, 0.0),
        }
        // Run lighting and LOD systems
        // Test that lighting affects LOD decisions appropriately
        assert_lod_level(&app, close_vehicle, LodLevel::High);
        assert_lod_level(&app, far_vehicle, LodLevel::Medium);
        // Test that entities maintain proper transforms for lighting
        let close_transform = world.get::<Transform>(close_vehicle).unwrap();
        let far_transform = world.get::<Transform>(far_vehicle).unwrap();
        assert!(close_transform.translation.is_finite());
        assert!(far_transform.translation.is_finite());
    fn test_shadow_casting_quality() {
        // Create entities at different distances for shadow testing
        let shadow_caster = create_test_building_at_distance(&mut app, 150.0);
        let shadow_receiver = create_test_building_at_distance(&mut app, 160.0);
        // Add shadow-casting light
                    color: Color::srgb(1.0, 1.0, 1.0),
                    illuminance: 15000.0,
                    shadow_depth_bias: 0.02,
                    shadow_normal_bias: 0.6,
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.3, 0.0)),
        // Run shadow systems
        // Test that entities are positioned correctly for shadow casting
        let caster_transform = world.get::<Transform>(shadow_caster).unwrap();
        let receiver_transform = world.get::<Transform>(shadow_receiver).unwrap();
        assert!(caster_transform.translation.is_finite());
        assert!(receiver_transform.translation.is_finite());
        // Test that both entities are visible (within shadow range)
        assert_visibility(&app, shadow_caster, Visibility::Visible);
        assert_visibility(&app, shadow_receiver, Visibility::Visible);
    fn test_material_system_quality() {
        // Create entities with different material requirements
        let metallic_vehicle = create_test_vehicle_at_distance(&mut app, 100.0);
        let building_facade = create_test_building_at_distance(&mut app, 200.0);
        // Run material systems
        // Test that entities maintain proper LOD for material quality
        assert_lod_level(&app, metallic_vehicle, LodLevel::High);
        assert_visibility(&app, building_facade, Visibility::Visible);
        // Test that high detail entities have proper components
        assert_has_component::<HighDetailVehicle>(&app, metallic_vehicle, "HighDetailVehicle");
        // Test that transforms are correct for material rendering
        let vehicle_transform = world.get::<Transform>(metallic_vehicle).unwrap();
        let building_transform = world.get::<Transform>(building_facade).unwrap();
        assert!(vehicle_transform.scale.is_finite());
        assert!(building_transform.scale.is_finite());
    fn test_visual_effects_system() {
        // Create entities that should have visual effects
        let vehicle_with_effects = create_test_vehicle_at_distance(&mut app, 75.0);
        // Add components for visual effects testing
            let mut entity_commands = world.entity_mut(vehicle_with_effects);
            // Add effect components that would be present in real game
            entity_commands.insert(Car::default());
            entity_commands.insert(ActiveEntity);
        // Run visual effects systems
        // Test that visual effects are applied correctly
        assert_lod_level(&app, vehicle_with_effects, LodLevel::High);
        assert_has_component::<HighDetailVehicle>(&app, vehicle_with_effects, "HighDetailVehicle");
        // Test that entity is visible and positioned correctly
        assert_visibility(&app, vehicle_with_effects, Visibility::Visible);
        let transform = world.get::<Transform>(vehicle_with_effects).unwrap();
        assert!(transform.translation.is_finite());
    fn test_render_quality_scaling() {
        // Create entities at various quality levels
        let entities = vec![
            (create_test_vehicle_at_distance(&mut app, 50.0), "high"),
            (create_test_vehicle_at_distance(&mut app, 175.0), "medium"),
            (create_test_vehicle_at_distance(&mut app, 350.0), "low"),
        ];
        // Run quality scaling systems
        // Test that quality scales appropriately with distance
        for (entity, quality) in entities {
            let world = app.world();
            let lod_level = world.get::<LodLevel>(entity).unwrap();
            match quality {
                "high" => {
                    assert_eq!(*lod_level, LodLevel::High);
                    assert!(world.get::<HighDetailVehicle>(entity).is_some());
                }
                "medium" => {
                    assert_eq!(*lod_level, LodLevel::Medium);
                    assert!(world.get::<HighDetailVehicle>(entity).is_none());
                "low" => {
                    assert_eq!(*lod_level, LodLevel::Sleep);
                    assert!(world.get::<SleepingEntity>(entity).is_some());
                _ => panic!("Unknown quality level"),
            }
    fn test_visual_consistency_across_frames() {
        // Create entity that should maintain visual consistency
        let consistent_entity = create_test_vehicle_at_distance(&mut app, 100.0);
        // Run multiple frames and check consistency
        let mut lod_levels = Vec::new();
        let mut visibilities = Vec::new();
        for _ in 0..10 {
            app.update();
            let lod_level = world.get::<LodLevel>(consistent_entity).unwrap();
            let visibility = world.get::<Visibility>(consistent_entity).unwrap();
            lod_levels.push(*lod_level);
            visibilities.push(*visibility);
        // Test that LOD level remains consistent
        assert!(lod_levels.iter().all(|&lod| lod == lod_levels[0]),
            "LOD level should remain consistent across frames");
        // Test that visibility remains consistent
        assert!(visibilities.iter().all(|&vis| vis == visibilities[0]),
            "Visibility should remain consistent across frames");
    fn test_quality_transition_smoothness() {
        // Create entity that will transition between quality levels
        let transitioning_entity = create_test_vehicle_at_distance(&mut app, 140.0);
        // Run initial frame
        run_render_simulation_frames(&mut app, 2);
        assert_lod_level(&app, transitioning_entity, LodLevel::High);
        // Move entity to trigger transition
            let mut transform = world.get_mut::<Transform>(transitioning_entity).unwrap();
            transform.translation.x = 180.0; // Move to medium range
        // Run transition frames
        run_render_simulation_frames(&mut app, 3);
        assert_lod_level(&app, transitioning_entity, LodLevel::Medium);
        // Move entity to trigger another transition
            transform.translation.x = 320.0; // Move to sleep range
        // Run final transition frames
        assert_lod_level(&app, transitioning_entity, LodLevel::Sleep);
        // Test that component transitions are clean
        assert_lacks_component::<HighDetailVehicle>(&app, transitioning_entity, "HighDetailVehicle");
        assert_has_component::<SleepingEntity>(&app, transitioning_entity, "SleepingEntity");
    fn test_render_queue_visual_impact() {
        // Create entities that will be in render queue
        let queue_entities: Vec<Entity> = (0..15)
            .map(|i| {
                let distance = 50.0 + (i as f32) * 20.0;
                create_test_vehicle_at_distance(&mut app, distance)
            })
            .collect();
        // Run render queue systems
        // Test that all entities in queue are rendered correctly
        for entity in queue_entities {
            let visibility = world.get::<Visibility>(entity).unwrap();
            // All entities should have valid LOD levels
            assert!(matches!(*lod_level, LodLevel::High | LodLevel::Medium | LodLevel::Sleep));
            // All entities should have valid visibility
            assert!(matches!(*visibility, Visibility::Visible | Visibility::Hidden));
    fn test_multi_entity_visual_quality() {
        // Create scene with multiple entity types
        let scene = TestSceneBuilder::new()
            .with_camera(&mut app)
            .with_vehicle_at_distance(&mut app, 60.0)
            .with_npc_at_distance(&mut app, 80.0)
            .with_building_at_distance(&mut app, 120.0)
            .with_vegetation_at_distance(&mut app, 90.0)
            .build();
        // Run all visual quality systems
        run_render_simulation_frames(&mut app, 8);
        // Test that each entity type has appropriate quality
        assert_lod_level(&app, scene.vehicle_entities[0], LodLevel::High);
        assert_lod_level(&app, scene.npc_entities[0], LodLevel::High);
        assert_visibility(&app, scene.building_entities[0], Visibility::Visible);
        assert_vegetation_lod_level(&app, scene.vegetation_entities[0], VegetationDetailLevel::Medium);
        // Test that quality components are correctly applied
        assert_has_component::<HighDetailVehicle>(&app, scene.vehicle_entities[0], "HighDetailVehicle");
        assert_has_component::<HighDetailNPC>(&app, scene.npc_entities[0], "HighDetailNPC");
        // Test that transforms are maintained for all entity types
        assert!(world.get::<Transform>(scene.vehicle_entities[0]).unwrap().translation.is_finite());
        assert!(world.get::<Transform>(scene.npc_entities[0]).unwrap().translation.is_finite());
        assert!(world.get::<Transform>(scene.building_entities[0]).unwrap().translation.is_finite());
        assert!(world.get::<Transform>(scene.vegetation_entities[0]).unwrap().translation.is_finite());
}
