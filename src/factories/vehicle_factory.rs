use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::bundles::{DynamicPhysicsBundle, VisibleChildBundle};
use crate::systems::{UnifiedCullable, MovementTracker};
use crate::factories::common::{FocusedFactory, GroundHeightCache, SpawnValidation, PhysicsSetup, EntityPhysicsType};
use crate::world::RoadNetwork;
use crate::GameConfig;

/// Focused factory for vehicle entities following AGENT.md simplicity principles
/// Single responsibility: vehicle creation with consistent physics and visuals
pub struct VehicleFactory {}

impl Default for VehicleFactory {
    fn default() -> Self {
        Self {}
    }
}

impl FocusedFactory for VehicleFactory {
    fn name() -> &'static str {
        "VehicleFactory"
    }
    
    fn entity_limit() -> usize {
        20 // From AGENT.md: 4% of 500 = 20 vehicles
    }
}

impl VehicleFactory {
    /// Create new vehicle factory
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Spawn vehicle entity with complete setup
    pub fn spawn_vehicle(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        config: &GameConfig,
        current_time: f32,
        road_network: Option<&RoadNetwork>,
        ground_cache: &mut GroundHeightCache,
    ) -> Result<Entity, String> {
        let mut rng = rand::thread_rng();
        
        // Validate spawn position (vehicles need roads)
        if !SpawnValidation::is_position_valid(position, ContentType::Vehicle, road_network) {
            return Err("Invalid position for vehicle - not on road".to_string());
        }
        
        // Position vehicle on ground surface
        let ground_level = ground_cache.get_ground_height(Vec2::new(position.x, position.z));
        let final_position = Vec3::new(position.x, ground_level + 0.5, position.z);
        
        // Random vehicle color
        let car_colors = [
            Color::srgb(1.0, 0.0, 0.0), Color::srgb(0.0, 0.0, 1.0), Color::srgb(0.0, 1.0, 0.0),
            Color::srgb(1.0, 1.0, 0.0), Color::srgb(1.0, 0.0, 1.0), Color::srgb(0.0, 1.0, 1.0),
            Color::srgb(0.5, 0.5, 0.5), Color::srgb(1.0, 1.0, 1.0), Color::srgb(0.0, 0.0, 0.0),
        ];
        let color = car_colors[rng.gen_range(0..car_colors.len())];
        
        // Create vehicle entity
        let vehicle_entity = commands.spawn((
            DynamicPhysicsBundle {
                dynamic_content: DynamicContent { content_type: ContentType::Vehicle },
                transform: Transform::from_translation(final_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                rigid_body: RigidBody::Dynamic,
                collider: Collider::cuboid(1.0, 0.5, 2.0),
                collision_groups: PhysicsSetup::collision_groups_for_entity(EntityPhysicsType::DynamicVehicle, config),
                velocity: Velocity::default(),
                cullable: UnifiedCullable::vehicle(),
            },
            // Vehicle-specific components
            Car,
            PhysicsSetup::locked_axes_for_entity(EntityPhysicsType::DynamicVehicle).unwrap(),
            PhysicsSetup::damping_for_entity(EntityPhysicsType::DynamicVehicle),
            MovementTracker::new(final_position, 10.0),
            Name::new(format!("Vehicle_{:.0}_{:.0}_{}", position.x, position.z, current_time)),
        )).id();
        
        // Add car body as child entity
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 1.0, 3.6))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
        ));
        
        Ok(vehicle_entity)
    }
    
    /// Spawn batch of vehicles
    pub fn spawn_batch(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        positions: Vec<Vec3>,
        config: &GameConfig,
        current_time: f32,
        road_network: Option<&RoadNetwork>,
        ground_cache: &mut GroundHeightCache,
    ) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for position in positions {
            if let Ok(entity) = self.spawn_vehicle(
                commands, 
                meshes, 
                materials, 
                position, 
                config, 
                current_time,
                road_network,
                ground_cache
            ) {
                entities.push(entity);
            }
        }
        
        entities
    }
}
