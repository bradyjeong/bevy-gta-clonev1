//! World serialization system for save-load functionality
//! 
//! This module provides comprehensive world state serialization to RON format,
//! supporting round-trip testing and save-load operations.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::bundles::*;

/// Serializable representation of a Bevy Transform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // Quaternion as [x, y, z, w]
    pub scale: [f32; 3],
}

impl From<Transform> for SerializableTransform {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation.to_array(),
            rotation: [
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ],
            scale: transform.scale.to_array(),
        }
    }
}

impl From<SerializableTransform> for Transform {
    fn from(serializable: SerializableTransform) -> Self {
        Self {
            translation: Vec3::from_array(serializable.translation),
            rotation: Quat::from_array(serializable.rotation),
            scale: Vec3::from_array(serializable.scale),
        }
    }
}

/// Serializable representation of physics velocity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableVelocity {
    pub linear: [f32; 3],
    pub angular: [f32; 3],
}

impl From<Velocity> for SerializableVelocity {
    fn from(velocity: Velocity) -> Self {
        Self {
            linear: velocity.linvel.to_array(),
            angular: velocity.angvel.to_array(),
        }
    }
}

impl From<SerializableVelocity> for Velocity {
    fn from(serializable: SerializableVelocity) -> Self {
        Self {
            linvel: Vec3::from_array(serializable.linear),
            angvel: Vec3::from_array(serializable.angular),
        }
    }
}

/// Comprehensive entity data for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEntity {
    pub original_id: u32,
    pub transform: Option<SerializableTransform>,
    pub velocity: Option<SerializableVelocity>,
    pub visibility: Option<String>,
    
    // Component markers
    pub is_player: bool,
    pub is_active_entity: bool,
    pub is_in_car: bool,
    pub is_car: bool,
    pub is_super_car: bool,
    pub is_helicopter: bool,
    pub is_f16: bool,
    pub is_building: bool,
    pub is_npc: bool,
    pub is_road_entity: bool,
    pub is_intersection_entity: bool,
    pub is_dynamic_terrain: bool,
    pub is_dynamic_content: bool,
    pub is_landmark: bool,
    pub is_main_camera: bool,
    pub is_performance_critical: bool,
    
    // Component data
    pub vehicle_type: Option<VehicleType>,
    pub vehicle_state: Option<VehicleState>,
    pub npc_type: Option<NPCType>,
    pub npc_behavior: Option<NPCBehaviorType>,
    pub npc_gender: Option<NPCGender>,
    pub building_type: Option<BuildingType>,
    pub content_type: Option<ContentType>,
    pub cullable: Option<CullingCategory>,
    pub lod_level: Option<LODLevel>,
    
    // Physics data
    pub rigid_body: Option<String>, // "Dynamic", "Fixed", "KinematicPositionBased", "KinematicVelocityBased"
    pub collider_shape: Option<String>, // Simplified collider description
    pub collision_group: Option<u32>,
    pub collision_mask: Option<u32>,
    pub mass: Option<f32>,
    
    // Hierarchy
    pub parent_id: Option<u32>,
    pub child_ids: Vec<u32>,
    
    // Metadata
    pub name: Option<String>,
}

/// Complete world state for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableWorld {
    pub entities: Vec<SerializableEntity>,
    pub entity_count: usize,
    pub version: String,
    pub timestamp: String,
    pub metadata: HashMap<String, String>,
}

/// World serialization system
pub struct WorldSerializer;

impl WorldSerializer {
    /// Serialize world to RON format
    pub fn serialize_world_to_ron(world: &World) -> Result<String, Box<dyn std::error::Error>> {
        let serializable_world = Self::extract_world_data(world)?;
        let ron_string = ron::to_string_pretty(&serializable_world, ron::PrettyConfig::default())?;
        Ok(ron_string)
    }
    
