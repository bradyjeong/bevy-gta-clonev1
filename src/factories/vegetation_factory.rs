use crate::bundles::VegetationBundle;
use crate::components::*;
use crate::config::GameConfig;
use crate::factories::generic_bundle::BundleError;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use rand::Rng;

/// Vegetation Factory - Focused factory for tree and vegetation spawning only
/// Handles trees, bushes, and other vegetation with proper LOD and visual components
/// Follows AGENT.md simplicity principles with single responsibility
#[derive(Debug, Clone)]
pub struct VegetationFactory {
    pub config: GameConfig,
}

impl VegetationFactory {
    /// Create new vegetation factory with default configuration
    pub fn new() -> Self {
        Self {
            config: GameConfig::default(),
        }
    }

    /// Create vegetation factory with custom configuration
    pub fn with_config(config: GameConfig) -> Self {
        Self { config }
    }

    /// Spawn tree with automatic type selection
    pub fn spawn_tree(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        tree_type: Option<VegetationType>,
    ) -> Result<Entity, BundleError> {
        let tree_type = tree_type.unwrap_or_else(|| self.random_tree_type());
        let (height, radius, color) = self.get_tree_properties(tree_type);

        // Position tree base on ground
        let final_position = Vec3::new(position.x, position.y, position.z);

        let tree_material = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });

        let tree_entity = commands
            .spawn((
                VegetationBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Tree,
                    },
                    transform: Transform::from_translation(final_position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 250.0..300.0,
                        use_aabb: false,
                    },
                },
                tree_type,
                VegetationLOD::default(),
                Mesh3d(meshes.add(Cylinder::new(radius, height))),
                MeshMaterial3d(tree_material),
                Name::new(format!("Tree_{}", tree_type.name())),
            ))
            .id();

        Ok(tree_entity)
    }

    /// Spawn bush with smaller dimensions
    pub fn spawn_bush(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Result<Entity, BundleError> {
        let height = 1.5;
        let radius = 0.8;
        let color = Color::srgb(0.2, 0.6, 0.2);

        let final_position = Vec3::new(position.x, position.y + height / 2.0, position.z);

        let bush_material = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });

        let bush_entity = commands
            .spawn((
                VegetationBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Tree, // Use Tree content type for vegetation
                    },
                    transform: Transform::from_translation(final_position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 150.0..200.0,
                        use_aabb: false,
                    },
                },
                VegetationType::Bush,
                VegetationLOD::default(),
                Mesh3d(meshes.add(Sphere::new(radius))), // Spherical bush
                MeshMaterial3d(bush_material),
                Name::new("Bush"),
            ))
            .id();

        Ok(bush_entity)
    }

    /// Spawn vegetation by type
    pub fn spawn_vegetation_by_type(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vegetation_type: VegetationType,
        position: Vec3,
    ) -> Result<Entity, BundleError> {
        match vegetation_type {
            VegetationType::Bush => self.spawn_bush(commands, meshes, materials, position),
            _ => self.spawn_tree(commands, meshes, materials, position, Some(vegetation_type)),
        }
    }

    /// Spawn multiple trees in batch
    pub fn spawn_tree_batch(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        positions: Vec<Vec3>,
        tree_type: Option<VegetationType>,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();

        for position in positions {
            let entity = self.spawn_tree(commands, meshes, materials, position, tree_type)?;
            entities.push(entity);
        }

        Ok(entities)
    }

    /// Get tree properties based on type
    fn get_tree_properties(&self, tree_type: VegetationType) -> (f32, f32, Color) {
        match tree_type {
            VegetationType::Oak => (8.0, 0.3, Color::srgb(0.4, 0.2, 0.1)), // Brown trunk
            VegetationType::Pine => (12.0, 0.25, Color::srgb(0.3, 0.15, 0.1)), // Darker trunk
            VegetationType::Palm => (10.0, 0.2, Color::srgb(0.5, 0.3, 0.2)), // Lighter trunk
            VegetationType::Bush => (1.5, 0.8, Color::srgb(0.2, 0.6, 0.2)), // Green bush
        }
    }

    /// Generate random tree type
    fn random_tree_type(&self) -> VegetationType {
        let mut rng = rand::thread_rng();
        let tree_types = [
            VegetationType::Oak,
            VegetationType::Pine,
            VegetationType::Palm,
        ];
        tree_types[rng.gen_range(0..tree_types.len())]
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum VegetationType {
    Oak,
    Pine,
    Palm,
    Bush,
}

impl VegetationType {
    pub fn name(self) -> &'static str {
        match self {
            VegetationType::Oak => "Oak",
            VegetationType::Pine => "Pine",
            VegetationType::Palm => "Palm",
            VegetationType::Bush => "Bush",
        }
    }
}

impl Default for VegetationType {
    fn default() -> Self {
        Self::Oak
    }
}

impl Default for VegetationFactory {
    fn default() -> Self {
        Self::new()
    }
}
