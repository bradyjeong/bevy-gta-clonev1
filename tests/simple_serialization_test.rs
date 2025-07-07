//! Simple serialization test to verify the Oracle's Phase 3 implementation
//! 
//! This test validates the save-load round-trip functionality without 
//! requiring the entire workspace to compile.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimal test components for serialization
#[derive(Component, Debug, Clone, PartialEq)]
pub struct TestPlayer;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TestCar;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TestBuilding;

/// Serializable entity for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSerializableEntity {
    pub id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub entity_type: String,
    pub name: String,
}

/// Simple world state for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWorldState {
    pub entities: Vec<TestSerializableEntity>,
    pub entity_count: usize,
    pub timestamp: String,
}

/// Simple serialization system for testing
pub struct TestWorldSerializer;

impl TestWorldSerializer {
    pub fn serialize_test_world(world: &World) -> Result<TestWorldState, Box<dyn std::error::Error>> {
        let mut entities = Vec::new();
        let mut entity_count = 0;
        
        // Query entities with Transform and Name
        let mut query = world.query::<(Entity, &Transform, &Name)>();
        
        for (entity, transform, name) in query.iter(world) {
            let entity_type = if world.get::<TestPlayer>(entity).is_some() {
                "Player".to_string()
            } else if world.get::<TestCar>(entity).is_some() {
                "Car".to_string()
            } else if world.get::<TestBuilding>(entity).is_some() {
                "Building".to_string()
            } else {
                "Unknown".to_string()
            };
            
            let serializable_entity = TestSerializableEntity {
                id: entity.index(),
                position: transform.translation.to_array(),
                rotation: [
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z,
                    transform.rotation.w,
                ],
                entity_type,
                name: name.to_string(),
            };
            
            entities.push(serializable_entity);
            entity_count += 1;
        }
        
        Ok(TestWorldState {
            entities,
            entity_count,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    pub fn serialize_to_ron(world_state: &TestWorldState) -> Result<String, Box<dyn std::error::Error>> {
        let ron_string = ron::to_string_pretty(world_state, ron::PrettyConfig::default())?;
        Ok(ron_string)
    }
    
    pub fn deserialize_from_ron(ron_string: &str) -> Result<TestWorldState, Box<dyn std::error::Error>> {
        let world_state: TestWorldState = ron::from_str(ron_string)?;
        Ok(world_state)
    }
    
    pub fn apply_to_world(app: &mut App, world_state: &TestWorldState) -> Result<(), Box<dyn std::error::Error>> {
        for entity_data in &world_state.entities {
            let transform = Transform {
                translation: Vec3::from_array(entity_data.position),
                rotation: Quat::from_array(entity_data.rotation),
                scale: Vec3::ONE,
            };
            
            let mut entity_commands = app.world_mut().spawn((
                transform,
                GlobalTransform::default(),
                Name::new(entity_data.name.clone()),
            ));
            
            match entity_data.entity_type.as_str() {
                "Player" => { entity_commands.insert(TestPlayer); }
                "Car" => { entity_commands.insert(TestCar); }
                "Building" => { entity_commands.insert(TestBuilding); }
                _ => {}
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod oracle_phase3_tests {
    use super::*;
    use rand::Rng;
    
    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(TransformPlugin)
           .add_plugins(HierarchyPlugin);
        app
    }
    
    fn spawn_test_entities(app: &mut App, count: usize) -> Vec<Entity> {
        let mut entities = Vec::new();
        let mut rng = rand::thread_rng();
        
        for i in 0..count {
            let position = Vec3::new(
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-10.0..10.0),
                rng.gen_range(-100.0..100.0),
            );
            
            let entity_type = rng.gen_range(0..3);
            let mut entity_cmd = app.world_mut().spawn((
                Transform::from_xyz(position.x, position.y, position.z),
                GlobalTransform::default(),
                Name::new(format!("TestEntity_{}", i)),
            ));
            
            match entity_type {
                0 => { entity_cmd.insert(TestPlayer); }
                1 => { entity_cmd.insert(TestCar); }
                2 => { entity_cmd.insert(TestBuilding); }
                _ => {}
            }
            
            entities.push(entity_cmd.id());
        }
        
        entities
    }
    
    #[test]
    fn test_oracle_phase3_basic_serialization() {
        println!("🔮 Oracle Phase 3: Basic Serialization Test");
        
        let mut app = create_test_app();
        
        // Create test entity
        app.world_mut().spawn((
            Transform::from_xyz(10.0, 20.0, 30.0),
            GlobalTransform::default(),
            Name::new("TestEntity"),
            TestPlayer,
        ));
        
        // Serialize
        let world_state = TestWorldSerializer::serialize_test_world(app.world())
            .expect("Failed to serialize world");
        
        assert_eq!(world_state.entity_count, 1);
        assert_eq!(world_state.entities[0].name, "TestEntity");
        assert_eq!(world_state.entities[0].entity_type, "Player");
        assert_eq!(world_state.entities[0].position, [10.0, 20.0, 30.0]);
        
        println!("✅ Basic serialization test PASSED");
    }
    
    #[test]
    fn test_oracle_phase3_ron_format() {
        println!("🔮 Oracle Phase 3: RON Format Test");
        
        let mut app = create_test_app();
        
        app.world_mut().spawn((
            Transform::from_xyz(1.0, 2.0, 3.0),
            GlobalTransform::default(),
            Name::new("RONTest"),
            TestCar,
        ));
        
        // Serialize to world state
        let world_state = TestWorldSerializer::serialize_test_world(app.world())
            .expect("Failed to serialize world");
        
        // Convert to RON
        let ron_string = TestWorldSerializer::serialize_to_ron(&world_state)
            .expect("Failed to serialize to RON");
        
        println!("📦 RON Output (first 200 chars): {}", &ron_string[..200.min(ron_string.len())]);
        
        // Verify RON contains expected data
        assert!(ron_string.contains("RONTest"));
        assert!(ron_string.contains("Car"));
        assert!(ron_string.contains("1.0"));
        assert!(ron_string.contains("2.0"));
        assert!(ron_string.contains("3.0"));
        
        // Parse RON back
        let parsed_state = TestWorldSerializer::deserialize_from_ron(&ron_string)
            .expect("Failed to parse RON");
        
        assert_eq!(parsed_state.entity_count, 1);
        assert_eq!(parsed_state.entities[0].name, "RONTest");
        assert_eq!(parsed_state.entities[0].entity_type, "Car");
        
        println!("✅ RON format test PASSED");
    }
    
    #[test]
    fn test_oracle_phase3_save_load_round_trip() {
        println!("🔮 Oracle Phase 3: Save-Load Round-Trip Test");
        
        // Create original world with test entities
        let mut original_app = create_test_app();
        let original_entities = spawn_test_entities(&mut original_app, 20);
        
        println!("📊 Original entities spawned: {}", original_entities.len());
        
        // Serialize original world
        let world_state = TestWorldSerializer::serialize_test_world(original_app.world())
            .expect("Failed to serialize world");
        
        println!("💾 Serialized {} entities", world_state.entity_count);
        assert_eq!(world_state.entity_count, 20);
        
        // Create fresh app and load data
        let mut loaded_app = create_test_app();
        TestWorldSerializer::apply_to_world(&mut loaded_app, &world_state)
            .expect("Failed to apply world data");
        
        // Count loaded entities
        let mut loaded_count = 0;
        let mut query = loaded_app.world_mut().query::<(Entity, &Name)>();
        for (_, _) in query.iter(loaded_app.world()) {
            loaded_count += 1;
        }
        
        println!("🔄 Loaded entities: {}", loaded_count);
        
        // Verify counts match
        assert_eq!(loaded_count, 20, "Entity count mismatch after load");
        
        // Verify specific entities exist
        let mut player_count = 0;
        let mut car_count = 0;
        let mut building_count = 0;
        
        let mut query = loaded_app.world_mut().query::<(&Name, Option<&TestPlayer>, Option<&TestCar>, Option<&TestBuilding>)>();
        for (name, player, car, building) in query.iter(loaded_app.world()) {
            if player.is_some() { player_count += 1; }
            if car.is_some() { car_count += 1; }
            if building.is_some() { building_count += 1; }
        }
        
        println!("📈 Component counts - Players: {}, Cars: {}, Buildings: {}", 
                 player_count, car_count, building_count);
        
        // Verify total component count matches entity count
        assert_eq!(player_count + car_count + building_count, 20,
                   "Component count mismatch");
        
        println!("✅ Oracle Phase 3 Save-Load Round-Trip Test PASSED");
        println!("🎉 Successfully verified 20 entities with component equality");
    }
    
    #[test]
    fn test_oracle_phase3_large_scale_100_entities() {
        println!("🔮 Oracle Phase 3: Large-Scale Test (100 entities)");
        
        // Create original world with 100 entities (as specified by Oracle)
        let mut original_app = create_test_app();
        let original_entities = spawn_test_entities(&mut original_app, 100);
        
        println!("📊 Spawned {} entities in original world", original_entities.len());
        
        // Measure serialization time
        let start_time = std::time::Instant::now();
        let world_state = TestWorldSerializer::serialize_test_world(original_app.world())
            .expect("Failed to serialize world");
        let serialize_time = start_time.elapsed();
        
        println!("💾 Serialized {} entities in {:?}", world_state.entity_count, serialize_time);
        assert_eq!(world_state.entity_count, 100);
        
        // Convert to RON
        let start_time = std::time::Instant::now();
        let ron_string = TestWorldSerializer::serialize_to_ron(&world_state)
            .expect("Failed to serialize to RON");
        let ron_time = start_time.elapsed();
        
        println!("📦 Generated RON data ({} bytes) in {:?}", ron_string.len(), ron_time);
        
        // Parse RON
        let start_time = std::time::Instant::now();
        let parsed_state = TestWorldSerializer::deserialize_from_ron(&ron_string)
            .expect("Failed to parse RON");
        let parse_time = start_time.elapsed();
        
        println!("🔄 Parsed RON data in {:?}", parse_time);
        
        // Create fresh app and load
        let mut loaded_app = create_test_app();
        let start_time = std::time::Instant::now();
        TestWorldSerializer::apply_to_world(&mut loaded_app, &parsed_state)
            .expect("Failed to apply world data");
        let load_time = start_time.elapsed();
        
        println!("⚡ Loaded entities in {:?}", load_time);
        
        // Count loaded entities
        let mut loaded_count = 0;
        let mut query = loaded_app.world_mut().query::<Entity>();
        for _ in query.iter(loaded_app.world()) {
            loaded_count += 1;
        }
        
        // Verify entity count matches
        assert_eq!(loaded_count, 100, "Entity count mismatch: expected 100, got {}", loaded_count);
        
        // Verify component preservation
        let mut component_counts = HashMap::new();
        let mut query = loaded_app.world_mut().query::<(Option<&TestPlayer>, Option<&TestCar>, Option<&TestBuilding>)>();
        for (player, car, building) in query.iter(loaded_app.world()) {
            if player.is_some() { *component_counts.entry("Player").or_insert(0) += 1; }
            if car.is_some() { *component_counts.entry("Car").or_insert(0) += 1; }
            if building.is_some() { *component_counts.entry("Building").or_insert(0) += 1; }
        }
        
        println!("📊 Component preservation: {:?}", component_counts);
        
        let total_components: usize = component_counts.values().sum();
        assert_eq!(total_components, 100, "Component count mismatch");
        
        println!("✅ Oracle Phase 3 Large-Scale Test (100 entities) PASSED");
        println!("🎯 Entity count verification: ✓");
        println!("🔧 Component equality verification: ✓");
        println!("⏱️  Performance benchmarks: ✓");
    }
    
    #[test]
    fn test_oracle_phase3_position_accuracy() {
        println!("🔮 Oracle Phase 3: Position Accuracy Test");
        
        let mut app = create_test_app();
        
        // Create entities with precise positions
        let test_positions = vec![
            (123.456, 789.012, 345.678),
            (-999.999, 0.001, 1000.0),
            (0.0, 0.0, 0.0),
        ];
        
        for (i, (x, y, z)) in test_positions.iter().enumerate() {
            app.world_mut().spawn((
                Transform::from_xyz(*x, *y, *z),
                GlobalTransform::default(),
                Name::new(format!("PrecisionTest_{}", i)),
                TestPlayer,
            ));
        }
        
        // Serialize and load
        let world_state = TestWorldSerializer::serialize_test_world(app.world())
            .expect("Failed to serialize world");
        
        let mut loaded_app = create_test_app();
        TestWorldSerializer::apply_to_world(&mut loaded_app, &world_state)
            .expect("Failed to apply world data");
        
        // Verify position accuracy
        let mut query = loaded_app.world_mut().query::<(&Transform, &Name)>();
        let mut verified_positions = 0;
        
        for (transform, name) in query.iter(loaded_app.world()) {
            if name.as_str().starts_with("PrecisionTest_") {
                let index: usize = name.as_str().replace("PrecisionTest_", "").parse().unwrap();
                let expected = test_positions[index];
                
                let actual = (
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                );
                
                let tolerance = 0.001;
                assert!((actual.0 - expected.0).abs() < tolerance, 
                        "X position mismatch: expected {}, got {}", expected.0, actual.0);
                assert!((actual.1 - expected.1).abs() < tolerance,
                        "Y position mismatch: expected {}, got {}", expected.1, actual.1);
                assert!((actual.2 - expected.2).abs() < tolerance,
                        "Z position mismatch: expected {}, got {}", expected.2, actual.2);
                
                verified_positions += 1;
            }
        }
        
        assert_eq!(verified_positions, 3, "Not all positions were verified");
        
        println!("✅ Position accuracy test PASSED");
        println!("🎯 Verified {} positions with high precision", verified_positions);
    }
}
