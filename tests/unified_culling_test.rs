use bevy::prelude::*;
use gta_game::systems::world::unified_distance_culling::*;
use gta_game::systems::distance_cache::*;
use gta_game::components::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(DistanceCache::new());
        
        // Add required resources manually
        app.insert_resource(FrameCounter::default())
            .insert_resource(PerformanceStats {
                entity_count: 0,
                culled_entities: 0,
            });
        app
    }

    #[test]
    fn test_unified_cullable_vehicle_config() {
        let cullable = UnifiedCullable::vehicle();
        
        assert_eq!(cullable.config.entity_type, "Vehicle");
        assert_eq!(cullable.config.lod_distances, vec![50.0, 150.0, 300.0]);
        assert_eq!(cullable.config.cull_distance, 500.0);
        assert_eq!(cullable.config.hysteresis, 5.0);
        assert_eq!(cullable.config.update_interval, 0.5);
    }

    #[test]
    fn test_unified_cullable_npc_config() {
        let cullable = UnifiedCullable::npc();
        
        assert_eq!(cullable.config.entity_type, "NPC");
        assert_eq!(cullable.config.lod_distances, vec![25.0, 75.0, 100.0]);
        assert_eq!(cullable.config.cull_distance, 150.0);
        assert_eq!(cullable.config.hysteresis, 3.0);
        assert_eq!(cullable.config.update_interval, 0.3);
    }

    #[test]
    fn test_distance_culling_config_lod_levels() {
        let config = DistanceCullingConfig::vegetation();
        
        // Test LOD level calculation
        assert_eq!(config.get_lod_level(25.0), 0); // Full LOD
        assert_eq!(config.get_lod_level(75.0), 1); // Medium LOD
        assert_eq!(config.get_lod_level(200.0), 2); // Billboard LOD
        assert_eq!(config.get_lod_level(500.0), 3); // Beyond all LOD levels
    }

    #[test]
    fn test_distance_culling_config_culling() {
        let config = DistanceCullingConfig::npc();
        
        // Test culling decisions
        assert!(!config.should_cull(100.0)); // Within range
        assert!(!config.should_cull(150.0)); // At boundary
        assert!(config.should_cull(200.0)); // Beyond cull distance + hysteresis
    }

    #[test]
    fn test_unified_cullable_update() {
        let mut cullable = UnifiedCullable::vehicle();
        
        // Test initial state
        assert_eq!(cullable.current_lod, 0);
        assert!(!cullable.is_culled);
        
        // Test LOD update
        let changed = cullable.update(200.0, 1.0);
        assert!(changed); // Should detect change
        assert_eq!(cullable.current_lod, 1); // Should be medium LOD
        assert!(!cullable.is_culled);
        
        // Test culling update
        let changed = cullable.update(600.0, 2.0);
        assert!(changed); // Should detect culling
        assert!(cullable.is_culled);
    }

    #[test]
    fn test_unified_cullable_needs_update() {
        let mut cullable = UnifiedCullable::npc();
        cullable.last_update = 0.0;
        cullable.last_distance = 50.0;
        
        // Test time-based update need
        assert!(cullable.needs_update(1.0, 50.0)); // Time elapsed > interval
        
        // Test distance-based update need
        assert!(cullable.needs_update(0.1, 60.0)); // Distance changed > hysteresis
        
        // Test no update needed
        assert!(!cullable.needs_update(0.1, 51.0)); // Small time, small distance change
    }

    #[test]
    fn test_hysteresis_prevents_flickering() {
        let config = DistanceCullingConfig::vehicle();
        
        // Test hysteresis at LOD boundary (150m)
        assert_eq!(config.get_lod_level(148.0), 0); // Full LOD
        assert_eq!(config.get_lod_level(152.0), 0); // Still full due to hysteresis
        assert_eq!(config.get_lod_level(157.0), 1); // Now medium LOD
        
        // Test hysteresis at cull boundary (500m)
        assert!(!config.should_cull(503.0)); // Not culled due to hysteresis
        assert!(config.should_cull(507.0)); // Now culled
    }

    #[test] 
    fn test_custom_config_creation() {
        let custom_config = DistanceCullingConfig {
            lod_distances: vec![30.0, 100.0],
            cull_distance: 200.0,
            hysteresis: 2.0,
            update_interval: 0.1,
            entity_type: "CustomType",
        };
        
        let cullable = UnifiedCullable::new(custom_config);
        
        assert_eq!(cullable.config.entity_type, "CustomType");
        assert_eq!(cullable.config.lod_distances.len(), 2);
        assert_eq!(cullable.config.cull_distance, 200.0);
    }

    #[test]
    fn test_lod_update_components() {
        // Test VehicleLODUpdate
        let vehicle_update = VehicleLODUpdate { new_lod: VehicleLOD::Medium };
        assert_eq!(vehicle_update.new_lod, VehicleLOD::Medium);
        
        // Test NPCLODUpdate
        let npc_update = NPCLODUpdate { new_lod: NPCLOD::Low };
        assert_eq!(npc_update.new_lod, NPCLOD::Low);
        
        // Test VegetationLODUpdate
        let veg_update = VegetationLODUpdate { 
            new_detail_level: VegetationDetailLevel::Billboard,
            distance: 200.0,
        };
        assert_eq!(veg_update.new_detail_level, VegetationDetailLevel::Billboard);
        assert_eq!(veg_update.distance, 200.0);
    }

    #[test]
    fn test_app_integration() {
        let mut app = setup_test_app();
        
        // Spawn test entities
        let entity = app.world_mut().spawn((
            UnifiedCullable::vehicle(),
            Transform::default(),
            Visibility::default(),
        )).id();
        
        // Spawn active entity
        app.world_mut().spawn((
            ActiveEntity,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        
        // Add the unified culling system manually for testing
        app.add_systems(Update, new_unified_distance_culling_system);
        
        // Run one update cycle
        app.update();
        
        // Verify entity exists and has expected components
        assert!(app.world().get::<UnifiedCullable>(entity).is_some());
        assert!(app.world().get::<Transform>(entity).is_some());
        assert!(app.world().get::<Visibility>(entity).is_some());
    }

    #[test]
    fn test_performance_scaling() {
        let mut app = setup_test_app();
        
        // Spawn active entity
        app.world_mut().spawn((
            ActiveEntity,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        
        // Spawn many entities to test performance
        for i in 0..1000 {
            let distance = i as f32 * 0.5; // Spread entities over distance
            app.world_mut().spawn((
                UnifiedCullable::vehicle(),
                Transform::from_xyz(distance, 0.0, 0.0),
                Visibility::default(),
            ));
        }
        
        // Add the unified culling system manually for testing
        app.add_systems(Update, new_unified_distance_culling_system);
        
        // Run several update cycles
        for _ in 0..10 {
            app.update();
        }
        
        // Test should complete without panicking or timeout
        // In real testing, you would measure frame times here
    }
}