    /// Save world to RON file
    pub fn save_world_to_file<P: AsRef<Path>>(world: &World, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let ron_string = Self::serialize_world_to_ron(world)?;
        fs::write(path, ron_string)?;
        Ok(())
    }
    
    /// Load world from RON file
    pub fn load_world_from_file<P: AsRef<Path>>(path: P) -> Result<SerializableWorld, Box<dyn std::error::Error>> {
        let ron_string = fs::read_to_string(path)?;
        let world_data: SerializableWorld = ron::from_str(&ron_string)?;
        Ok(world_data)
    }
    
    /// Apply serialized world data to a fresh Bevy App
    pub fn apply_world_data(app: &mut App, world_data: &SerializableWorld) -> Result<(), Box<dyn std::error::Error>> {
        let mut entity_id_map = HashMap::new();
        
        // First pass: Create all entities and store ID mappings
        for serializable_entity in &world_data.entities {
            let entity_id = app.world_mut().spawn_empty().id();
            entity_id_map.insert(serializable_entity.original_id, entity_id);
        }
        
        // Second pass: Apply components and set up hierarchy
        for serializable_entity in &world_data.entities {
            let entity_id = entity_id_map[&serializable_entity.original_id];
            Self::apply_entity_components(app.world_mut(), entity_id, serializable_entity, &entity_id_map)?;
        }
        
        Ok(())
    }
    
    /// Extract world data for serialization
    fn extract_world_data(world: &World) -> Result<SerializableWorld, Box<dyn std::error::Error>> {
        let mut entities = Vec::new();
        let mut entity_count = 0;
        
        // Query all entities with Transform (our primary serialization target)
        let mut query = world.query::<(Entity, Option<&Transform>, Option<&Parent>, Option<&Children>)>();
        
        for (entity, transform, parent, children) in query.iter(world) {
            let serializable_entity = Self::extract_entity_data(world, entity, transform, parent, children)?;
            entities.push(serializable_entity);
            entity_count += 1;
        }
        
        // Also capture entities without Transform but with important components
        let mut query_no_transform = world.query_filtered::<Entity, (Without<Transform>, Or<(With<Player>, With<Car>, With<NPC>, With<Building>)>)>();
        
        for entity in query_no_transform.iter(world) {
            let serializable_entity = Self::extract_entity_data(world, entity, None, None, None)?;
            entities.push(serializable_entity);
            entity_count += 1;
        }
        
        let metadata = HashMap::from([
            ("description".to_string(), "Bevy GTA-like game world snapshot".to_string()),
            ("generator".to_string(), "WorldSerializer".to_string()),
        ]);
        
        Ok(SerializableWorld {
            entities,
            entity_count,
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata,
        })
    }
    
