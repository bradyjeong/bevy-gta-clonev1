//! Save-load round-trip tests following Oracle's Phase 3 specifications
//! 
//! Tests comprehensive world serialization with 100+ random entities,
//! verifying entity count and component equality across save-load cycles.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use gta_game::components::*;
use gta_game::bundles::*;
use gta_game::factories::entity_factory_unified::*;
use gta_game::serialization::WorldSerializer;
use rand::Rng;
use std::collections::HashMap;

#[cfg(test)]
mod save_load_tests {
    use super::*;
    
    /// Create a test app with necessary plugins
    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(TransformPlugin)
           .add_plugins(HierarchyPlugin)
           .add_plugins(AssetPlugin::default())
           .init_resource::<CullingSettings>()
           .init_resource::<PerformanceStats>()
           .init_resource::<EntityLimits>()
           .init_resource::<MeshCache>();
        app
    }
    
    /// Generate random entities with various components
    fn spawn_random_entities(app: &mut App, count: usize) -> Vec<Entity> {
        let mut entities = Vec::new();
        let mut rng = rand::thread_rng();
        
        for i in 0..count {
            let entity_type = rng.gen_range(0..7);
            let entity_id = match entity_type {
                0 => spawn_random_vehicle(app, i),
                1 => spawn_random_npc(app, i),
                2 => spawn_random_building(app, i),
                3 => spawn_random_player(app, i),
                4 => spawn_random_road_entity(app, i),
                5 => spawn_random_terrain(app, i),
                6 => spawn_random_landmark(app, i),
                _ => spawn_random_vehicle(app, i),
            };
            entities.push(entity_id);
        }
        
        entities
    }
    
    fn spawn_random_vehicle(app: &mut App, index: usize) -> Entity {
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range(-1000.0..1000.0),
            rng.gen_range(0.0..100.0),
            rng.gen_range(-1000.0..1000.0),
        );
        
        let vehicle_type = match rng.gen_range(0..4) {
            0 => VehicleType::Car,
            1 => VehicleType::Helicopter,
            2 => VehicleType::Airplane,
            _ => VehicleType::Car,
        };
        
        let vehicle_state = match rng.gen_range(0..3) {
            0 => VehicleState::Parked,
            1 => VehicleState::Driving,
            _ => VehicleState::Stopped,
        };
        
        let entity_id = app.world_mut().spawn((
            Transform::from_xyz(position.x, position.y, position.z),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Car,
            vehicle_type,
            vehicle_state,
            UnifiedCullable::vehicle(),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 1.0, 4.0),
            Velocity::default(),
            CollisionGroups::new(Group::GROUP_1, Group::ALL),
            ColliderMassProperties::Density(1200.0),
            Name::new(format!("Vehicle_{}", index)),
        )).id();
        
        // Randomly add super car marker
        if rng.gen_bool(0.3) {
            app.world_mut().entity_mut(entity_id).insert(SuperCar);
        }
        
        entity_id
    }
    
    fn spawn_random_npc(app: &mut App, index: usize) -> Entity {
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range(-1000.0..1000.0),
            rng.gen_range(0.0..10.0),
            rng.gen_range(-1000.0..1000.0),
        );
        
        let npc_type = match rng.gen_range(0..4) {
            0 => NPCType::Civilian,
            1 => NPCType::Police,
            2 => NPCType::Paramedic,
            _ => NPCType::Civilian,
        };
        
        let npc_gender = match rng.gen_range(0..2) {
            0 => NPCGender::Male,
            _ => NPCGender::Female,
        };
        
        let behavior_type = match rng.gen_range(0..3) {
            0 => NPCBehaviorType::Walking,
            1 => NPCBehaviorType::Standing,
            _ => NPCBehaviorType::Sitting,
        };
        
        app.world_mut().spawn((
            Transform::from_xyz(position.x, position.y, position.z),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            NPC,
            npc_type,
            npc_gender,
            NPCBehaviorComponent {
                behavior_type,
                last_update: 0.0,
            },
            UnifiedCullable::npc(),
            RigidBody::Dynamic,
            Collider::capsule_y(0.9, 0.3),
            Velocity::default(),
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
            ColliderMassProperties::Density(70.0),
            Name::new(format!("NPC_{}", index)),
        )).id()
    }
    
    fn spawn_random_building(app: &mut App, index: usize) -> Entity {
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range(-1000.0..1000.0),
            0.0,
            rng.gen_range(-1000.0..1000.0),
        );
        
        let building_type = match rng.gen_range(0..4) {
            0 => BuildingType::Residential,
            1 => BuildingType::Commercial,
            2 => BuildingType::Industrial,
            _ => BuildingType::Residential,
        };
        
        let height = rng.gen_range(10.0..200.0);
        let width = rng.gen_range(20.0..100.0);
        let depth = rng.gen_range(20.0..100.0);
        
        app.world_mut().spawn((
            Transform::from_xyz(position.x, height / 2.0, position.z),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Building,
            building_type,
            UnifiedCullable::building(),
            RigidBody::Fixed,
            Collider::cuboid(width / 2.0, height / 2.0, depth / 2.0),
            CollisionGroups::new(Group::GROUP_3, Group::ALL),
            Name::new(format!("Building_{}", index)),
        )).id()
    }
    
    fn spawn_random_player(app: &mut App, index: usize) -> Entity {
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range(-100.0..100.0),
            rng.gen_range(0.0..10.0),
            rng.gen_range(-100.0..100.0),
        );
        
        let entity_id = app.world_mut().spawn((
            Transform::from_xyz(position.x, position.y, position.z),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Player,
            ActiveEntity,
            HumanMovement::default(),
            HumanAnimation::default(),
            HumanBehavior::default(),
            UnifiedCullable::player(),
            RigidBody::Dynamic,
            Collider::capsule_y(0.9, 0.3),
            Velocity::default(),
            CollisionGroups::new(Group::GROUP_4, Group::ALL),
            ColliderMassProperties::Density(70.0),
            Name::new(format!("Player_{}", index)),
        )).id();
        
        // Randomly add in-car state
        if rng.gen_bool(0.2) {
            app.world_mut().entity_mut(entity_id).insert(InCar);
        }
        
        entity_id
    }
    
    fn spawn_random_road_entity(app: &mut App, index: usize) -> Entity {
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range(-1000.0..1000.0),
            0.0,
            rng.gen_range(-1000.0..1000.0),
        );
        
        let is_intersection = rng.gen_bool(0.3);
        let mut entity_cmd = app.world_mut().spawn((
            Transform::from_xyz(position.x, position.y, position.z),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            UnifiedCullable::road(),
            RigidBody::Fixed,
            Collider::cuboid(10.0, 0.1, 10.0),
            CollisionGroups::new(Group::GROUP_5, Group::ALL),
            Name::new(format!("Road_{}", index)),
        ));
        
        if is_intersection {
            entity_cmd.insert(IntersectionEntity);
        } else {
            entity_cmd.insert(RoadEntity);
        }
        
        entity_cmd.id()
    }
    
    fn spawn_random_terrain(app: &mut App, index: usize) -> Entity {
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range(-2000.0..2000.0),
            rng.gen_range(-10.0..50.0),
            rng.gen_range(-2000.0..2000.0),
        );
        
        let size = rng.gen_range(50.0..500.0);
        let height = rng.gen_range(1.0..20.0);
        
        let is_dynamic = rng.gen_bool(0.4);
        let mut entity_cmd = app.world_mut().spawn((
            Transform::from_xyz(position.x, position.y, position.z),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            UnifiedCullable::terrain(),
            RigidBody::Fixed,
            Collider::cuboid(size / 2.0, height / 2.0, size / 2.0),
            CollisionGroups::new(Group::GROUP_6, Group::ALL),
            Name::new(format!("Terrain_{}", index)),
        ));
        
        if is_dynamic {
            entity_cmd.insert(DynamicTerrain);
            entity_cmd.insert(DynamicContent);
            entity_cmd.insert(ContentType::Terrain);
        }
        
        entity_cmd.id()
    }
    
    fn spawn_random_landmark(app: &mut App, index: usize) -> Entity {
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range(-500.0..500.0),
            rng.gen_range(0.0..100.0),
            rng.gen_range(-500.0..500.0),
        );
        
        let size = rng.gen_range(5.0..50.0);
        let height = rng.gen_range(10.0..100.0);
        
        app.world_mut().spawn((
            Transform::from_xyz(position.x, position.y, position.z),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Landmark,
            Building,
            BuildingType::Landmark,
            UnifiedCullable::landmark(),
            RigidBody::Fixed,
            Collider::cuboid(size / 2.0, height / 2.0, size / 2.0),
            CollisionGroups::new(Group::GROUP_7, Group::ALL),
            PerformanceCritical,
            Name::new(format!("Landmark_{}", index)),
        )).id()
    }
    
    /// Count entities by type in a world
    fn count_entities_by_type(world: &World) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        // Count vehicles
        let mut query = world.query::<&Car>();
        counts.insert("Car".to_string(), query.iter(world).count());
        
        // Count NPCs
        let mut query = world.query::<&NPC>();
        counts.insert("NPC".to_string(), query.iter(world).count());
        
        // Count buildings
        let mut query = world.query::<&Building>();
        counts.insert("Building".to_string(), query.iter(world).count());
        
        // Count players
        let mut query = world.query::<&Player>();
        counts.insert("Player".to_string(), query.iter(world).count());
        
        // Count roads
        let mut query = world.query::<&RoadEntity>();
        counts.insert("Road".to_string(), query.iter(world).count());
        
        // Count intersections
        let mut query = world.query::<&IntersectionEntity>();
        counts.insert("Intersection".to_string(), query.iter(world).count());
        
        // Count terrain
        let mut query = world.query::<&DynamicTerrain>();
        counts.insert("Terrain".to_string(), query.iter(world).count());
        
        // Count landmarks
        let mut query = world.query::<&Landmark>();
        counts.insert("Landmark".to_string(), query.iter(world).count());
        
        counts
    }
    
    /// Verify component equality between two worlds
    fn verify_component_equality(original: &World, loaded: &World) -> bool {
        // Check transform equality
        let mut original_transforms = Vec::new();
        let mut loaded_transforms = Vec::new();
        
        {
            let mut query = original.query::<(&Transform, &Name)>();
            for (transform, name) in query.iter(original) {
                original_transforms.push((name.as_str().to_string(), transform.translation));
            }
        }
        
        {
            let mut query = loaded.query::<(&Transform, &Name)>();
            for (transform, name) in query.iter(loaded) {
                loaded_transforms.push((name.as_str().to_string(), transform.translation));
            }
        }
        
        // Sort by name for comparison
        original_transforms.sort_by(|a, b| a.0.cmp(&b.0));
        loaded_transforms.sort_by(|a, b| a.0.cmp(&b.0));
        
        if original_transforms.len() != loaded_transforms.len() {
            return false;
        }
        
        for (orig, loaded) in original_transforms.iter().zip(loaded_transforms.iter()) {
            if orig.0 != loaded.0 {
                return false;
            }
            // Check position equality with tolerance
            let diff = (orig.1 - loaded.1).length();
            if diff > 0.01 {
                return false;
            }
        }
        
        true
    }
    
    #[test]
    fn test_oracle_phase3_save_load_round_trip_100_entities() {
        println!("🔮 Oracle Phase 3: Save-Load Round-Trip Test with 100 Random Entities");
        
        // Create original world with 100 random entities
        let mut original_app = create_test_app();
        let original_entities = spawn_random_entities(&mut original_app, 100);
        
        println!("✅ Spawned {} entities in original world", original_entities.len());
        
        // Count entities by type in original world
        let original_counts = count_entities_by_type(original_app.world());
        println!("📊 Original world composition: {:?}", original_counts);
        
        // Get total entity count
        let mut total_entities = 0;
        let mut query = original_app.world_mut().query::<Entity>();
        for _ in query.iter(original_app.world()) {
            total_entities += 1;
        }
        println!("🎯 Total entities in original world: {}", total_entities);
        
        // Serialize world to RON
        let world_data = WorldSerializer::extract_world_data(original_app.world())
            .expect("Failed to serialize world");
        println!("💾 Serialized world data: {} entities", world_data.entity_count);
        
        // Verify serialization captured all entities
        assert_eq!(world_data.entity_count, total_entities);
        
        // Create fresh app and load world data
        let mut loaded_app = create_test_app();
        WorldSerializer::apply_world_data(&mut loaded_app, &world_data)
            .expect("Failed to load world data");
        
        // Count entities in loaded world
        let mut loaded_entity_count = 0;
        let mut query = loaded_app.world_mut().query::<Entity>();
        for _ in query.iter(loaded_app.world()) {
            loaded_entity_count += 1;
        }
        println!("🔄 Loaded entities: {}", loaded_entity_count);
        
        // Verify entity count matches
        assert_eq!(loaded_entity_count, total_entities, "Entity count mismatch after load");
        
        // Count entities by type in loaded world
        let loaded_counts = count_entities_by_type(loaded_app.world());
        println!("📊 Loaded world composition: {:?}", loaded_counts);
        
        // Verify entity type counts match
        for (entity_type, original_count) in &original_counts {
            let loaded_count = loaded_counts.get(entity_type).unwrap_or(&0);
            assert_eq!(
                original_count, loaded_count,
                "Entity type {} count mismatch: original={}, loaded={}",
                entity_type, original_count, loaded_count
            );
        }
        
        // Verify component equality
        assert!(
            verify_component_equality(original_app.world(), loaded_app.world()),
            "Component equality verification failed"
        );
        
        println!("✅ Oracle Phase 3 Save-Load Round-Trip Test PASSED");
        println!("🎉 Successfully verified {} entities with component equality", total_entities);
    }
    
    #[test]
    fn test_complex_hierarchy_preservation() {
        println!("🏗️ Testing hierarchy preservation in save-load");
        
        let mut original_app = create_test_app();
        
        // Create parent-child hierarchy
        let parent = original_app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Building,
            BuildingType::Commercial,
            UnifiedCullable::building(),
            Name::new("ParentBuilding"),
        )).id();
        
        let child1 = original_app.world_mut().spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            Landmark,
            UnifiedCullable::landmark(),
            Name::new("Child1"),
        )).id();
        
        let child2 = original_app.world_mut().spawn((
            Transform::from_xyz(-5.0, 0.0, 0.0),
            Landmark,
            UnifiedCullable::landmark(),
            Name::new("Child2"),
        )).id();
        
        // Set up hierarchy
        original_app.world_mut().entity_mut(child1).insert(Parent(parent));
        original_app.world_mut().entity_mut(child2).insert(Parent(parent));
        original_app.world_mut().entity_mut(parent).insert(Children::from_slice(&[child1, child2]));
        
        // Serialize and load
        let world_data = WorldSerializer::extract_world_data(original_app.world())
            .expect("Failed to serialize world");
        
        let mut loaded_app = create_test_app();
        WorldSerializer::apply_world_data(&mut loaded_app, &world_data)
            .expect("Failed to load world data");
        
        // Verify hierarchy
        let mut parent_found = false;
        let mut children_count = 0;
        
        let mut query = loaded_app.world_mut().query::<(&Name, Option<&Parent>, Option<&Children>)>();
        for (name, parent, children) in query.iter(loaded_app.world()) {
            if name.as_str() == "ParentBuilding" {
                parent_found = true;
                if let Some(children) = children {
                    children_count = children.len();
                }
            }
        }
        
        assert!(parent_found, "Parent entity not found");
        assert_eq!(children_count, 2, "Wrong number of children");
        
        println!("✅ Hierarchy preservation test PASSED");
    }
    
    #[test]
    fn test_large_scale_serialization() {
        println!("🚀 Testing large-scale serialization (500 entities)");
        
        let mut original_app = create_test_app();
        let original_entities = spawn_random_entities(&mut original_app, 500);
        
        // Serialize to RON string
        let start_time = std::time::Instant::now();
        let ron_string = WorldSerializer::serialize_world_to_ron(original_app.world())
            .expect("Failed to serialize world");
        let serialize_time = start_time.elapsed();
        
        println!("📦 Serialized {} entities in {:?}", original_entities.len(), serialize_time);
        println!("📏 RON data size: {} bytes", ron_string.len());
        
        // Parse back from RON
        let start_time = std::time::Instant::now();
        let world_data: gta_game::serialization::SerializableWorld = 
            ron::from_str(&ron_string).expect("Failed to parse RON");
        let parse_time = start_time.elapsed();
        
        println!("🔄 Parsed RON data in {:?}", parse_time);
        
        // Load into fresh app
        let mut loaded_app = create_test_app();
        let start_time = std::time::Instant::now();
        WorldSerializer::apply_world_data(&mut loaded_app, &world_data)
            .expect("Failed to load world data");
        let load_time = start_time.elapsed();
        
        println!("⚡ Loaded {} entities in {:?}", world_data.entity_count, load_time);
        
        // Verify counts
        let mut loaded_count = 0;
        let mut query = loaded_app.world_mut().query::<Entity>();
        for _ in query.iter(loaded_app.world()) {
            loaded_count += 1;
        }
        
        assert_eq!(loaded_count, original_entities.len(), "Entity count mismatch");
        
        println!("✅ Large-scale serialization test PASSED");
    }
    
    #[test]
    fn test_file_save_load_round_trip() {
        println!("💾 Testing file-based save-load round-trip");
        
        let mut original_app = create_test_app();
        spawn_random_entities(&mut original_app, 50);
        
        // Save to file
        let temp_path = std::env::temp_dir().join("test_world.ron");
        WorldSerializer::save_world_to_file(original_app.world(), &temp_path)
            .expect("Failed to save world to file");
        
        println!("💾 Saved world to {:?}", temp_path);
        
        // Load from file
        let world_data = WorldSerializer::load_world_from_file(&temp_path)
            .expect("Failed to load world from file");
        
        println!("📂 Loaded world from file: {} entities", world_data.entity_count);
        
        // Apply to fresh app
        let mut loaded_app = create_test_app();
        WorldSerializer::apply_world_data(&mut loaded_app, &world_data)
            .expect("Failed to apply world data");
        
        // Verify
        let mut loaded_count = 0;
        let mut query = loaded_app.world_mut().query::<Entity>();
        for _ in query.iter(loaded_app.world()) {
            loaded_count += 1;
        }
        
        assert_eq!(loaded_count, world_data.entity_count, "Entity count mismatch");
        
        // Clean up
        std::fs::remove_file(&temp_path).ok();
        
        println!("✅ File-based save-load round-trip test PASSED");
    }
    
    #[test]
    fn test_component_specific_serialization() {
        println!("🔧 Testing component-specific serialization accuracy");
        
        let mut original_app = create_test_app();
        
        // Create entity with specific component values
        let test_entity = original_app.world_mut().spawn((
            Transform::from_xyz(123.456, 789.012, 345.678),
            Car,
            VehicleType::Car,
            VehicleState::Driving,
            SuperCar,
            UnifiedCullable::vehicle(),
            RigidBody::Dynamic,
            Collider::cuboid(2.5, 1.2, 4.8),
            Velocity {
                linvel: Vec3::new(10.0, 0.0, 15.0),
                angvel: Vec3::new(0.0, 0.5, 0.0),
            },
            CollisionGroups::new(Group::GROUP_1, Group::GROUP_2),
            ColliderMassProperties::Density(1500.0),
            Name::new("TestVehicle"),
        )).id();
        
        // Serialize and load
        let world_data = WorldSerializer::extract_world_data(original_app.world())
            .expect("Failed to serialize world");
        
        let mut loaded_app = create_test_app();
        WorldSerializer::apply_world_data(&mut loaded_app, &world_data)
            .expect("Failed to load world data");
        
        // Verify specific component values
        let mut query = loaded_app.world_mut().query::<(&Transform, &VehicleType, &VehicleState, &Velocity, &Name)>();
        let mut found_test_entity = false;
        
        for (transform, vehicle_type, vehicle_state, velocity, name) in query.iter(loaded_app.world()) {
            if name.as_str() == "TestVehicle" {
                found_test_entity = true;
                
                // Check transform precision
                assert!((transform.translation.x - 123.456).abs() < 0.001);
                assert!((transform.translation.y - 789.012).abs() < 0.001);
                assert!((transform.translation.z - 345.678).abs() < 0.001);
                
                // Check enum values
                assert_eq!(*vehicle_type, VehicleType::Car);
                assert_eq!(*vehicle_state, VehicleState::Driving);
                
                // Check velocity
                assert!((velocity.linvel.x - 10.0).abs() < 0.001);
                assert!((velocity.linvel.z - 15.0).abs() < 0.001);
                assert!((velocity.angvel.y - 0.5).abs() < 0.001);
                
                break;
            }
        }
        
        assert!(found_test_entity, "Test entity not found in loaded world");
        
        // Verify SuperCar component
        let mut query = loaded_app.world_mut().query::<(&SuperCar, &Name)>();
        let mut super_car_found = false;
        for (_, name) in query.iter(loaded_app.world()) {
            if name.as_str() == "TestVehicle" {
                super_car_found = true;
                break;
            }
        }
        assert!(super_car_found, "SuperCar component not preserved");
        
        println!("✅ Component-specific serialization test PASSED");
    }
}
