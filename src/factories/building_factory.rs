use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::bundles::DynamicContentBundle;
use crate::systems::UnifiedCullable;
use crate::factories::common::{FocusedFactory, GroundHeightCache, PhysicsSetup, EntityPhysicsType};
use crate::world::RoadNetwork;
use crate::GameConfig;

/// Focused factory for building entities following AGENT.md simplicity principles
/// Single responsibility: building creation with consistent physics and visuals
pub struct BuildingsFactory {}

impl Default for BuildingsFactory {
    fn default() -> Self {
        Self {}
    }
}

impl FocusedFactory for BuildingsFactory {
    fn name() -> &'static str {
        "BuildingsFactory"
    }
    
    fn entity_limit() -> usize {
        80 // From AGENT.md: 8% of 1000 = 80 buildings
    }
}

impl BuildingsFactory {
    /// Create new buildings factory
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Spawn building entity with complete setup
    pub fn spawn_building(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        config: &GameConfig,
        current_time: f32,
        _road_network: Option<&RoadNetwork>,
        ground_cache: &mut GroundHeightCache,
    ) -> Result<Entity, String> {
        let mut rng = rand::thread_rng();
        
        // NOTE: Position validation performed upstream in event-driven pipeline
        // When called through RequestDynamicSpawn events, position is already validated
        
        // Generate building properties
        let height = rng.gen_range(8.0..30.0);
        let width = rng.gen_range(8.0..15.0);
        let building_type = Self::random_building_type(&mut rng);
        
        // Position on ground surface
        let ground_level = ground_cache.get_ground_height(Vec2::new(position.x, position.z));
        let building_y = ground_level + height / 2.0;
        let final_position = Vec3::new(position.x, building_y, position.z);
        
        // Create material
        let material = materials.add(StandardMaterial {
            base_color: Self::building_color(&mut rng, building_type),
            ..default()
        });
        
        // Spawn building entity
        let building_entity = commands.spawn((
            DynamicContentBundle {
                dynamic_content: DynamicContent { content_type: ContentType::Building },
                transform: Transform::from_translation(final_position),
                visibility: Visibility::Visible,
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                cullable: UnifiedCullable::building(),
            },
            Building {
                building_type: crate::components::world::BuildingType::Generic,
                height,
                scale: Vec3::new(width, height, width),
            },
            // Physics setup using common utilities
            RigidBody::Fixed,
            Collider::cuboid(width / 2.0, height / 2.0, width / 2.0),
            PhysicsSetup::collision_groups_for_entity(EntityPhysicsType::StaticBuilding, config),
            // Visual components
            Mesh3d(meshes.add(Cuboid::new(width, height, width))),
            MeshMaterial3d(material),
            Name::new(format!("Building_{:.0}_{:.0}_{}", position.x, position.z, current_time)),
        )).id();
        
        Ok(building_entity)
    }
    
    /// Spawn batch of buildings
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
            if let Ok(entity) = self.spawn_building(
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
    
    /// Generate random building type
    fn random_building_type(rng: &mut impl Rng) -> BuildingType {
        match rng.gen_range(0..3) {
            0 => BuildingType::Residential,
            1 => BuildingType::Commercial,
            _ => BuildingType::Industrial,
        }
    }
    
    /// Get building color based on type
    fn building_color(rng: &mut impl Rng, building_type: BuildingType) -> Color {
        match building_type {
            BuildingType::Residential => Color::srgb(
                rng.gen_range(0.6..0.9),
                rng.gen_range(0.5..0.8),
                rng.gen_range(0.4..0.7),
            ),
            BuildingType::Commercial => Color::srgb(
                rng.gen_range(0.7..1.0),
                rng.gen_range(0.7..1.0),
                rng.gen_range(0.8..1.0),
            ),
            BuildingType::Industrial => Color::srgb(
                rng.gen_range(0.4..0.6),
                rng.gen_range(0.4..0.6),
                rng.gen_range(0.4..0.6),
            ),
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum BuildingType {
    Residential,
    Commercial,
    Industrial,
}

impl Default for BuildingType {
    fn default() -> Self {
        Self::Residential
    }
}