    /// Extract data from a single entity
    fn extract_entity_data(
        world: &World,
        entity: Entity,
        transform: Option<&Transform>,
        parent: Option<&Parent>,
        children: Option<&Children>,
    ) -> Result<SerializableEntity, Box<dyn std::error::Error>> {
        let mut serializable = SerializableEntity {
            original_id: entity.index(),
            transform: transform.map(|t| (*t).into()),
            velocity: world.get::<Velocity>(entity).map(|v| (*v).into()),
            visibility: world.get::<Visibility>(entity).map(|v| format!("{:?}", v)),
            
            // Initialize all flags as false
            is_player: false,
            is_active_entity: false,
            is_in_car: false,
            is_car: false,
            is_super_car: false,
            is_helicopter: false,
            is_f16: false,
            is_building: false,
            is_npc: false,
            is_road_entity: false,
            is_intersection_entity: false,
            is_dynamic_terrain: false,
            is_dynamic_content: false,
            is_landmark: false,
            is_main_camera: false,
            is_performance_critical: false,
            
            // Component data
            vehicle_type: world.get::<VehicleType>(entity).cloned(),
            vehicle_state: world.get::<VehicleState>(entity).cloned(),
            npc_type: world.get::<NPCType>(entity).cloned(),
            npc_behavior: world.get::<NPCBehaviorComponent>(entity).map(|npc| npc.behavior_type),
            npc_gender: world.get::<NPCGender>(entity).cloned(),
            building_type: world.get::<BuildingType>(entity).cloned(),
            content_type: world.get::<ContentType>(entity).cloned(),
            cullable: world.get::<UnifiedCullable>(entity).map(|c| c.category),
            lod_level: world.get::<LODLevel>(entity).cloned(),
            
            // Physics data
            rigid_body: world.get::<RigidBody>(entity).map(|rb| format!("{:?}", rb)),
            collider_shape: world.get::<Collider>(entity).map(|_| "Complex".to_string()), // Simplified
            collision_group: world.get::<CollisionGroups>(entity).map(|cg| cg.memberships.bits()),
            collision_mask: world.get::<CollisionGroups>(entity).map(|cg| cg.filters.bits()),
            mass: world.get::<ColliderMassProperties>(entity).and_then(|mass| {
                match mass {
                    ColliderMassProperties::Density(d) => Some(*d),
                    ColliderMassProperties::Mass(m) => Some(*m),
                    _ => None,
                }
            }),
            
            // Hierarchy
            parent_id: parent.map(|p| p.get().index()),
            child_ids: children.map(|c| c.iter().map(|child| child.index()).collect()).unwrap_or_default(),
            
            // Metadata
            name: world.get::<Name>(entity).map(|n| n.to_string()),
        };
        
        // Set component flags
        serializable.is_player = world.get::<Player>(entity).is_some();
        serializable.is_active_entity = world.get::<ActiveEntity>(entity).is_some();
        serializable.is_in_car = world.get::<InCar>(entity).is_some();
        serializable.is_car = world.get::<Car>(entity).is_some();
        serializable.is_super_car = world.get::<SuperCar>(entity).is_some();
        serializable.is_helicopter = world.get::<Helicopter>(entity).is_some();
        serializable.is_f16 = world.get::<F16>(entity).is_some();
        serializable.is_building = world.get::<Building>(entity).is_some();
        serializable.is_npc = world.get::<NPC>(entity).is_some();
        serializable.is_road_entity = world.get::<RoadEntity>(entity).is_some();
        serializable.is_intersection_entity = world.get::<IntersectionEntity>(entity).is_some();
        serializable.is_dynamic_terrain = world.get::<DynamicTerrain>(entity).is_some();
        serializable.is_dynamic_content = world.get::<DynamicContent>(entity).is_some();
        serializable.is_landmark = world.get::<Landmark>(entity).is_some();
        serializable.is_main_camera = world.get::<MainCamera>(entity).is_some();
        serializable.is_performance_critical = world.get::<PerformanceCritical>(entity).is_some();
        
        Ok(serializable)
    }
    
