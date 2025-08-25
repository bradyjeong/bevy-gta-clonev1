use crate::bundles::{DynamicPhysicsBundle, VisibleChildBundle};
use crate::components::water::Yacht;
use crate::components::*;
use crate::config::GameConfig;
use crate::factories::generic_bundle::BundleError;
use crate::systems::MovementTracker;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_rapier3d::prelude::*;
use rand::Rng;

/// Vehicle Factory - Focused factory for vehicle spawning only
/// Handles SuperCar, Helicopter, F16, and Yacht creation with proper mesh-collider consistency
/// Follows AGENT.md simplicity principles with single responsibility
#[derive(Debug, Clone)]
pub struct VehicleFactory {
    pub config: GameConfig,
}

impl VehicleFactory {
    /// Create new vehicle factory with default configuration
    pub fn new() -> Self {
        Self {
            config: GameConfig::default(),
        }
    }

    /// Create vehicle factory with custom configuration
    pub fn with_config(config: GameConfig) -> Self {
        Self { config }
    }

    /// Spawn SuperCar with proper mesh-collider consistency
    /// Mesh: 1.9×1.3×4.7, Collider: 0.76×0.52×1.88 (0.8x for GTA-style forgiving collision)
    pub fn spawn_super_car(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        let color = color.unwrap_or_else(|| self.random_car_color());

        let vehicle_entity = commands
            .spawn((
                DynamicPhysicsBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Vehicle,
                    },
                    transform: Transform::from_translation(position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::cuboid(0.76 / 2.0, 0.52 / 2.0, 1.88 / 2.0), // GTA-style 0.8x
                    collision_groups: CollisionGroups::new(
                        self.config.physics.vehicle_group,
                        self.config.physics.static_group
                            | self.config.physics.vehicle_group
                            | self.config.physics.character_group,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 450.0..500.0,
                        use_aabb: false,
                    },
                },
                Car,
                VehicleState::new(VehicleType::SuperCar),
                LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
                Damping {
                    linear_damping: 1.0,
                    angular_damping: 5.0,
                },
                MovementTracker::new(position, 10.0),
                Name::new("SuperCar"),
            ))
            .id();

        // Add car body as child entity with proper mesh size
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.9, 1.3, 4.7))), // Full visual size
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
        ));

        Ok(vehicle_entity)
    }

    /// Spawn Helicopter with proper mesh-collider consistency
    /// Mesh: 3×3×12, Collider: 1.2×1.2×4.8 (0.8x for GTA-style forgiving collision)
    pub fn spawn_helicopter(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        let color = color.unwrap_or(Color::srgb(0.2, 0.2, 0.8));

        let vehicle_entity = commands
            .spawn((
                DynamicPhysicsBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Vehicle,
                    },
                    transform: Transform::from_translation(position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::cuboid(1.2 / 2.0, 1.2 / 2.0, 4.8 / 2.0), // GTA-style 0.8x
                    collision_groups: CollisionGroups::new(
                        self.config.physics.vehicle_group,
                        self.config.physics.static_group | self.config.physics.vehicle_group,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 600.0..700.0,
                        use_aabb: false,
                    },
                },
                Helicopter,
                VehicleState::new(VehicleType::Helicopter),
                Damping {
                    linear_damping: 2.0,
                    angular_damping: 8.0,
                },
                MovementTracker::new(position, 15.0),
                Name::new("Helicopter"),
            ))
            .id();

        // Add helicopter body as child entity
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(3.0, 3.0, 12.0))), // Full visual size
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
        ));

        Ok(vehicle_entity)
    }

    /// Spawn F16 Fighter Jet with capsule collider
    /// Mesh: 15×5×10, Collider: capsule 6.0 radius × 4.0 half-height (0.8x)
    pub fn spawn_f16(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        let color = color.unwrap_or(Color::srgb(0.5, 0.5, 0.5));

        let vehicle_entity = commands
            .spawn((
                DynamicPhysicsBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Vehicle,
                    },
                    transform: Transform::from_translation(position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::capsule_z(6.0, 4.0), // GTA-style capsule for aircraft
                    collision_groups: CollisionGroups::new(
                        self.config.physics.vehicle_group,
                        self.config.physics.static_group | self.config.physics.vehicle_group,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 800.0..1000.0,
                        use_aabb: false,
                    },
                },
                F16,
                VehicleState::new(VehicleType::F16),
                Damping {
                    linear_damping: 0.5,
                    angular_damping: 3.0,
                },
                MovementTracker::new(position, 25.0),
                Name::new("F16"),
            ))
            .id();

        // Add F16 body as child entity
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(15.0, 5.0, 10.0))), // Full visual size
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
        ));

        Ok(vehicle_entity)
    }

    /// Spawn Yacht with intentionally smaller collider for water navigation
    /// Mesh: 8×2×20, Collider: 4×1×10 (0.5x for boats - smaller for easier navigation)
    pub fn spawn_yacht(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        let color = color.unwrap_or(Color::srgb(1.0, 1.0, 1.0));

        let vehicle_entity = commands
            .spawn((
                DynamicPhysicsBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Vehicle,
                    },
                    transform: Transform::from_translation(position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::cuboid(4.0 / 2.0, 1.0 / 2.0, 10.0 / 2.0), // 0.5x for boats
                    collision_groups: CollisionGroups::new(
                        self.config.physics.vehicle_group,
                        self.config.physics.static_group,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 500.0..600.0,
                        use_aabb: false,
                    },
                },
                Yacht::default(),
                VehicleState::new(VehicleType::SuperCar), // Use SuperCar for yacht physics
                Damping {
                    linear_damping: 3.0,
                    angular_damping: 10.0,
                },
                MovementTracker::new(position, 12.0),
                Name::new("Yacht"),
            ))
            .id();

        // Add yacht body as child entity
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(8.0, 2.0, 20.0))), // Full visual size
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
        ));

        Ok(vehicle_entity)
    }

    /// Spawn vehicle by type with automatic configuration
    pub fn spawn_vehicle_by_type(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vehicle_type: VehicleType,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        match vehicle_type {
            VehicleType::SuperCar => {
                self.spawn_super_car(commands, meshes, materials, position, color)
            }
            VehicleType::Helicopter => {
                self.spawn_helicopter(commands, meshes, materials, position, color)
            }
            VehicleType::F16 => self.spawn_f16(commands, meshes, materials, position, color),
        }
    }

    /// Spawn yacht directly (not part of VehicleType enum)
    pub fn spawn_yacht_direct(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        self.spawn_yacht(commands, meshes, materials, position, color)
    }

    /// Generate random car color
    fn random_car_color(&self) -> Color {
        let mut rng = rand::thread_rng();
        let car_colors = [
            Color::srgb(1.0, 0.0, 0.0), // Red
            Color::srgb(0.0, 0.0, 1.0), // Blue
            Color::srgb(0.0, 1.0, 0.0), // Green
            Color::srgb(1.0, 1.0, 0.0), // Yellow
            Color::srgb(1.0, 0.0, 1.0), // Magenta
            Color::srgb(0.0, 1.0, 1.0), // Cyan
            Color::srgb(0.5, 0.5, 0.5), // Gray
            Color::srgb(1.0, 1.0, 1.0), // White
            Color::srgb(0.0, 0.0, 0.0), // Black
        ];
        car_colors[rng.gen_range(0..car_colors.len())]
    }
}

impl Default for VehicleFactory {
    fn default() -> Self {
        Self::new()
    }
}
