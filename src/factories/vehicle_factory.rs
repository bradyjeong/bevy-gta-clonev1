use crate::bundles::{DynamicPhysicsBundle, VisibleChildBundle};
use crate::components::MovementTracker;
use crate::components::water::Yacht;
use crate::components::water_new::WaterBodyId;
use crate::components::{
    AircraftFlight, Car, ContentType, DynamicContent, F16, Helicopter, JetFlame, MainRotor,
    SimpleCarSpecs, SimpleF16Specs, SimpleHelicopterSpecs, TailRotor, VehicleState, VehicleType,
};
use crate::config::GameConfig;
use crate::factories::generic_bundle::BundleError;
use crate::factories::{MaterialFactory, MeshFactory};
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
                        end_margin: 450.0..550.0,
                        use_aabb: false,
                    },
                },
                Car,
                VehicleState::new(VehicleType::SuperCar),
                SimpleCarSpecs::default(),
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
        // VisibilityRange required on each mesh entity (doesn't inherit per Bevy docs)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.9, 1.3, 4.7))), // Full visual size
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
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
        _color: Option<Color>,
    ) -> Result<Entity, BundleError> {
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
                        end_margin: 450.0..550.0,
                        use_aabb: false,
                    },
                },
                Helicopter,
                VehicleState::new(VehicleType::Helicopter),
                SimpleHelicopterSpecs::default(),
                Damping {
                    linear_damping: 2.0,
                    angular_damping: 8.0,
                },
                MovementTracker::new(position, 15.0),
                Name::new("Helicopter"),
            ))
            .id();

        // Helicopter body - Realistic shape using capsule
        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.8, 4.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.25, 0.28, 0.3), // Military gunmetal
                metallic: 0.8,
                perceptual_roughness: 0.4,
                reflectance: 0.3,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Cockpit bubble - rounded cockpit
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.05, 0.05, 0.08, 0.15),
                metallic: 0.1,
                perceptual_roughness: 0.1,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.2, 1.5).with_scale(Vec3::new(1.2, 0.8, 1.0)),
            ChildOf(vehicle_entity),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Tail boom - tapered cylinder
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.25, 3.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.25, 0.28, 0.3),
                metallic: 0.8,
                perceptual_roughness: 0.4,
                reflectance: 0.3,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 4.5),
            ChildOf(vehicle_entity),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Main rotor blades - thin and aerodynamic
        for i in 0..4 {
            let angle = i as f32 * std::f32::consts::PI / 2.0;
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(8.0, 0.02, 0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.08, 0.08, 0.08),
                    metallic: 0.2,
                    perceptual_roughness: 0.9,
                    ..default()
                })),
                Transform::from_xyz(0.0, 2.2, 0.0).with_rotation(Quat::from_rotation_y(angle)),
                ChildOf(vehicle_entity),
                MainRotor,
                VisibilityRange {
                    start_margin: 0.0..0.0,
                    end_margin: 450.0..550.0,
                    use_aabb: false,
                },
            ));
        }

        // Landing skids - long narrow cylinders
        for x in [-0.8, 0.8] {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.04, 3.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.35, 0.35, 0.35),
                    metallic: 0.7,
                    perceptual_roughness: 0.6,
                    ..default()
                })),
                Transform::from_xyz(x, -1.0, 0.0)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                ChildOf(vehicle_entity),
                VisibilityRange {
                    start_margin: 0.0..0.0,
                    end_margin: 450.0..550.0,
                    use_aabb: false,
                },
            ));
        }

        // Tail rotor at end of tail boom
        commands.spawn((
            Mesh3d(MeshFactory::create_tail_rotor(meshes)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.08, 0.08, 0.08),
                metallic: 0.2,
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform::from_xyz(0.0, 1.0, 6.2),
            ChildOf(vehicle_entity),
            TailRotor,
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
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
        _color: Option<Color>,
    ) -> Result<Entity, BundleError> {
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
                    collider: Collider::capsule_z(10.0, 0.9), // half_height=10.0, radius=0.9 along Z-axis
                    collision_groups: CollisionGroups::new(
                        self.config.physics.vehicle_group,
                        self.config.physics.static_group | self.config.physics.vehicle_group,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 450.0..550.0,
                        use_aabb: false,
                    },
                },
                F16,
                VehicleState::new(VehicleType::F16),
                AircraftFlight::default(),
                SimpleF16Specs::default(),
                Damping {
                    linear_damping: 0.5,
                    angular_damping: 3.0,
                },
                MovementTracker::new(position, 25.0),
                Name::new("F16"),
            ))
            .id();

        // Part 1: Fuselage - using dedicated F16 mesh factory, rotated horizontal
        let fuselage_mesh = MeshFactory::create_f16_body(meshes);
        let fuselage_material = MaterialFactory::create_f16_fuselage_material(materials);

        commands.spawn((
            Mesh3d(fuselage_mesh),
            MeshMaterial3d(fuselage_material),
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Part 2: Left Wing - Large delta wing with aggressive sweep
        let wing_mesh = MeshFactory::create_f16_wing(meshes);
        let wing_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.27, 0.30), // Darker gray for wings
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        });

        commands.spawn((
            Mesh3d(wing_mesh.clone()),
            MeshMaterial3d(wing_material.clone()),
            Transform::from_xyz(-4.0, -0.2, 1.0)
                .with_rotation(Quat::from_rotation_y(-0.25) * Quat::from_rotation_z(-0.05)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Part 3: Right Wing - Large delta wing with aggressive sweep (mirror)
        commands.spawn((
            Mesh3d(wing_mesh),
            MeshMaterial3d(wing_material),
            Transform::from_xyz(4.0, -0.2, 1.0)
                .with_rotation(Quat::from_rotation_y(0.25) * Quat::from_rotation_z(0.05)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Part 4: Canopy (transparent cockpit bubble) - Forward position for sleek look
        let canopy_mesh = MeshFactory::create_f16_canopy(meshes);
        let canopy_material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.15, 0.2, 0.6), // Dark tinted blue
            alpha_mode: AlphaMode::Blend,
            metallic: 0.9,
            perceptual_roughness: 0.1,
            reflectance: 0.8,
            ..default()
        });

        commands.spawn((
            Mesh3d(canopy_mesh),
            MeshMaterial3d(canopy_material),
            Transform::from_xyz(0.0, 1.5, -5.0).with_scale(Vec3::new(1.2, 1.0, 1.5)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Part 5: Vertical Tail (tail fin) - Taller and swept with accent color
        let tail_mesh = MeshFactory::create_f16_vertical_tail(meshes);
        let tail_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.55, 0.60, 0.65), // Lighter accent for tail
            metallic: 0.85,
            perceptual_roughness: 0.25,
            ..default()
        });

        commands.spawn((
            Mesh3d(tail_mesh),
            MeshMaterial3d(tail_material),
            Transform::from_xyz(0.0, 2.2, 7.5).with_rotation(Quat::from_rotation_x(0.1)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Part 6: Left Air Intake - Signature F16 side intake
        let intake_mesh = MeshFactory::create_f16_air_intake(meshes);
        let intake_material = MaterialFactory::create_f16_intake_material(materials);

        commands.spawn((
            Mesh3d(intake_mesh.clone()),
            MeshMaterial3d(intake_material.clone()),
            Transform::from_xyz(-1.5, -0.5, -2.0).with_rotation(Quat::from_rotation_y(1.57)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Part 7: Right Air Intake - Signature F16 side intake (mirror)
        commands.spawn((
            Mesh3d(intake_mesh),
            MeshMaterial3d(intake_material),
            Transform::from_xyz(1.5, -0.5, -2.0).with_rotation(Quat::from_rotation_y(-1.57)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Part 8: Engine Nozzle (rear thrust visual)
        let engine_mesh = meshes.add(Cylinder::new(0.9, 2.5));
        let engine_material = MaterialFactory::create_f16_engine_material(materials);

        commands.spawn((
            Mesh3d(engine_mesh),
            MeshMaterial3d(engine_material),
            Transform::from_xyz(0.0, 0.0, 9.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));

        // Part 9: Jet Flames (afterburner exhaust effects)
        let flame_mesh = meshes.add(Cone {
            radius: 0.6,
            height: 2.5,
        });
        let flame_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.5, 0.2),
            emissive: LinearRgba::rgb(1.0, 0.3, 0.0),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            Mesh3d(flame_mesh),
            MeshMaterial3d(flame_material),
            Transform::from_xyz(0.0, 0.0, 10.5)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            JetFlame {
                intensity: 0.0,
                base_scale: 0.5,
                max_scale: 2.0,
                flicker_speed: 15.0,
                color_intensity: 1.0,
            },
            Visibility::Hidden,
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
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
                VehicleState::new(VehicleType::Yacht),
                WaterBodyId, // Mark yacht for water physics
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
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
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
            VehicleType::Yacht => self.spawn_yacht(commands, meshes, materials, position, color),
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
