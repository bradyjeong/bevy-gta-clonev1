use crate::bundles::DynamicContentBundle;
use crate::components::world::BuildingType as WorldBuildingType;
use crate::components::{Building, ContentType, DynamicContent};
use crate::config::GameConfig;
use crate::factories::generic_bundle::BundleError;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_rapier3d::prelude::*;
use rand::Rng;

/// Building Factory - Focused factory for building spawning only
/// Handles various building types with proper physics and visual components
/// Follows AGENT.md simplicity principles with single responsibility
#[derive(Debug, Clone)]
pub struct BuildingFactory {
    pub config: GameConfig,
}

impl BuildingFactory {
    /// Create new building factory with default configuration
    pub fn new() -> Self {
        Self {
            config: GameConfig::default(),
        }
    }

    /// Create building factory with custom configuration
    pub fn with_config(config: GameConfig) -> Self {
        Self { config }
    }

    /// Spawn building with automatic size and color generation
    pub fn spawn_building(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        building_type: Option<BuildingType>,
    ) -> Result<Entity, BundleError> {
        let mut rng = rand::thread_rng();

        // Determine building parameters
        let building_type = building_type.unwrap_or(BuildingType::Generic);
        let height = rng.gen_range(8.0..30.0);
        let width = rng.gen_range(8.0..15.0);

        // Adjust position to place building base on ground
        let final_position = Vec3::new(position.x, position.y + height / 2.0, position.z);

        // Random building color
        let building_material = materials.add(StandardMaterial {
            base_color: Color::srgb(
                rng.gen_range(0.5..0.9),
                rng.gen_range(0.5..0.9),
                rng.gen_range(0.5..0.9),
            ),
            ..default()
        });

        let building_entity = commands
            .spawn((
                DynamicContentBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Building,
                    },
                    transform: Transform::from_translation(final_position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 350.0..400.0,
                        use_aabb: false,
                    },
                },
                Building {
                    building_type: building_type.to_world_building_type(),
                    height,
                    scale: Vec3::new(width, height, width),
                },
                RigidBody::Fixed,
                Collider::cuboid(width / 2.0, height / 2.0, width / 2.0),
                CollisionGroups::new(self.config.physics.static_group, Group::ALL),
                Mesh3d(meshes.add(Cuboid::new(width, height, width))),
                MeshMaterial3d(building_material),
                Name::new(format!("Building_{}", building_type.name())),
            ))
            .id();

        Ok(building_entity)
    }

    /// Spawn building with specific dimensions
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_building_with_size(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        size: Vec3,
        building_type: BuildingType,
        color: Color,
    ) -> Result<Entity, BundleError> {
        let final_position = Vec3::new(position.x, position.y + size.y / 2.0, position.z);

        let building_material = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });

        let building_entity = commands
            .spawn((
                DynamicContentBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Building,
                    },
                    transform: Transform::from_translation(final_position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 350.0..400.0,
                        use_aabb: false,
                    },
                },
                Building {
                    building_type: building_type.to_world_building_type(),
                    height: size.y,
                    scale: size,
                },
                RigidBody::Fixed,
                Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
                CollisionGroups::new(self.config.physics.static_group, Group::ALL),
                Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
                MeshMaterial3d(building_material),
                Name::new(format!("Building_{}", building_type.name())),
            ))
            .id();

        Ok(building_entity)
    }

    /// Spawn multiple buildings in batch
    pub fn spawn_building_batch(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        positions: Vec<Vec3>,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();

        for position in positions {
            let entity = self.spawn_building(commands, meshes, materials, position, None)?;
            entities.push(entity);
        }

        Ok(entities)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum BuildingType {
    Generic,
    Residential,
    Commercial,
    Industrial,
}

impl BuildingType {
    pub fn name(self) -> &'static str {
        match self {
            BuildingType::Generic => "Generic",
            BuildingType::Residential => "Residential",
            BuildingType::Commercial => "Commercial",
            BuildingType::Industrial => "Industrial",
        }
    }

    pub fn to_world_building_type(self) -> WorldBuildingType {
        match self {
            BuildingType::Generic => WorldBuildingType::Generic,
            BuildingType::Residential => WorldBuildingType::Residential,
            BuildingType::Commercial => WorldBuildingType::Commercial,
            BuildingType::Industrial => WorldBuildingType::Industrial,
        }
    }
}

impl Default for BuildingType {
    fn default() -> Self {
        Self::Generic
    }
}

impl Default for BuildingFactory {
    fn default() -> Self {
        Self::new()
    }
}