    /// Apply entity components to a world
    fn apply_entity_components(
        world: &mut World,
        entity_id: Entity,
        serializable_entity: &SerializableEntity,
        entity_id_map: &HashMap<u32, Entity>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut entity_mut = world.entity_mut(entity_id);
        
        // Apply transform
        if let Some(transform) = &serializable_entity.transform {
            entity_mut.insert(Transform::from(transform.clone()));
        }
        
        // Apply velocity
        if let Some(velocity) = &serializable_entity.velocity {
            entity_mut.insert(Velocity::from(velocity.clone()));
        }
        
        // Apply visibility
        if let Some(visibility_str) = &serializable_entity.visibility {
            let visibility = match visibility_str.as_str() {
                "Visible" => Visibility::Visible,
                "Hidden" => Visibility::Hidden,
                _ => Visibility::Inherited,
            };
            entity_mut.insert(visibility);
        }
        
        // Apply component markers
        if serializable_entity.is_player {
            entity_mut.insert(Player);
        }
        if serializable_entity.is_active_entity {
            entity_mut.insert(ActiveEntity);
        }
        if serializable_entity.is_in_car {
            entity_mut.insert(InCar);
        }
        if serializable_entity.is_car {
            entity_mut.insert(Car);
        }
        if serializable_entity.is_super_car {
            entity_mut.insert(SuperCar);
        }
        if serializable_entity.is_helicopter {
            entity_mut.insert(Helicopter);
        }
        if serializable_entity.is_f16 {
            entity_mut.insert(F16);
        }
        if serializable_entity.is_building {
            entity_mut.insert(Building);
        }
        if serializable_entity.is_npc {
            entity_mut.insert(NPC);
        }
        if serializable_entity.is_road_entity {
            entity_mut.insert(RoadEntity);
        }
        if serializable_entity.is_intersection_entity {
            entity_mut.insert(IntersectionEntity);
        }
        if serializable_entity.is_dynamic_terrain {
            entity_mut.insert(DynamicTerrain);
        }
        if serializable_entity.is_dynamic_content {
            entity_mut.insert(DynamicContent);
        }
        if serializable_entity.is_landmark {
            entity_mut.insert(Landmark);
        }
        if serializable_entity.is_main_camera {
            entity_mut.insert(MainCamera);
        }
        if serializable_entity.is_performance_critical {
            entity_mut.insert(PerformanceCritical);
        }
        
        // Apply component data
        if let Some(vehicle_type) = &serializable_entity.vehicle_type {
            entity_mut.insert(vehicle_type.clone());
        }
        if let Some(vehicle_state) = &serializable_entity.vehicle_state {
            entity_mut.insert(vehicle_state.clone());
        }
        if let Some(npc_type) = &serializable_entity.npc_type {
            entity_mut.insert(npc_type.clone());
        }
        if let Some(npc_behavior) = &serializable_entity.npc_behavior {
            entity_mut.insert(NPCBehaviorComponent {
                behavior_type: *npc_behavior,
                last_update: 0.0,
            });
        }
        if let Some(npc_gender) = &serializable_entity.npc_gender {
            entity_mut.insert(npc_gender.clone());
        }
        if let Some(building_type) = &serializable_entity.building_type {
            entity_mut.insert(building_type.clone());
        }
        if let Some(content_type) = &serializable_entity.content_type {
            entity_mut.insert(content_type.clone());
        }
        if let Some(cullable) = &serializable_entity.cullable {
            entity_mut.insert(UnifiedCullable::new(*cullable));
        }
        if let Some(lod_level) = &serializable_entity.lod_level {
            entity_mut.insert(lod_level.clone());
        }
        
        // Apply physics components
        if let Some(rigid_body_str) = &serializable_entity.rigid_body {
            let rigid_body = match rigid_body_str.as_str() {
                "Dynamic" => RigidBody::Dynamic,
                "Fixed" => RigidBody::Fixed,
                "KinematicPositionBased" => RigidBody::KinematicPositionBased,
                "KinematicVelocityBased" => RigidBody::KinematicVelocityBased,
                _ => RigidBody::Dynamic,
            };
            entity_mut.insert(rigid_body);
        }
        
        if serializable_entity.collider_shape.is_some() {
            // Create a default collider for testing
            entity_mut.insert(Collider::cuboid(1.0, 1.0, 1.0));
        }
        
        if let (Some(group), Some(mask)) = (&serializable_entity.collision_group, &serializable_entity.collision_mask) {
            entity_mut.insert(CollisionGroups::new(
                Group::from_bits_truncate(*group),
                Group::from_bits_truncate(*mask),
            ));
        }
        
        if let Some(mass) = &serializable_entity.mass {
            entity_mut.insert(ColliderMassProperties::Density(*mass));
        }
        
        // Apply hierarchy
        if let Some(parent_id) = &serializable_entity.parent_id {
            if let Some(&parent_entity) = entity_id_map.get(parent_id) {
                entity_mut.insert(Parent(parent_entity));
            }
        }
        
        if !serializable_entity.child_ids.is_empty() {
            let children: Vec<Entity> = serializable_entity
                .child_ids
                .iter()
                .filter_map(|id| entity_id_map.get(id).cloned())
                .collect();
            if !children.is_empty() {
                entity_mut.insert(Children::from_slice(&children));
            }
        }
        
        // Apply name
        if let Some(name) = &serializable_entity.name {
            entity_mut.insert(Name::new(name.clone()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundles::*;
    use crate::factories::entity_factory_unified::*;
    use bevy_rapier3d::prelude::*;
    use rand::Rng;
    
    #[test]
    fn test_basic_world_serialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(TransformPlugin)
           .add_plugins(HierarchyPlugin);
        
        // Create a simple entity
        app.world_mut().spawn((
            Transform::from_xyz(10.0, 20.0, 30.0),
            Visibility::Visible,
            Name::new("TestEntity"),
        ));
        
        let ron_string = WorldSerializer::serialize_world_to_ron(app.world()).unwrap();
        assert!(ron_string.contains("TestEntity"));
        assert!(ron_string.contains("10.0"));
        assert!(ron_string.contains("20.0"));
        assert!(ron_string.contains("30.0"));
    }
    
    #[test]
    fn test_component_serialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(TransformPlugin)
           .add_plugins(HierarchyPlugin);
        
        // Create entity with various components
        app.world_mut().spawn((
            Transform::from_xyz(5.0, 10.0, 15.0),
            Player,
            ActiveEntity,
            Car,
            VehicleType::Car,
            VehicleState::Parked,
            UnifiedCullable::vehicle(),
            Name::new("PlayerCar"),
        ));
        
        let ron_string = WorldSerializer::serialize_world_to_ron(app.world()).unwrap();
        assert!(ron_string.contains("PlayerCar"));
        assert!(ron_string.contains("is_player: true"));
        assert!(ron_string.contains("is_car: true"));
        assert!(ron_string.contains("Parked"));
    }
    
    #[test]
    fn test_save_load_round_trip() {
        let mut original_app = App::new();
        original_app.add_plugins(MinimalPlugins)
                   .add_plugins(TransformPlugin)
                   .add_plugins(HierarchyPlugin);
        
        // Create test entities
        let entity1 = original_app.world_mut().spawn((
            Transform::from_xyz(1.0, 2.0, 3.0),
            Player,
            ActiveEntity,
            Name::new("Player1"),
        )).id();
        
        let entity2 = original_app.world_mut().spawn((
            Transform::from_xyz(4.0, 5.0, 6.0),
            Car,
            VehicleType::Car,
            VehicleState::Driving,
            UnifiedCullable::vehicle(),
            Name::new("Car1"),
        )).id();
        
        // Serialize to RON
        let world_data = WorldSerializer::extract_world_data(original_app.world()).unwrap();
        assert_eq!(world_data.entity_count, 2);
        
        // Create fresh app and apply world data
        let mut new_app = App::new();
        new_app.add_plugins(MinimalPlugins)
               .add_plugins(TransformPlugin)
               .add_plugins(HierarchyPlugin);
        
        WorldSerializer::apply_world_data(&mut new_app, &world_data).unwrap();
        
        // Verify entity count
        let mut entity_count = 0;
        let mut query = new_app.world_mut().query::<Entity>();
        for _ in query.iter(new_app.world()) {
            entity_count += 1;
        }
        assert_eq!(entity_count, 2);
        
        // Verify components
        let mut player_found = false;
        let mut car_found = false;
        let mut query = new_app.world_mut().query::<(&Transform, &Name)>();
        for (transform, name) in query.iter(new_app.world()) {
            if name.as_str() == "Player1" {
                player_found = true;
                assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
            } else if name.as_str() == "Car1" {
                car_found = true;
                assert_eq!(transform.translation, Vec3::new(4.0, 5.0, 6.0));
            }
        }
        assert!(player_found);
        assert!(car_found);
    }
}
