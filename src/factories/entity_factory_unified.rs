use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::bundles::{VisibleChildBundle};
use crate::factories::{MaterialFactory, MeshFactory, TransformFactory};
use crate::factories::generic_bundle::{GenericBundleFactory, BundleError, ColliderShape, ParticleEffectType};
use crate::systems::audio::realistic_vehicle_audio::{VehicleAudioState, VehicleAudioSources};
use crate::systems::distance_cache::MovementTracker;

use crate::GameConfig;

/// Unified Entity Factory - Single point of all entity creation
/// Consolidates EntityFactory, UnifiedEntityFactory, RealisticVehicleFactory functionality
/// Uses builder pattern with fluent interfaces for type-safe entity construction
#[derive(Resource)]
pub struct UnifiedEntityFactory {
    pub config: GameConfig,
}

impl UnifiedEntityFactory {
    /// Create new factory - configuration is accessed via services
    pub fn new() -> Self {
        Self {
            config: GameConfig::default(),
        }
    }
    
    /// Validate position is within world bounds
    pub fn validate_position(&self, position: Vec3) -> Result<Vec3, BundleError> {
        if position.x.abs() > self.config.physics.max_world_coord ||
           position.z.abs() > self.config.physics.max_world_coord {
            return Err(BundleError::PositionOutOfBounds {
                position,
                max_coord: self.config.physics.max_world_coord,
            });
        }
        
        Ok(position.clamp(
            Vec3::splat(self.config.physics.min_world_coord),
            Vec3::splat(self.config.physics.max_world_coord),
        ))
    }
}

/// VEHICLE CREATION METHODS
impl UnifiedEntityFactory {
    /// Spawn complete vehicle entity with all components
    pub fn spawn_vehicle(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vehicle_type: VehicleType,
        position: Vec3,
        color: Color,
    ) -> Result<Entity, BundleError> {
        // Validate position
        let validated_position = self.validate_position(position)?;
        
        // Create bundle using generic system
        let bundle = GenericBundleFactory::vehicle(vehicle_type, validated_position, color, &self.config)?;
        
        // Get vehicle configuration
        let _vehicle_config = match vehicle_type {
            VehicleType::BasicCar => &self.config.vehicles.basic_car,
            VehicleType::SuperCar => &self.config.vehicles.super_car,
            VehicleType::Helicopter => &self.config.vehicles.helicopter,
            VehicleType::F16 => &self.config.vehicles.f16,
        };
        
        // Create visual components using existing factories
        let mesh_handle = match vehicle_type {
            VehicleType::BasicCar => MeshFactory::create_car_body(meshes),
            VehicleType::SuperCar => MeshFactory::create_sports_car_body(meshes),
            VehicleType::Helicopter => MeshFactory::create_helicopter_body(meshes),
            VehicleType::F16 => MeshFactory::create_f16_body(meshes),
        };
        
        let material_handle = MaterialFactory::create_vehicle_metallic(materials, color);
        
        // Spawn main vehicle entity
        let vehicle_entity = commands.spawn((
            bundle,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
        )).id();
        
        // Add vehicle-specific components and children
        match vehicle_type {
            VehicleType::Helicopter => {
                self.add_helicopter_components(commands, vehicle_entity, meshes, materials)?;
            }
            VehicleType::F16 => {
                self.add_f16_components(commands, vehicle_entity, meshes, materials)?;
            }
            _ => {
                self.add_car_components(commands, vehicle_entity, meshes, materials, vehicle_type)?;
            }
        }
        
        Ok(vehicle_entity)
    }
    
