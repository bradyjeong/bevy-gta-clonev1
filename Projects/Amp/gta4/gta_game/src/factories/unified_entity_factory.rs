use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::config::*;
use crate::components::*;
use crate::bundles::*;
use crate::factories::generic_bundle::*;
use crate::factories::*;

/// CRITICAL: Unified Entity Factory - Single point of entity creation
/// Replaces 4-factory coordination with unified, validated approach
/// Eliminates 400+ duplicate entity creation patterns

#[derive(Resource)]
pub struct UnifiedEntityFactory {
    pub config: GameConfig,
}

impl UnifiedEntityFactory {
    /// Create new factory with validated configuration
    pub fn new(mut config: GameConfig) -> Self {
        config.validate_and_clamp();
        Self { config }
    }
    
    /// Update configuration with validation
    pub fn update_config(&mut self, mut new_config: GameConfig) {
        new_config.validate_and_clamp();
        self.config = new_config;
    }
    
    /// VEHICLE CREATION METHODS - Unified vehicle spawning
    
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
        // Create bundle using generic system
        let bundle = GenericBundleFactory::vehicle(vehicle_type, position, color, &self.config)?;
        
        // Get vehicle configuration
        let vehicle_config = match vehicle_type {
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
        
        // Add vehicle-specific components
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
            )).set_parent(parent_entity);
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
        )).set_parent(parent_entity);
        
        // Tail rotor
        commands.spawn((
            TailRotor,
            Mesh3d(rotor_mesh),
            MeshMaterial3d(rotor_material),
            TransformFactory::tail_rotor(),
            Cullable { max_distance: self.config.world.lod_distances[1], is_culled: false },
        )).set_parent(parent_entity);
        
        Ok(())
    }
    
    /// Add F16-specific components
    fn add_f16_components(
        &self,
        commands: &mut Commands,
        parent_entity: Entity,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Result<(), BundleError> {
        let wing_mesh = MeshFactory::create_f16_wing(meshes);
        let wing_material = MaterialFactory::create_aircraft_material(materials, Color::srgb(0.4, 0.4, 0.5));
        
        // Add wings
        commands.spawn((
            Mesh3d(wing_mesh.clone()),
            MeshMaterial3d(wing_material.clone()),
            TransformFactory::f16_left_wing(),
            Cullable { max_distance: self.config.world.lod_distances[2], is_culled: false },
        )).set_parent(parent_entity);
        
        commands.spawn((
            Mesh3d(wing_mesh),
            MeshMaterial3d(wing_material),
            TransformFactory::f16_right_wing(),
            Cullable { max_distance: self.config.world.lod_distances[2], is_culled: false },
        )).set_parent(parent_entity);
        
        Ok(())
    }
    
    /// NPC CREATION METHODS - Unified NPC spawning
    
    /// Spawn complete NPC entity with all components
    pub fn spawn_npc(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        appearance: NPCAppearance,
    ) -> Result<Entity, BundleError> {
        // Create bundle using generic system
        let bundle = GenericBundleFactory::npc(
            position,
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
        )).set_parent(parent_entity);
        
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
        )).set_parent(parent_entity);
        
        Ok(())
    }
    
    /// BUILDING CREATION METHODS - Unified building spawning
    
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
        // Create bundle using generic system
        let bundle = GenericBundleFactory::building(position, size, building_type, color, &self.config)?;
        
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
    
    /// ENVIRONMENT CREATION METHODS - Unified environment spawning
    
    /// Spawn terrain/ground entities
    pub fn spawn_terrain(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        size: Vec2,
    ) -> Result<Entity, BundleError> {
        // Create physics bundle for terrain
        let bundle = GenericBundleFactory::physics_object(
            position,
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
        // Create physics bundle for tree
        let bundle = GenericBundleFactory::physics_object(
            position + Vec3::new(0.0, height / 2.0, 0.0),
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
        )).set_parent(tree_entity);
        
        Ok(tree_entity)
    }
    
    /// WATER SYSTEM CREATION - Unified water spawning
    
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
        // Create physics bundle for water bottom
        let bottom_bundle = GenericBundleFactory::physics_object(
            position - Vec3::new(0.0, depth / 2.0, 0.0),
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
        )).set_parent(water_entity);
        
        Ok(water_entity)
    }
    
    /// EFFECT CREATION METHODS - Unified effect spawning
    
    /// Spawn particle effect
    pub fn spawn_particle_effect(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        effect_type: ParticleEffectType,
    ) -> Result<Entity, BundleError> {
        let (mesh, material_color, lifetime) = match effect_type {
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
        };
        
        let material_handle = materials.add(StandardMaterial {
            base_color: material_color,
            emissive: LinearRgba::rgb(1.0, 0.5, 0.0),
            ..Default::default()
        });
        
        // Create minimal physics for particle
        let bundle = GenericBundleFactory::physics_object(
            position,
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
    
    /// BATCH CREATION METHODS - Efficient bulk spawning
    
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
    
    /// UTILITY METHODS - Helper functions
    
    /// Get current configuration
    pub fn get_config(&self) -> &GameConfig {
        &self.config
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

/// Particle effect types for unified effect spawning
#[derive(Debug, Clone, Copy)]
pub enum ParticleEffectType {
    Exhaust,
    Explosion,
    Spark,
}

impl Default for UnifiedEntityFactory {
    fn default() -> Self {
        Self::new(GameConfig::default())
    }
}

/// System to initialize UnifiedEntityFactory as a resource
pub fn setup_unified_entity_factory(mut commands: Commands) {
    commands.insert_resource(UnifiedEntityFactory::default());
}
