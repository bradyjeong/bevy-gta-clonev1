use crate::bundles::{DynamicPhysicsBundle, VisibleChildBundle};
use crate::components::MovementTracker;
use crate::components::unified_water::WaterBodyId;
use crate::components::water::{Yacht, YachtSpecs, YachtState};
use crate::components::{
    AircraftFlight, Car, ContentType, DynamicContent, F16, Helicopter, HelicopterRuntime, JetFlame,
    LandingLight, MainRotor, NavigationLight, NavigationLightType, RotorBlurDisk, RotorWash,
    SimpleCarSpecs, SimpleCarSpecsHandle, SimpleF16Specs, SimpleF16SpecsHandle,
    SimpleHelicopterSpecs, SimpleHelicopterSpecsHandle, TailRotor, VehicleState, VehicleType,
};
use crate::config::GameConfig;
use crate::factories::generic_bundle::BundleError;
use crate::factories::{MaterialFactory, MeshFactory};
use crate::systems::movement::simple_yacht::YachtSpecsHandle;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_rapier3d::dynamics::AdditionalMassProperties;
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

    /// Get visibility range for vehicles based on config with ±10% variance
    fn visibility_range(&self) -> VisibilityRange {
        let distance = self.config.performance.vehicle_visibility_distance;
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: (distance * 0.9)..(distance * 1.1),
            use_aabb: false,
        }
    }

    /// Spawn SuperCar with multi-part realistic geometry
    /// Visual: 1.8×1.25×4.2, Collider: 0.72×0.5×1.68 (0.8x for GTA-style forgiving collision)
    pub fn spawn_super_car(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        let color = color.unwrap_or_else(|| self.random_car_color());

        // Load car specs asset following YachtSpecs pattern
        let car_specs_handle: Handle<SimpleCarSpecs> = asset_server.load("config/simple_car.ron");

        let vehicle_entity = commands
            .spawn((
                DynamicPhysicsBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Vehicle,
                    },
                    transform: Transform::from_translation(position + Vec3::new(0.0, 0.125, 0.0)),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::cuboid(0.72, 0.5, 1.68), // 0.8x of visual bounds: width 0.9, height 0.625, length 2.1
                    collision_groups: CollisionGroups::new(
                        self.config.physics.vehicle_group,
                        self.config.physics.static_group
                            | self.config.physics.vehicle_group
                            | self.config.physics.character_group,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: self.visibility_range(),
                },
                Car,
                VehicleState::new(VehicleType::SuperCar),
                SimpleCarSpecsHandle(car_specs_handle),
                LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
                Ccd::enabled(), // High-speed cars need continuous collision detection
                Damping {
                    linear_damping: 0.2,  // Reduced to avoid double-damping with custom grip
                    angular_damping: 2.0, // Reduced for better steering responsiveness
                },
                Friction::coefficient(0.2), // Low friction to avoid conflicting with custom lateral grip
                MovementTracker::new(position, 10.0),
                Name::new("SuperCar"),
            ))
            .id();

        // Build realistic multi-part car
        let body_color = materials.add(color);
        let dark_color = materials.add(Color::srgb(0.1, 0.1, 0.12));
        let glass_color = materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.3, 0.4, 0.6),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.1,
            metallic: 0.0,
            ..default()
        });

        // Lower chassis (wider, longer)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 0.6, 4.2))),
            MeshMaterial3d(body_color.clone()),
            Transform::from_xyz(0.0, -0.2, 0.0), // Bottom at collider bottom: -0.5 + 0.3
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            self.visibility_range(),
        ));

        // Upper cabin (narrower, sleeker)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.6, 0.7, 2.0))),
            MeshMaterial3d(body_color.clone()),
            Transform::from_xyz(0.0, 0.275, -0.3), // Offset from collider center
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            self.visibility_range(),
        ));

        // Windshield
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.5, 0.5, 0.1))),
            MeshMaterial3d(glass_color.clone()),
            Transform::from_xyz(0.0, 0.375, 0.7), // Offset from collider center
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            self.visibility_range(),
        ));

        // Hood (front slope)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.7, 0.3, 1.2))),
            MeshMaterial3d(body_color.clone()),
            Transform::from_xyz(0.0, -0.125, 1.5), // Front aligns with chassis front at 2.1
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            self.visibility_range(),
        ));

        // 4 Wheels (positioned at collider bottom: -0.5 + wheel_radius 0.25)
        let wheel_mesh = MeshFactory::create_sports_wheel(meshes);
        let wheel_y = -0.5 + 0.25; // collider_bottom (-half_height) + wheel_radius
        let wheel_positions = [
            Vec3::new(-0.9, wheel_y, 1.2),  // Front left
            Vec3::new(0.9, wheel_y, 1.2),   // Front right
            Vec3::new(-0.9, wheel_y, -1.2), // Rear left
            Vec3::new(0.9, wheel_y, -1.2),  // Rear right
        ];

        for pos in wheel_positions {
            commands.spawn((
                Mesh3d(wheel_mesh.clone()),
                MeshMaterial3d(dark_color.clone()),
                Transform::from_translation(pos)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                ChildOf(vehicle_entity),
                VisibleChildBundle::default(),
                self.visibility_range(),
            ));
        }

        Ok(vehicle_entity)
    }

    /// Spawn Helicopter with proper mesh-collider consistency
    /// Mesh: 3×3×12, Collider: 1.2×1.2×4.8 (0.8x for GTA-style forgiving collision)
    pub fn spawn_helicopter(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        position: Vec3,
        _color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        // Load helicopter specs asset following YachtSpecs pattern
        let heli_specs_handle: Handle<SimpleHelicopterSpecs> =
            asset_server.load("config/simple_helicopter.ron");

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
                    collider: Collider::cuboid(1.2, 0.7, 2.5),
                    collision_groups: CollisionGroups::new(
                        self.config.physics.vehicle_group,
                        self.config.physics.static_group | self.config.physics.vehicle_group,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: self.visibility_range(),
                },
                Helicopter,
                VehicleState::new(VehicleType::Helicopter),
                SimpleHelicopterSpecsHandle(heli_specs_handle),
                HelicopterRuntime::default(),
                ExternalForce::default(),
                RotorWash,
                Damping {
                    linear_damping: 0.1,
                    angular_damping: 0.3,
                },
                AdditionalMassProperties::Mass(1500.0), // 1.5 ton helicopter
                Sleeping::default(),
                MovementTracker::new(position, 15.0),
                crate::components::Enterable {
                    vehicle_type: crate::components::VehicleControlType::Helicopter,
                },
                Name::new("Helicopter"),
            ))
            .id();

        // Helicopter fuselage - Main cabin (above landing gear)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.2, 1.6, 3.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.18, 0.22, 0.25),
                metallic: 0.85,
                perceptual_roughness: 0.35,
                reflectance: 0.5,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.3, 0.3),
            ChildOf(vehicle_entity),
            self.visibility_range(),
        ));

        // Helicopter nose - Tapered front
        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.5, 0.6))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.18, 0.22, 0.25),
                metallic: 0.85,
                perceptual_roughness: 0.35,
                reflectance: 0.5,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.1, -1.3)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ChildOf(vehicle_entity),
            self.visibility_range(),
        ));

        // Cockpit bubble - glass canopy over cabin
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.15, 0.18, 0.22, 0.3),
                metallic: 0.0,
                perceptual_roughness: 0.08,
                reflectance: 0.9,
                alpha_mode: AlphaMode::Blend,
                ior: 1.5,
                ..default()
            })),
            Transform::from_xyz(0.0, 1.0, -0.3).with_scale(Vec3::new(1.5, 0.8, 1.6)),
            ChildOf(vehicle_entity),
            self.visibility_range(),
        ));

        // Tail boom - horizontal cylinder extending from rear
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.25, 4.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.18, 0.22, 0.25),
                metallic: 0.85,
                perceptual_roughness: 0.35,
                reflectance: 0.5,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.5, 4.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ChildOf(vehicle_entity),
            self.visibility_range(),
        ));

        // Main rotor blades - composite material with matte finish
        for i in 0..4 {
            let angle = i as f32 * std::f32::consts::PI / 2.0;
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(8.0, 0.02, 0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.12, 0.12, 0.14),
                    metallic: 0.15,
                    perceptual_roughness: 0.85,
                    reflectance: 0.2,
                    ..default()
                })),
                Transform::from_xyz(0.0, 1.6, 0.0).with_rotation(Quat::from_rotation_y(angle)),
                ChildOf(vehicle_entity),
                MainRotor,
                self.visibility_range(),
            ));
        }

        // Main rotor blur disk - appears when blades spin fast
        commands.spawn((
            Mesh3d(meshes.add(Circle::new(4.2))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.3, 0.3, 0.35, 0.4),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 1.61, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ChildOf(vehicle_entity),
            RotorBlurDisk {
                min_rpm_for_blur: 10.0,
                is_main_rotor: true,
            },
            Visibility::Hidden,
            self.visibility_range(),
        ));

        // Landing skids - ON THE GROUND underneath helicopter
        for x in [-0.6, 0.6] {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.08, 3.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.55, 0.58, 0.60),
                    metallic: 0.9,
                    perceptual_roughness: 0.4,
                    reflectance: 0.6,
                    ..default()
                })),
                Transform::from_xyz(x, -0.62, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ChildOf(vehicle_entity),
                self.visibility_range(),
            ));
        }

        // Skid struts - vertical supports from cabin down to skids
        for (x, z) in [(-0.6, -1.2), (-0.6, 1.2), (0.6, -1.2), (0.6, 1.2)] {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.04, 0.8))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.55, 0.58, 0.60),
                    metallic: 0.9,
                    perceptual_roughness: 0.4,
                    reflectance: 0.6,
                    ..default()
                })),
                Transform::from_xyz(x, -0.22, z),
                ChildOf(vehicle_entity),
                self.visibility_range(),
            ));
        }

        // Cross-braces between skids for structural strength
        for z in [-1.2, 0.0, 1.2] {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.03, 1.2))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.55, 0.58, 0.60),
                    metallic: 0.9,
                    perceptual_roughness: 0.4,
                    reflectance: 0.6,
                    ..default()
                })),
                Transform::from_xyz(0.0, -0.62, z)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                ChildOf(vehicle_entity),
                self.visibility_range(),
            ));
        }

        // Tail rotor at end of tail boom (vertical orientation)
        commands.spawn((
            Mesh3d(MeshFactory::create_tail_rotor(meshes)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.14),
                metallic: 0.15,
                perceptual_roughness: 0.85,
                reflectance: 0.2,
                ..default()
            })),
            Transform::from_xyz(0.8, 0.3, 7.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ChildOf(vehicle_entity),
            TailRotor,
            self.visibility_range(),
        ));

        // Tail rotor blur disk (vertical plane)
        commands.spawn((
            Mesh3d(meshes.add(Circle::new(0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.3, 0.3, 0.35, 0.5),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.82, 0.3, 7.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ChildOf(vehicle_entity),
            RotorBlurDisk {
                min_rpm_for_blur: 15.0,
                is_main_rotor: false,
            },
            TailRotor,
            Visibility::Hidden,
            self.visibility_range(),
        ));

        // NAVIGATION LIGHTS - Red port (left), Green starboard (right)
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.0, 0.0),
                intensity: 50000.0,
                range: 12.0,
                radius: 0.1,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(-1.3, 0.5, -1.0),
            ChildOf(vehicle_entity),
            NavigationLight::new(NavigationLightType::RedPort),
            self.visibility_range(),
        ));

        commands.spawn((
            PointLight {
                color: Color::srgb(0.0, 1.0, 0.0),
                intensity: 50000.0,
                range: 12.0,
                radius: 0.1,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(1.3, 0.5, -1.0),
            ChildOf(vehicle_entity),
            NavigationLight::new(NavigationLightType::GreenStarboard),
            self.visibility_range(),
        ));

        // White tail light (blinking)
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 1.0, 1.0),
                intensity: 80000.0,
                range: 15.0,
                radius: 0.15,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.0, 0.5, 7.2),
            ChildOf(vehicle_entity),
            NavigationLight::new(NavigationLightType::WhiteTail),
            self.visibility_range(),
        ));

        // Red anti-collision beacon on top of cabin (blinking)
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.0, 0.0),
                intensity: 100000.0,
                range: 20.0,
                radius: 0.2,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.0, 1.8, 0.0),
            ChildOf(vehicle_entity),
            NavigationLight::new(NavigationLightType::RedBeacon),
            self.visibility_range(),
        ));

        // Forward landing spotlights - illuminate ground when low altitude
        for x in [-0.8, 0.8] {
            commands.spawn((
                SpotLight {
                    color: Color::srgb(1.0, 0.95, 0.85),
                    intensity: 0.0,
                    range: 40.0,
                    radius: 0.3,
                    shadows_enabled: true,
                    inner_angle: 0.4,
                    outer_angle: 0.7,
                    ..default()
                },
                Transform::from_xyz(x, -0.6, -1.8).looking_at(Vec3::new(x, -5.0, 0.0), Vec3::Y),
                ChildOf(vehicle_entity),
                LandingLight {
                    activation_altitude: 25.0,
                },
                self.visibility_range(),
            ));
        }

        Ok(vehicle_entity)
    }

    /// Spawn F16 Fighter Jet with capsule collider
    /// Mesh: 15×5×10, Collider: capsule 6.0 radius × 4.0 half-height (0.8x)
    pub fn spawn_f16(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        position: Vec3,
        _color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        // Load F16 specs asset following YachtSpecs pattern
        let f16_specs_handle: Handle<SimpleF16Specs> = asset_server.load("config/simple_f16.ron");

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
                    visibility_range: self.visibility_range(),
                },
                Ccd::enabled(), // Enable CCD for high-speed F16 to prevent tunneling
                F16,
                VehicleState::new(VehicleType::F16),
                AircraftFlight::default(),
                SimpleF16SpecsHandle(f16_specs_handle),
                Damping {
                    linear_damping: 0.5,
                    angular_damping: 3.0,
                },
                Sleeping::default(),
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
            self.visibility_range(),
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
            self.visibility_range(),
        ));

        // Part 3: Right Wing - Large delta wing with aggressive sweep (mirror)
        commands.spawn((
            Mesh3d(wing_mesh),
            MeshMaterial3d(wing_material),
            Transform::from_xyz(4.0, -0.2, 1.0)
                .with_rotation(Quat::from_rotation_y(0.25) * Quat::from_rotation_z(0.05)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            self.visibility_range(),
        ));

        // Part 4: Canopy (transparent cockpit bubble) - Forward position for sleek look
        let canopy_mesh = MeshFactory::create_f16_canopy(meshes);
        let canopy_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.15, 0.2), // Dark tinted blue
            alpha_mode: AlphaMode::Opaque,
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
            self.visibility_range(),
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
            self.visibility_range(),
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
            self.visibility_range(),
        ));

        // Part 7: Right Air Intake - Signature F16 side intake (mirror)
        commands.spawn((
            Mesh3d(intake_mesh),
            MeshMaterial3d(intake_material),
            Transform::from_xyz(1.5, -0.5, -2.0).with_rotation(Quat::from_rotation_y(-1.57)),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            self.visibility_range(),
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
            self.visibility_range(),
        ));

        // Part 9: Jet Flames (afterburner exhaust effects)
        let flame_mesh = meshes.add(Cone {
            radius: 0.6,
            height: 2.5,
        });
        let flame_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.5, 0.2),
            emissive: LinearRgba::rgb(1.0, 0.3, 0.0),
            alpha_mode: AlphaMode::Opaque,
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
            self.visibility_range(),
        ));

        Ok(vehicle_entity)
    }

    /// Spawn Superyacht with multi-deck design and helipad
    /// Uses config.vehicles.yacht for mesh and collider dimensions
    pub fn spawn_yacht(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &AssetServer,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        let yacht_config = &self.config.vehicles.yacht;
        let hull_color = color.unwrap_or(yacht_config.default_color);
        let yacht_specs_handle: Handle<YachtSpecs> = asset_server.load("config/simple_yacht.ron");
        let yacht_visibility =
            || VisibilityRange::abrupt(0.0, self.config.performance.vehicle_visibility_distance);

        let hull_collider = Collider::cuboid(
            yacht_config.collider_size.x,
            yacht_config.collider_size.y,
            yacht_config.collider_size.z,
        );

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
                    collider: hull_collider,
                    collision_groups: CollisionGroups::new(
                        self.config.physics.vehicle_group,
                        self.config.physics.static_group
                            | self.config.physics.vehicle_group
                            | self.config.physics.character_group,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: yacht_visibility(),
                },
                Yacht::default(),
                YachtState::default(),
                VehicleState::new(VehicleType::Yacht),
                WaterBodyId,
                YachtSpecsHandle(yacht_specs_handle),
                ExternalForce::default(),
                Ccd::enabled(),
                Damping {
                    linear_damping: yacht_config.linear_damping,
                    angular_damping: yacht_config.angular_damping,
                },
                MovementTracker::new(position, 12.0),
                Name::new("Superyacht"),
            ))
            .id();

        let deck_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.85, 0.85),
            metallic: 0.3,
            perceptual_roughness: 0.7,
            ..default()
        });

        // Hull mesh using yacht config body_size
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                yacht_config.body_size.x,
                yacht_config.body_size.y,
                yacht_config.body_size.z,
            ))),
            MeshMaterial3d(materials.add(hull_color)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Yacht Hull"),
        ));

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(18.0, 1.0, 40.0))),
            MeshMaterial3d(deck_material.clone()),
            Transform::from_xyz(0.0, 3.5, 0.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Main Deck"),
        ));

        // Helipad markings (flat layered decals on deck surface at y=4.0)
        let deck_gray = materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.75),
            metallic: 0.1,
            perceptual_roughness: 0.9,
            ..default()
        });

        let helipad_white = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.95, 0.98),
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        });

        let helipad_red = materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.1, 0.1),
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        });

        // Base disc
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(5.4, 0.02))),
            MeshMaterial3d(deck_gray.clone()),
            Transform::from_xyz(0.0, 4.05, 10.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Helipad Base Disc"),
        ));

        // White ring outer
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(5.2, 0.015))),
            MeshMaterial3d(helipad_white.clone()),
            Transform::from_xyz(0.0, 4.06, 10.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Helipad Ring Outer"),
        ));

        // Ring inner mask
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(4.4, 0.018))),
            MeshMaterial3d(deck_gray.clone()),
            Transform::from_xyz(0.0, 4.062, 10.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Helipad Ring Inner Mask"),
        ));

        // "H" letter - left bar
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.35, 0.02, 2.2))),
            MeshMaterial3d(helipad_red.clone()),
            Transform::from_xyz(-0.55, 4.07, 10.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Helipad H - Left Bar"),
        ));

        // "H" letter - right bar
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.35, 0.02, 2.2))),
            MeshMaterial3d(helipad_red.clone()),
            Transform::from_xyz(0.55, 4.07, 10.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Helipad H - Right Bar"),
        ));

        // "H" letter - crossbar
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.5, 0.02, 0.35))),
            MeshMaterial3d(helipad_red),
            Transform::from_xyz(0.0, 4.07, 10.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Helipad H - Crossbar"),
        ));

        let accent_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.25),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        });

        let mast_groups = CollisionGroups::new(
            self.config.physics.vehicle_group,
            self.config.physics.character_group,
        );

        for (x, z) in [(-9.5, 25.0), (9.5, 25.0), (-9.5, -25.0), (9.5, -25.0)] {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.08, 1.5))),
                MeshMaterial3d(accent_material.clone()),
                Transform::from_xyz(x, 4.25, z),
                Collider::cylinder(0.75, 0.08),
                mast_groups,
                ChildOf(vehicle_entity),
                VisibleChildBundle::default(),
                yacht_visibility(),
                Name::new("Railing Post"),
            ));
        }

        let steel_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.86, 0.9),
            metallic: 1.0,
            perceptual_roughness: 0.2,
            reflectance: 0.5,
            ..default()
        });

        let rail_specs = [
            (4.25, 0.055, "Top"),
            (3.95, 0.04, "Mid"),
            (3.65, 0.035, "Low"),
        ];

        for (y, radius, tag) in rail_specs {
            let length_x = 19.0;
            let length_z = 50.0;

            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(radius, length_z))),
                MeshMaterial3d(steel_material.clone()),
                Transform::from_xyz(-9.5, y, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                Collider::cylinder(length_z / 2.0, radius),
                mast_groups,
                ChildOf(vehicle_entity),
                VisibleChildBundle::default(),
                yacht_visibility(),
                Name::new(format!("{tag} Rail Left")),
            ));

            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(radius, length_z))),
                MeshMaterial3d(steel_material.clone()),
                Transform::from_xyz(9.5, y, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                Collider::cylinder(length_z / 2.0, radius),
                mast_groups,
                ChildOf(vehicle_entity),
                VisibleChildBundle::default(),
                yacht_visibility(),
                Name::new(format!("{tag} Rail Right")),
            ));

            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(radius, length_x))),
                MeshMaterial3d(steel_material.clone()),
                Transform::from_xyz(0.0, y, -25.0)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                Collider::cylinder(length_x / 2.0, radius),
                mast_groups,
                ChildOf(vehicle_entity),
                VisibleChildBundle::default(),
                yacht_visibility(),
                Name::new(format!("{tag} Rail Front")),
            ));

            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(radius, length_x))),
                MeshMaterial3d(steel_material.clone()),
                Transform::from_xyz(0.0, y, 25.0)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                Collider::cylinder(length_x / 2.0, radius),
                mast_groups,
                ChildOf(vehicle_entity),
                VisibleChildBundle::default(),
                yacht_visibility(),
                Name::new(format!("{tag} Rail Back")),
            ));
        }

        commands.spawn((
            Transform::from_xyz(0.0, 4.0, 0.0),
            ChildOf(vehicle_entity),
            crate::components::DeckWalkAnchor,
            Name::new("DeckWalkAnchor"),
        ));

        commands.spawn((
            Transform::from_xyz(0.0, 4.0, 0.0),
            Collider::cuboid(8.5, 0.02, 19.0),
            Sensor,
            CollisionGroups::new(
                self.config.physics.vehicle_group,
                self.config.physics.character_group,
            ),
            Friction::coefficient(0.0),
            Restitution::coefficient(0.0),
            ChildOf(vehicle_entity),
            crate::components::DeckWalkable,
            Name::new("DeckWalkVolume"),
        ));

        commands.spawn((
            Transform::from_xyz(0.0, 5.5, 10.0), // Detection zone above deck surface
            Collider::cuboid(6.0, 3.0, 6.0),     // 12m × 6m height × 12m
            Sensor,
            ChildOf(vehicle_entity),
            crate::components::Helipad,
            Name::new("HelipadVolume"),
        ));

        commands.spawn((
            Transform::from_xyz(0.0, 3.5, 5.0),
            ChildOf(vehicle_entity),
            crate::components::ExitPoint {
                kind: crate::components::ExitPointKind::Deck,
            },
            Name::new("ExitPoint Deck"),
        ));

        // Water exit points should be BELOW sea level (SEA_LEVEL = 0.0) to trigger swimming
        commands.spawn((
            Transform::from_xyz(11.0, -0.5, 0.0),
            ChildOf(vehicle_entity),
            crate::components::ExitPoint {
                kind: crate::components::ExitPointKind::Water,
            },
            Name::new("ExitPoint Water Starboard"),
        ));

        commands.spawn((
            Transform::from_xyz(-11.0, -0.5, 0.0),
            ChildOf(vehicle_entity),
            crate::components::ExitPoint {
                kind: crate::components::ExitPointKind::Water,
            },
            Name::new("ExitPoint Water Port"),
        ));

        // PROPELLER ASSEMBLY (Decorative - positioned at stern below waterline)
        let prop_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35),
            metallic: 0.9,
            perceptual_roughness: 0.3,
            ..default()
        });

        let prop_hub_entity = commands
            .spawn((
                Mesh3d(meshes.add(Cylinder::new(0.25, 0.4))),
                MeshMaterial3d(prop_material.clone()),
                Transform::from_xyz(0.0, -1.0, 29.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ChildOf(vehicle_entity),
                VisibleChildBundle::default(),
                yacht_visibility(),
                crate::components::PropellerHub,
                Name::new("Propeller Hub"),
            ))
            .id();

        for i in 0..3 {
            let angle = (i as f32) * (2.0 * std::f32::consts::PI / 3.0);
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.05, 1.2, 0.3))),
                MeshMaterial3d(prop_material.clone()),
                Transform::from_xyz(angle.sin() * 0.6, 0.0, angle.cos() * 0.6)
                    .with_rotation(Quat::from_rotation_y(angle)),
                ChildOf(prop_hub_entity),
                VisibleChildBundle::default(),
                yacht_visibility(),
                Name::new(format!("Propeller Blade {}", i + 1)),
            ));
        }

        // RUDDER (Decorative control surface behind propeller)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 1.2, 1.0))),
            MeshMaterial3d(prop_material.clone()),
            Transform::from_xyz(0.0, -0.5, 29.5),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Rudder"),
        ));

        // BRIDGE/SUPERSTRUCTURE (Command center with windows)
        let bridge_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 0.92),
            metallic: 0.2,
            perceptual_roughness: 0.6,
            ..default()
        });

        let window_material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.3, 0.4, 0.7),
            metallic: 0.8,
            perceptual_roughness: 0.1,
            alpha_mode: bevy::prelude::AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(8.0, 2.5, 14.0))),
            MeshMaterial3d(bridge_material.clone()),
            Transform::from_xyz(0.0, 4.25, -15.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Bridge Structure"),
        ));

        for x in [-2.5, 2.5] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.8, 1.8, 0.05))),
                MeshMaterial3d(window_material.clone()),
                Transform::from_xyz(x, 4.25, -8.0),
                ChildOf(vehicle_entity),
                VisibleChildBundle::default(),
                yacht_visibility(),
                Name::new(format!(
                    "Bridge Window {}",
                    if x < 0.0 { "Port" } else { "Starboard" }
                )),
            ));
        }

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(6.0, 1.8, 0.05))),
            MeshMaterial3d(window_material.clone()),
            Transform::from_xyz(0.0, 4.25, -22.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Bridge Front Windscreen"),
        ));

        // NAVIGATION LIGHTS (Port red, Starboard green, Stern white)
        let red_light_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.1, 0.1),
            emissive: LinearRgba::new(2.0, 0.2, 0.2, 1.0),
            unlit: true,
            ..default()
        });

        let green_light_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 1.0, 0.1),
            emissive: LinearRgba::new(0.2, 2.0, 0.2, 1.0),
            unlit: true,
            ..default()
        });

        let white_light_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 1.0),
            emissive: LinearRgba::new(3.0, 3.0, 3.0, 1.0),
            unlit: true,
            ..default()
        });

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(red_light_material),
            Transform::from_xyz(-9.0, 4.5, -25.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Nav Light Port Red"),
        ));

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(green_light_material),
            Transform::from_xyz(9.0, 4.5, -25.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Nav Light Starboard Green"),
        ));

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(white_light_material),
            Transform::from_xyz(0.0, 5.0, 29.0),
            ChildOf(vehicle_entity),
            VisibleChildBundle::default(),
            yacht_visibility(),
            Name::new("Nav Light Stern White"),
        ));

        Ok(vehicle_entity)
    }

    /// Spawn vehicle by type with automatic configuration
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_vehicle_by_type(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        vehicle_type: VehicleType,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        match vehicle_type {
            VehicleType::SuperCar => {
                self.spawn_super_car(commands, meshes, materials, asset_server, position, color)
            }
            VehicleType::Helicopter => {
                self.spawn_helicopter(commands, meshes, materials, asset_server, position, color)
            }
            VehicleType::F16 => {
                self.spawn_f16(commands, meshes, materials, asset_server, position, color)
            }
            VehicleType::Yacht => {
                self.spawn_yacht(commands, meshes, materials, asset_server, position, color)
            }
        }
    }

    /// Spawn yacht directly (not part of VehicleType enum)
    pub fn spawn_yacht_direct(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &AssetServer,
        position: Vec3,
        color: Option<Color>,
    ) -> Result<Entity, BundleError> {
        self.spawn_yacht(commands, meshes, materials, asset_server, position, color)
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