    /// Create a realistic vehicle with full physics simulation
    pub fn spawn_realistic_vehicle(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vehicle_type: RealisticVehicleType,
        position: Vec3,
        rotation: Quat,
    ) -> Result<Entity, BundleError> {
        // Validate and clamp position for safety
        let safe_position = self.validate_position(position)?;
        
        // Get vehicle configuration based on type
        let vehicle_config = self.get_realistic_vehicle_configuration(&vehicle_type);
        
        // Create main vehicle entity with basic components
        let vehicle_entity = commands.spawn((
            Transform::from_translation(safe_position).with_rotation(rotation),
            Visibility::default(),
            RigidBody::Dynamic,
            Velocity::default(),
        )).id();
        
        // Add physics components
        commands.entity(vehicle_entity).insert((
            Collider::cuboid(
                vehicle_config.body_size.x / 2.0,
                vehicle_config.body_size.y / 2.0,
                vehicle_config.body_size.z / 2.0,
            ),
            CollisionGroups::new(
                Group::from_bits_truncate(self.config.physics.vehicle_group.bits()),
                Group::from_bits_truncate(self.config.physics.static_group.bits() | self.config.physics.character_group.bits()),
            ),
            AdditionalMassProperties::Mass(vehicle_config.mass),
            Damping {
                linear_damping: vehicle_config.linear_damping,
                angular_damping: vehicle_config.angular_damping,
            },
            Friction {
                coefficient: self.config.physics.ground_friction,
                combine_rule: CoefficientCombineRule::Average,
            },
        ));
        
        // Add visual components
        commands.entity(vehicle_entity).insert((
            Mesh3d(meshes.add(Cuboid::new(
                vehicle_config.body_size.x,
                vehicle_config.body_size.y,
                vehicle_config.body_size.z,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: vehicle_config.default_color,
                metallic: 0.8,
                perceptual_roughness: 0.2,
                ..default()
            })),
        ));
        
        // Add vehicle system components
        commands.entity(vehicle_entity).insert((
            RealisticVehicle {
                vehicle_type: vehicle_type.clone(),
                physics_enabled: true,
                ..default()
            },
            vehicle_config.dynamics.clone(),
            vehicle_config.engine.clone(),
            vehicle_config.suspension.clone(),
            vehicle_config.tire_physics.clone(),
            VehicleAudioState::default(),
            Car,
            Cullable::new(200.0),
            MovementTracker::new(safe_position, 10.0), // Track vehicle movement with 10m threshold
        ));
        
        // Add SuperCar component if needed
        if vehicle_type == RealisticVehicleType::SuperCar {
            commands.entity(vehicle_entity).insert(SuperCar::default());
        }
        
        // Create vehicle wheels with individual physics
        self.create_realistic_vehicle_wheels(commands, meshes, materials, vehicle_entity, &vehicle_type)?;
        
        // Create audio sources for realistic sound
        self.create_vehicle_audio_sources(commands, vehicle_entity)?;
        
        Ok(vehicle_entity)
    }
    
    /// Add car-specific components (wheels, lights, etc.)
    fn add_car_components(
        &self,
        commands: &mut Commands,
        parent_entity: Entity,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vehicle_type: VehicleType,
    ) -> Result<(), BundleError> {
        let wheel_mesh = MeshFactory::create_standard_wheel(meshes);
        let wheel_material = MaterialFactory::create_simple_material(materials, Color::srgb(0.2, 0.2, 0.2));
        
        // Add four wheels using transform factory
        let wheel_positions = match vehicle_type {
            VehicleType::SuperCar => [(-0.9, -0.3, 1.5), (0.9, -0.3, 1.5), (-0.9, -0.3, -1.5), (0.9, -0.3, -1.5)],
            _ => [(-0.8, -0.3, 1.2), (0.8, -0.3, 1.2), (-0.8, -0.3, -1.2), (0.8, -0.3, -1.2)],
        };
        
        for (x, y, z) in wheel_positions {
            commands.spawn((
                Mesh3d(wheel_mesh.clone()),
                MeshMaterial3d(wheel_material.clone()),
                TransformFactory::wheel_with_rotation(x, y, z),
                Cullable { max_distance: self.config.world.lod_distances[1], is_culled: false },
                ChildOf(parent_entity),
            ));
        }
        
        Ok(())
    }
    
