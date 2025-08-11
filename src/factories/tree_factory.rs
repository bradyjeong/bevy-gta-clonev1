use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::bundles::VegetationBundle;
use crate::systems::UnifiedCullable;
use crate::factories::common::{FocusedFactory, GroundHeightCache, SpawnValidation, PhysicsSetup, EntityPhysicsType};
use crate::systems::RoadNetwork;
use crate::GameConfig;

/// Focused factory for vegetation entities following AGENT.md simplicity principles
/// Single responsibility: tree/vegetation creation with LOD support
pub struct VegetationFactory {}

impl Default for VegetationFactory {
    fn default() -> Self {
        Self {}
    }
}

impl FocusedFactory for VegetationFactory {
    fn name() -> &'static str {
        "VegetationFactory"
    }
    
    fn entity_limit() -> usize {
        100 // From AGENT.md: 5% of 2000 = 100 trees
    }
}

impl VegetationFactory {
    /// Create new vegetation factory
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Spawn vegetation entity with complete setup
    pub fn spawn_vegetation(
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
        
        // Validate spawn position (trees avoid roads)
        if !SpawnValidation::is_position_valid(position, ContentType::Tree, road_network) {
            return Err("Invalid position for vegetation - on road".to_string());
        }
        
        // Position vegetation on ground
        let ground_level = ground_cache.get_ground_height(Vec2::new(position.x, position.z));
        let final_position = Vec3::new(position.x, ground_level, position.z);
        
        // Random vegetation properties
        let vegetation_type = Self::random_vegetation_type(&mut rng);
        let (height, radius, color) = Self::vegetation_properties(vegetation_type, &mut rng);
        
        // Create vegetation entity
        let vegetation_entity = commands.spawn((
            VegetationBundle {
                dynamic_content: DynamicContent { content_type: ContentType::Tree },
                transform: Transform::from_translation(final_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                cullable: UnifiedCullable::vegetation(),
            },
            VegetationLOD::default(),
            // Vegetation physics using common utilities
            RigidBody::Fixed,
            Collider::cylinder(radius, height / 2.0),
            PhysicsSetup::collision_groups_for_entity(EntityPhysicsType::StaticVegetation, config),
            vegetation_type,
            Name::new(format!("Tree_{:.0}_{:.0}_{}", position.x, position.z, current_time)),
        )).id();
        
        // Add visual components as children for LOD
        Self::add_vegetation_visual(
            commands,
            vegetation_entity,
            meshes,
            materials,
            vegetation_type,
            height,
            radius,
            color,
        );
        
        Ok(vegetation_entity)
    }
    
    /// Spawn batch of vegetation
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
            if let Ok(entity) = self.spawn_vegetation(
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
    
    /// Generate random vegetation type
    fn random_vegetation_type(rng: &mut impl Rng) -> VegetationType {
        match rng.gen_range(0..4) {
            0 => VegetationType::Oak,
            1 => VegetationType::Pine,
            2 => VegetationType::Palm,
            _ => VegetationType::Bush,
        }
    }
    
    /// Get vegetation properties based on type
    fn vegetation_properties(vegetation_type: VegetationType, rng: &mut impl Rng) -> (f32, f32, Color) {
        match vegetation_type {
            VegetationType::Oak => (
                rng.gen_range(6.0..12.0),
                rng.gen_range(0.4..0.8),
                Color::srgb(0.2, 0.6, 0.1),
            ),
            VegetationType::Pine => (
                rng.gen_range(8.0..15.0),
                rng.gen_range(0.3..0.6),
                Color::srgb(0.1, 0.4, 0.1),
            ),
            VegetationType::Palm => (
                rng.gen_range(10.0..18.0),
                rng.gen_range(0.3..0.5),
                Color::srgb(0.3, 0.7, 0.2),
            ),
            VegetationType::Bush => (
                rng.gen_range(1.0..3.0),
                rng.gen_range(0.8..1.5),
                Color::srgb(0.3, 0.5, 0.2),
            ),
        }
    }
    
    /// Add visual components for vegetation with LOD support
    fn add_vegetation_visual(
        commands: &mut Commands,
        parent: Entity,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        vegetation_type: VegetationType,
        height: f32,
        radius: f32,
        color: Color,
    ) {
        let mesh = match vegetation_type {
            VegetationType::Bush => meshes.add(Sphere::new(radius)),
            _ => meshes.add(Cylinder::new(radius, height)),
        };
        
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Transform::from_xyz(0.0, height / 2.0, 0.0),
            ChildOf(parent),
        ));
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum VegetationType {
    Oak,
    Pine,
    Palm,
    Bush,
}

impl Default for VegetationType {
    fn default() -> Self {
        Self::Oak
    }
}