    /// Add helicopter-specific components (rotors)
    fn add_helicopter_components(
        &self,
        commands: &mut Commands,
        parent_entity: Entity,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Result<(), BundleError> {
        let rotor_mesh = MeshFactory::create_rotor_blade(meshes);
        let rotor_material = MaterialFactory::create_simple_material(materials, Color::srgb(0.1, 0.1, 0.1));
        
        // Main rotor
        commands.spawn((
            MainRotor,
            Mesh3d(rotor_mesh.clone()),
            MeshMaterial3d(rotor_material.clone()),
            TransformFactory::main_rotor(),
            Cullable { max_distance: self.config.world.lod_distances[1], is_culled: false },
            ChildOf(parent_entity),
        ));
        
        // Tail rotor
        commands.spawn((
            TailRotor,
            Mesh3d(rotor_mesh),
            MeshMaterial3d(rotor_material),
            TransformFactory::tail_rotor(),
            Cullable { max_distance: self.config.world.lod_distances[1], is_culled: false },
            ChildOf(parent_entity),
        ));
        
        Ok(())
    }
    
    /// Add F16-specific components (realistic fighter jet assembly)
    fn add_f16_components(
        &self,
        commands: &mut Commands,
        parent_entity: Entity,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Result<(), BundleError> {
        let lod_distance = self.config.world.lod_distances.get(2).copied().unwrap_or(300.0).clamp(50.0, 1000.0);
        
        // Wings (swept delta configuration)
        let wing_mesh = MeshFactory::create_f16_wing(meshes);
        let wing_material = MaterialFactory::create_f16_fuselage_material(materials);
        
        commands.spawn((
            Mesh3d(wing_mesh.clone()),
            MeshMaterial3d(wing_material.clone()),
            TransformFactory::f16_left_wing(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        commands.spawn((
            Mesh3d(wing_mesh),
            MeshMaterial3d(wing_material.clone()),
            TransformFactory::f16_right_wing(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        // Canopy (bubble canopy)
        commands.spawn((
            Mesh3d(MeshFactory::create_f16_canopy(meshes)),
            MeshMaterial3d(MaterialFactory::create_f16_canopy_material(materials)),
            TransformFactory::f16_canopy(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        // Air intakes
        let intake_mesh = MeshFactory::create_f16_air_intake(meshes);
        let intake_material = MaterialFactory::create_f16_intake_material(materials);
        
        commands.spawn((
            Mesh3d(intake_mesh.clone()),
            MeshMaterial3d(intake_material.clone()),
            TransformFactory::f16_left_air_intake(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        commands.spawn((
            Mesh3d(intake_mesh),
            MeshMaterial3d(intake_material),
            TransformFactory::f16_right_air_intake(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        // Vertical tail
        commands.spawn((
            Mesh3d(MeshFactory::create_f16_vertical_tail(meshes)),
            MeshMaterial3d(wing_material.clone()),
            TransformFactory::f16_vertical_tail(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        // Horizontal stabilizers
        let h_stab_mesh = MeshFactory::create_f16_horizontal_stabilizer(meshes);
        
        commands.spawn((
            Mesh3d(h_stab_mesh.clone()),
            MeshMaterial3d(wing_material.clone()),
            TransformFactory::f16_left_horizontal_stabilizer(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        commands.spawn((
            Mesh3d(h_stab_mesh),
            MeshMaterial3d(wing_material),
            TransformFactory::f16_right_horizontal_stabilizer(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        // Engine nozzle
        commands.spawn((
            Mesh3d(MeshFactory::create_f16_engine_nozzle(meshes)),
            MeshMaterial3d(MaterialFactory::create_f16_engine_material(materials)),
            TransformFactory::f16_engine_nozzle(),
            Cullable { max_distance: lod_distance, is_culled: false },
            ChildOf(parent_entity),
        ));
        
        Ok(())
    }

    /// Get configuration for specific realistic vehicle type
    fn get_realistic_vehicle_configuration(&self, vehicle_type: &RealisticVehicleType) -> VehicleConfiguration {
        match vehicle_type {
            RealisticVehicleType::BasicCar => VehicleConfiguration {
                body_size: self.config.vehicles.basic_car.body_size,
                mass: self.config.vehicles.basic_car.mass,
                linear_damping: self.config.vehicles.basic_car.linear_damping,
                angular_damping: self.config.vehicles.basic_car.angular_damping,
                default_color: self.config.vehicles.basic_car.default_color,
                dynamics: VehicleDynamics {
                    total_mass: self.config.vehicles.basic_car.mass,
                    front_weight_ratio: 0.6,
                    center_of_gravity: Vec3::new(0.0, 0.3, 0.1),
                    drag_coefficient: 0.35,
                    frontal_area: 2.2,
                    ..default()
                },
                engine: EnginePhysics {
                    max_torque: 200.0,
                    max_rpm: 6000.0,
                    gear_ratios: vec![-3.0, 3.5, 2.0, 1.3, 1.0, 0.8],
                    ..default()
                },
                suspension: VehicleSuspension {
                    spring_strength: 25000.0,
                    damping_ratio: 0.6,
                    max_compression: 0.3,
                    rest_length: 0.5,
                    ..default()
                },
                tire_physics: TirePhysics {
                    dry_grip: 1.0,
                    wet_grip: 0.7,
                    lateral_grip: 0.9,
                    rolling_resistance: 0.015,
                    ..default()
                },
            },
            RealisticVehicleType::SuperCar => VehicleConfiguration {
                body_size: self.config.vehicles.super_car.body_size,
                mass: self.config.vehicles.super_car.mass,
                linear_damping: self.config.vehicles.super_car.linear_damping,
                angular_damping: self.config.vehicles.super_car.angular_damping,
                default_color: self.config.vehicles.super_car.default_color,
                dynamics: VehicleDynamics {
                    total_mass: self.config.vehicles.super_car.mass,
                    front_weight_ratio: 0.45,
                    center_of_gravity: Vec3::new(0.0, 0.25, -0.1),
                    drag_coefficient: 0.28,
                    frontal_area: 1.9,
                    downforce_coefficient: 0.3,
                    ..default()
                },
                engine: EnginePhysics {
                    max_torque: 400.0,
                    max_rpm: 8500.0,
                    gear_ratios: vec![-3.2, 4.0, 2.4, 1.6, 1.2, 0.9, 0.7],
                    ..default()
                },
                suspension: VehicleSuspension {
                    spring_strength: 35000.0,
                    damping_ratio: 0.7,
                    max_compression: 0.2,
                    rest_length: 0.4,
                    ..default()
                },
                tire_physics: TirePhysics {
                    dry_grip: 1.4,
                    wet_grip: 0.9,
                    lateral_grip: 1.3,
                    rolling_resistance: 0.012,
                    ..default()
                },
            },
            RealisticVehicleType::Truck => VehicleConfiguration {
                body_size: Vec3::new(2.5, 2.0, 8.0),
                mass: 8000.0,
                linear_damping: 2.0,
                angular_damping: 8.0,
                default_color: Color::srgb(0.6, 0.6, 0.7),
                dynamics: VehicleDynamics {
                    total_mass: 8000.0,
                    front_weight_ratio: 0.4,
                    center_of_gravity: Vec3::new(0.0, 1.0, 0.5),
                    drag_coefficient: 0.6,
                    frontal_area: 4.0,
                    ..default()
                },
                engine: EnginePhysics {
                    max_torque: 800.0,
                    max_rpm: 3500.0,
                    gear_ratios: vec![-4.5, 5.0, 3.8, 2.8, 2.0, 1.5, 1.0, 0.8],
                    differential_ratio: 4.5,
                    ..default()
                },
                suspension: VehicleSuspension {
                    spring_strength: 45000.0,
                    damping_ratio: 0.8,
                    max_compression: 0.4,
                    rest_length: 0.6,
                    ..default()
                },
                tire_physics: TirePhysics {
                    dry_grip: 0.9,
                    wet_grip: 0.6,
                    lateral_grip: 0.7,
                    rolling_resistance: 0.025,
                    ..default()
                },
            },
            _ => {
                // Default to basic car configuration
                self.get_realistic_vehicle_configuration(&RealisticVehicleType::BasicCar)
            }
        }
    }
    
    /// Create individual wheels with physics for realistic vehicles
    fn create_realistic_vehicle_wheels(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vehicle_entity: Entity,
        vehicle_type: &RealisticVehicleType,
    ) -> Result<(), BundleError> {
        let wheel_positions = match vehicle_type {
            RealisticVehicleType::BasicCar | RealisticVehicleType::SuperCar => {
                vec![
                    Vec3::new(-0.8, -0.2, 1.2),  // Front left
                    Vec3::new(0.8, -0.2, 1.2),   // Front right
                    Vec3::new(-0.8, -0.2, -1.2), // Rear left
                    Vec3::new(0.8, -0.2, -1.2),  // Rear right
                ]
            },
            RealisticVehicleType::Truck => {
                vec![
                    Vec3::new(-1.0, -0.5, 2.5),  // Front left
                    Vec3::new(1.0, -0.5, 2.5),   // Front right
                    Vec3::new(-1.0, -0.5, -2.5), // Rear left
                    Vec3::new(1.0, -0.5, -2.5),  // Rear right
                ]
            },
            _ => vec![],
        };
        
        for (index, position) in wheel_positions.iter().enumerate() {
            commands.spawn((
                Transform::from_translation(*position),
                Visibility::default(),
                
                // Wheel mesh
                Mesh3d(meshes.add(Cylinder::new(0.35, 0.2))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.2, 0.2),
                    metallic: 0.1,
                    perceptual_roughness: 0.8,
                    ..default()
                })),
                
                // Wheel physics
                VehicleWheel {
                    index,
                    position: *position,
                    max_steering_angle: if index < 2 { 0.6 } else { 0.0 }, // Front wheels steer
                    is_drive_wheel: match vehicle_type {
                        RealisticVehicleType::SuperCar => index >= 2, // RWD
                        _ => true, // AWD for others
                    },
                    is_brake_wheel: true,
                    radius: if matches!(vehicle_type, RealisticVehicleType::Truck) { 0.5 } else { 0.35 },
                    width: if matches!(vehicle_type, RealisticVehicleType::Truck) { 0.3 } else { 0.2 },
                    ..default()
                },
                
                ChildOf(vehicle_entity),
            ));
        }
        
        Ok(())
    }
    
    /// Create audio sources for realistic vehicle sounds
    fn create_vehicle_audio_sources(
        &self,
        commands: &mut Commands,
        vehicle_entity: Entity,
    ) -> Result<(), BundleError> {
        // Create placeholder audio sources (would need actual audio assets)
        let engine_source = commands.spawn((
            Transform::default(),
            ChildOf(vehicle_entity),
            // AudioSource placeholder - would need actual implementation
        )).id();
        
        let tire_source = commands.spawn((
            Transform::default(),
            ChildOf(vehicle_entity),
            // AudioSource placeholder
        )).id();
        
        let wind_source = commands.spawn((
            Transform::default(),
            ChildOf(vehicle_entity),
            // AudioSource placeholder
        )).id();
        
        let brake_source = commands.spawn((
            Transform::default(),
            ChildOf(vehicle_entity),
            // AudioSource placeholder
        )).id();
        
        // Add audio sources component to vehicle
        commands.entity(vehicle_entity).insert(VehicleAudioSources {
            engine_source,
            tire_source,
            wind_source,
            brake_source,
        });
        
        Ok(())
    }
}

/// NPC CREATION METHODS
impl UnifiedEntityFactory {
    /// Spawn complete NPC entity with all components
    pub fn spawn_npc(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        appearance: NPCAppearance,
    ) -> Result<Entity, BundleError> {
        // Validate position
        let validated_position = self.validate_position(position)?;
        
        // Create bundle using generic system
        let bundle = GenericBundleFactory::npc(
            validated_position,
            appearance.height,
            appearance.build,
            appearance.clone(),
            &self.config,
        )?;
        
        // Create NPC with basic body
        let npc_entity = commands.spawn(bundle).id();
        
        // Add visual components
        self.add_npc_visual_components(commands, npc_entity, meshes, materials, &appearance)?;
        
        Ok(npc_entity)
    }
    
    /// Add NPC visual components (head, body, limbs)
    fn add_npc_visual_components(
        &self,
        commands: &mut Commands,
        parent_entity: Entity,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        appearance: &NPCAppearance,
    ) -> Result<(), BundleError> {
        // Head
        let head_mesh = MeshFactory::create_npc_head(meshes, appearance.build);
        let head_material = MaterialFactory::create_simple_material(materials, appearance.skin_tone);
        
        commands.spawn((
            Mesh3d(head_mesh),
            MeshMaterial3d(head_material),
            Transform::from_xyz(0.0, appearance.height * 0.85, 0.0),
            Cullable { max_distance: self.config.world.lod_distances[0], is_culled: false },
            ChildOf(parent_entity),
        ));
        
        // Body
        let body_mesh = meshes.add(Cuboid::new(
            0.4 * appearance.build,
            0.6 * appearance.height,
            0.2 * appearance.build,
        ));
        let body_material = MaterialFactory::create_simple_material(materials, appearance.shirt_color);
        
        commands.spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_material),
            Transform::from_xyz(0.0, appearance.height * 0.5, 0.0),
            Cullable { max_distance: self.config.world.lod_distances[0], is_culled: false },
            ChildOf(parent_entity),
        ));
        
        Ok(())
    }
}

/// BUILDING & ENVIRONMENT CREATION METHODS
impl UnifiedEntityFactory {
    /// Spawn complete building entity
    pub fn spawn_building(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        size: Vec3,
        building_type: BuildingType,
        color: Color,
    ) -> Result<Entity, BundleError> {
        // Validate position
        let validated_position = self.validate_position(position)?;
        
        // Create bundle using generic system
        let bundle = GenericBundleFactory::building(validated_position, size, building_type, color, &self.config)?;
        
        // Create appropriate mesh based on building type
        let mesh_handle = match building_type {
            BuildingType::Residential => meshes.add(Cuboid::new(size.x, size.y, size.z)),
            BuildingType::Commercial => meshes.add(Cuboid::new(size.x * 0.8, size.y * 0.8, size.z * 0.8)),
            BuildingType::Industrial => meshes.add(Cuboid::new(size.x, size.y * 1.5, size.z)),
            BuildingType::Skyscraper => meshes.add(Cuboid::new(size.x * 0.5, size.y * 2.0, size.z * 0.5)),
            BuildingType::Generic => meshes.add(Cuboid::new(size.x, size.y, size.z)),
        };
        
        let material_handle = MaterialFactory::create_simple_material(materials, color);
        
        let building_entity = commands.spawn((
            bundle,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
        )).id();
        
        Ok(building_entity)
    }
    
    /// Spawn terrain/ground entities
    pub fn spawn_terrain(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        size: Vec2,
    ) -> Result<Entity, BundleError> {
        // Validate position
        let validated_position = self.validate_position(position)?;
        
        // Create physics bundle for terrain
        let bundle = GenericBundleFactory::physics_object(
            validated_position,
            ColliderShape::Box(Vec3::new(size.x, 0.2, size.y)),
            1000.0, // Large mass for terrain
            self.config.physics.static_group,
            false, // Static terrain
            &self.config,
        )?;
        
        let mesh_handle = meshes.add(Plane3d::default().mesh().size(size.x, size.y));
        let material_handle = MaterialFactory::create_simple_material(
            materials,
            Color::srgb(0.85, 0.75, 0.6), // Ground color
        );
        
        let terrain_entity = commands.spawn((
            bundle,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            DynamicTerrain,
        )).id();
        
        Ok(terrain_entity)
    }
    
    /// Spawn tree entities
    pub fn spawn_tree(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        height: f32,
        trunk_radius: f32,
    ) -> Result<Entity, BundleError> {
        // Validate position
        let validated_position = self.validate_position(position + Vec3::new(0.0, height / 2.0, 0.0))?;
        
        // Create physics bundle for tree
        let bundle = GenericBundleFactory::physics_object(
            validated_position,
            ColliderShape::Cylinder { radius: trunk_radius, height },
            100.0 * height, // Mass proportional to height
            self.config.physics.static_group,
            false, // Static tree
            &self.config,
        )?;
        
        let trunk_mesh = meshes.add(Cylinder::new(trunk_radius, height));
        let trunk_material = MaterialFactory::create_simple_material(
            materials,
            Color::srgb(0.4, 0.2, 0.1), // Brown trunk
        );
        
        let tree_entity = commands.spawn((
            bundle,
            Mesh3d(trunk_mesh),
            MeshMaterial3d(trunk_material),
        )).id();
        
        // Add leaves/canopy
        let leaves_mesh = meshes.add(Sphere::new(trunk_radius * 3.0));
        let leaves_material = MaterialFactory::create_simple_material(
            materials,
            Color::srgb(0.2, 0.6, 0.2), // Green leaves
        );
        
        commands.spawn((
            Mesh3d(leaves_mesh),
            MeshMaterial3d(leaves_material),
            Transform::from_xyz(0.0, height * 0.7, 0.0),
            Cullable { max_distance: self.config.world.lod_distances[1], is_culled: false },
            ChildOf(tree_entity),
        ));
        
        Ok(tree_entity)
    }
    
    /// Spawn water body (lake, river, etc.)
    pub fn spawn_water_body(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        size: Vec2,
        depth: f32,
    ) -> Result<Entity, BundleError> {
        // Validate position
        let validated_position = self.validate_position(position - Vec3::new(0.0, depth / 2.0, 0.0))?;
        
        // Create physics bundle for water bottom
        let bottom_bundle = GenericBundleFactory::physics_object(
            validated_position,
            ColliderShape::Cylinder { radius: size.x / 2.0, height: depth },
            10000.0, // Heavy water body
            self.config.physics.static_group,
            false, // Static water body
            &self.config,
        )?;
        
        // Water bottom
        let bottom_mesh = meshes.add(Cylinder::new(size.x / 2.0, depth));
        let bottom_material = MaterialFactory::create_water_bottom_material(
            materials,
            Color::srgb(0.3, 0.25, 0.2),
        );
        
        let water_entity = commands.spawn((
            bottom_bundle,
            Mesh3d(bottom_mesh),
            MeshMaterial3d(bottom_material),
            Lake {
                size: size.x,
                depth,
                wave_height: 0.5,
                wave_speed: 1.0,
                position,
            },
            WaterBody,
        )).id();
        
        // Water surface
        let surface_mesh = meshes.add(Plane3d::default().mesh().size(size.x * 0.9, size.y * 0.9));
        let surface_material = MaterialFactory::create_water_surface_material(
            materials,
            Color::srgba(0.1, 0.4, 0.8, 0.7),
        );
        
        commands.spawn((
            Mesh3d(surface_mesh),
            MeshMaterial3d(surface_material),
            Transform::from_translation(position),
            WaterWave {
                amplitude: 0.5,
                frequency: 1.0,
                phase: 0.0,
            },
            Cullable { max_distance: self.config.world.lod_distances[2], is_culled: false },
            ChildOf(water_entity),
        ));
        
        Ok(water_entity)
    }
    
    /// Spawn particle effect
    pub fn spawn_particle_effect(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        effect_type: ParticleEffectType,
    ) -> Result<Entity, BundleError> {
        // Validate position
        let validated_position = self.validate_position(position)?;
        
        let (mesh, material_color, _lifetime) = match effect_type {
            ParticleEffectType::Exhaust => (
                meshes.add(Sphere::new(0.15)),
                Color::srgb(1.0, 0.3, 0.0),
                2.0,
            ),
            ParticleEffectType::Explosion => (
                meshes.add(Sphere::new(0.5)),
                Color::srgb(1.0, 0.8, 0.0),
                1.0,
            ),
            ParticleEffectType::Spark => (
                meshes.add(Sphere::new(0.05)),
                Color::srgb(1.0, 1.0, 0.8),
                0.5,
            ),
            _ => (
                meshes.add(Sphere::new(0.1)),
                Color::srgb(0.5, 0.5, 0.5),
                1.0,
            ),
        };
        
        let material_handle = materials.add(StandardMaterial {
            base_color: material_color,
            emissive: LinearRgba::rgb(1.0, 0.5, 0.0),
            ..Default::default()
        });
        
        // Create minimal physics for particle
        let bundle = GenericBundleFactory::physics_object(
            validated_position,
            ColliderShape::Sphere(0.1),
            0.1, // Very light particle
            self.config.physics.static_group,
            true, // Dynamic particle
            &self.config,
        )?;
        
        let effect_entity = commands.spawn((
            bundle,
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            ExhaustFlame,
            Cullable { max_distance: self.config.world.lod_distances[0], is_culled: false },
        )).id();
        
        Ok(effect_entity)
    }
}

/// VISUAL HELPER METHODS
impl UnifiedEntityFactory {
    /// Create a visual child entity with standardized components
    pub fn spawn_visual_child(
        &self,
        commands: &mut Commands,
        parent: Entity,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
    ) -> Entity {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            ChildOf(parent),
            VisibleChildBundle::default(),
        )).id()
    }
    
    /// Create a visual child entity with a name (for debugging)
    pub fn spawn_named_visual_child(
        &self,
        commands: &mut Commands,
        parent: Entity,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
        name: &str,
    ) -> Entity {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            ChildOf(parent),
            VisibleChildBundle::default(),
            Name::new(name.to_string()),
        )).id()
    }
    
    /// Create a road entity with standardized components
    pub fn spawn_road_entity(
        &self,
        commands: &mut Commands,
        position: Vec3,
        size: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Result<Entity, BundleError> {
        // Validate position
        let validated_position = self.validate_position(position)?;
        
        let road_entity = commands.spawn((
            Transform::from_translation(validated_position),
            RigidBody::Fixed,
            Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
            CollisionGroups::new(
                self.config.physics.static_group,
                self.config.physics.vehicle_group | self.config.physics.character_group
            ),
            RoadEntity { road_id: 0 },
            Name::new("Road Segment"),
        )).id();
        
        // Add visual representation
        self.spawn_visual_child(
            commands,
            road_entity,
            meshes.add(Cuboid::new(size.x, size.y, size.z)),
            MaterialFactory::create_water_bottom_material(materials, Color::srgb(0.3, 0.3, 0.3)),
            Transform::default(),
        );
        
        Ok(road_entity)
    }
}

/// BATCH CREATION METHODS
impl UnifiedEntityFactory {
    /// Spawn multiple vehicles in batch
    pub fn spawn_vehicle_batch(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vehicle_specs: Vec<(VehicleType, Vec3, Color)>,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();
        
        for (vehicle_type, position, color) in vehicle_specs {
            let entity = self.spawn_vehicle(commands, meshes, materials, vehicle_type, position, color)?;
            entities.push(entity);
        }
        
        Ok(entities)
    }
    
    /// Spawn multiple NPCs in batch
    pub fn spawn_npc_batch(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        npc_specs: Vec<(Vec3, NPCAppearance)>,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();
        
        for (position, appearance) in npc_specs {
            let entity = self.spawn_npc(commands, meshes, materials, position, appearance)?;
            entities.push(entity);
        }
        
        Ok(entities)
    }
    
    /// Spawn multiple buildings in batch
    pub fn spawn_building_batch(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        building_specs: Vec<(Vec3, Vec3, BuildingType, Color)>,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();
        
        for (position, size, building_type, color) in building_specs {
            let entity = self.spawn_building(commands, meshes, materials, position, size, building_type, color)?;
            entities.push(entity);
        }
        
        Ok(entities)
    }
}

/// Configuration structure for realistic vehicle creation
#[derive(Clone)]
struct VehicleConfiguration {
    body_size: Vec3,
    mass: f32,
    linear_damping: f32,
    angular_damping: f32,
    default_color: Color,
    dynamics: VehicleDynamics,
    engine: EnginePhysics,
    suspension: VehicleSuspension,
    tire_physics: TirePhysics,
}

impl Default for UnifiedEntityFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// System to initialize UnifiedEntityFactory as a resource
pub fn setup_unified_entity_factory(mut commands: Commands) {
    commands.insert_resource(UnifiedEntityFactory::default());
}

/// System to convert legacy vehicles to realistic vehicles
pub fn convert_legacy_vehicles_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    factory: Res<UnifiedEntityFactory>,
    legacy_vehicles: Query<(Entity, &Transform, Option<&SuperCar>), (With<Car>, Without<RealisticVehicle>)>,
) {
    for (entity, transform, supercar) in legacy_vehicles.iter() {
        // Determine vehicle type
        let vehicle_type = if supercar.is_some() {
            RealisticVehicleType::SuperCar
        } else {
            RealisticVehicleType::BasicCar
        };
        
        // Create new realistic vehicle at same position
        if let Ok(_new_vehicle) = factory.spawn_realistic_vehicle(
            &mut commands,
            &mut meshes,
            &mut materials,
            vehicle_type,
            transform.translation,
            transform.rotation,
        ) {
            // Remove old vehicle
            commands.entity(entity).despawn();
            
            info!("Converted legacy vehicle to realistic vehicle at {:?}", transform.translation);
        }
    }
}

/// Performance monitoring for unified entity factory
pub fn unified_entity_factory_performance_system(
    time: Res<Time>,
    realistic_vehicles: Query<&RealisticVehicle>,
) {
    let current_time = time.elapsed_secs();
    static mut LAST_REPORT: f32 = 0.0;
    
    unsafe {
        if current_time - LAST_REPORT > 20.0 {
            LAST_REPORT = current_time;
            let total_realistic = realistic_vehicles.iter().count();
            let physics_enabled = realistic_vehicles.iter().filter(|v| v.physics_enabled).count();
            info!("UNIFIED FACTORY: {}/{} realistic vehicles with physics enabled", physics_enabled, total_realistic);
        }
    }
}
