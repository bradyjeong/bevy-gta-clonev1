use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use crate::components::*;
use crate::bundles::{
    VisibleChildBundle, VehicleBundle, NPCBundle, BuildingBundle, DynamicContentBundle, 
    DynamicPhysicsBundle, VegetationBundle, StaticPhysicsBundle
};
use crate::factories::{MaterialFactory, MeshFactory, TransformFactory};
use crate::factories::generic_bundle::{GenericBundleFactory, BundleError, ColliderShape, ParticleEffectType};
use crate::systems::audio::realistic_vehicle_audio::{VehicleAudioState, VehicleAudioSources};
use crate::systems::distance_cache::MovementTracker;
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::road_generation::is_on_road_spline;
use crate::systems::world::unified_distance_culling::UnifiedCullable;

use crate::GameConfig;

/// Unified Entity Factory - Single point of all entity creation
/// Consolidates EntityFactory, UnifiedEntityFactory, RealisticVehicleFactory functionality
/// Phase 2.1: Enhanced with centralized spawn logic and entity limit management
#[derive(Resource)]
pub struct UnifiedEntityFactory {
    pub config: GameConfig,
    /// Entity limit management with configurable thresholds
    pub entity_limits: EntityLimitManager,
    /// Position validation cache for performance
    pub position_cache: HashMap<(i32, i32), f32>, // (grid_x, grid_z) -> ground_height
}

/// Entity limit manager with configurable thresholds and automatic cleanup
#[derive(Debug, Clone)]
pub struct EntityLimitManager {
    pub max_buildings: usize,
    pub max_vehicles: usize,
    pub max_npcs: usize,
    pub max_trees: usize,
    pub max_particles: usize,
    
    // Entity tracking with timestamps for FIFO cleanup
    pub building_entities: Vec<(Entity, f32)>,
    pub vehicle_entities: Vec<(Entity, f32)>,
    pub npc_entities: Vec<(Entity, f32)>,
    pub tree_entities: Vec<(Entity, f32)>,
    pub particle_entities: Vec<(Entity, f32)>,
}

impl Default for EntityLimitManager {
    fn default() -> Self {
        Self {
            // Configurable limits based on AGENT.md
            max_buildings: (1000.0 * 0.08) as usize, // 8% of 1000 = 80 buildings
            max_vehicles: (500.0 * 0.04) as usize,   // 4% of 500 = 20 vehicles  
            max_npcs: (200.0 * 0.01) as usize,       // 1% of 200 = 2 NPCs
            max_trees: (2000.0 * 0.05) as usize,     // 5% of 2000 = 100 trees
            max_particles: 50,
            
            building_entities: Vec::new(),
            vehicle_entities: Vec::new(),
            npc_entities: Vec::new(),
            tree_entities: Vec::new(),
            particle_entities: Vec::new(),
        }
    }
}

impl EntityLimitManager {
    /// Check if entity limit has been reached and despawn oldest if needed
    pub fn enforce_limit(&mut self, commands: &mut Commands, entity_type: ContentType, entity: Entity, timestamp: f32) {
        match entity_type {
            ContentType::Building => {
                if self.building_entities.len() >= self.max_buildings {
                    if let Some((oldest_entity, _)) = self.building_entities.first().copied() {
                        commands.entity(oldest_entity).despawn();
                        self.building_entities.remove(0);
                    }
                }
                self.building_entities.push((entity, timestamp));
            }
            ContentType::Vehicle => {
                if self.vehicle_entities.len() >= self.max_vehicles {
                    if let Some((oldest_entity, _)) = self.vehicle_entities.first().copied() {
                        commands.entity(oldest_entity).despawn();
                        self.vehicle_entities.remove(0);
                    }
                }
                self.vehicle_entities.push((entity, timestamp));
            }
            ContentType::NPC => {
                if self.npc_entities.len() >= self.max_npcs {
                    if let Some((oldest_entity, _)) = self.npc_entities.first().copied() {
                        commands.entity(oldest_entity).despawn();
                        self.npc_entities.remove(0);
                    }
                }
                self.npc_entities.push((entity, timestamp));
            }
            ContentType::Tree => {
                if self.tree_entities.len() >= self.max_trees {
                    if let Some((oldest_entity, _)) = self.tree_entities.first().copied() {
                        commands.entity(oldest_entity).despawn();
                        self.tree_entities.remove(0);
                    }
                }
                self.tree_entities.push((entity, timestamp));
            }
            _ => {} // Other types don't have limits
        }
    }
    
    /// Get current entity counts for each type
    pub fn get_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.building_entities.len(),
            self.vehicle_entities.len(), 
            self.npc_entities.len(),
            self.tree_entities.len()
        )
    }
}

impl UnifiedEntityFactory {
    /// Create new factory with default configuration
    pub fn new() -> Self {
        Self {
            config: GameConfig::default(),
            entity_limits: EntityLimitManager::default(),
            position_cache: HashMap::new(),
        }
    }
    
    /// Create factory with custom configuration
    pub fn with_config(config: GameConfig) -> Self {
        Self {
            config,
            entity_limits: EntityLimitManager::default(),
            position_cache: HashMap::new(),
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
    
    /// Get ground height at position with caching for performance
    pub fn get_ground_height(&mut self, position: Vec2) -> f32 {
        let grid_x = (position.x / 10.0) as i32; // 10m grid resolution
        let grid_z = (position.y / 10.0) as i32;
        
        if let Some(&cached_height) = self.position_cache.get(&(grid_x, grid_z)) {
            return cached_height;
        }
        
        // Simple ground detection - would be enhanced with actual terrain data
        let ground_height = -0.05; // Match terrain level
        
        // Cache for future use
        self.position_cache.insert((grid_x, grid_z), ground_height);
        ground_height
    }
    
    /// Check if position is valid for spawning (not on roads, water, etc.)
    pub fn is_spawn_position_valid(
        &self, 
        position: Vec3, 
        content_type: ContentType,
        road_network: Option<&RoadNetwork>
    ) -> bool {
        // Check if on road (invalid for buildings and trees)
        if let Some(roads) = road_network {
            let road_tolerance: f32 = match content_type {
                ContentType::Building => 25.0,
                ContentType::Tree => 15.0,
                ContentType::Vehicle => -8.0, // Negative means vehicles NEED roads
                ContentType::NPC => 0.0, // NPCs can be anywhere
                _ => 10.0,
            };
            
            let on_road = is_on_road_spline(position, roads, road_tolerance.abs());
            
            match content_type {
                ContentType::Vehicle => {
                    if !on_road { return false; } // Vehicles need roads
                }
                ContentType::Building | ContentType::Tree => {
                    if on_road { return false; } // Buildings/trees avoid roads
                }
                _ => {} // NPCs and others don't care about roads
            }
        }
        
        // Check if in water area
        if self.is_in_water_area(position) && !matches!(content_type, ContentType::Vehicle) {
            return false;
        }
        
        true
    }
    
    /// Check if position is in water area
    fn is_in_water_area(&self, position: Vec3) -> bool {
        // Lake position and size (must match water.rs setup)
        let lake_center = Vec3::new(300.0, -2.0, 300.0);
        let lake_size = 200.0;
        let buffer = 20.0; // Extra buffer around lake
        
        let distance = Vec2::new(
            position.x - lake_center.x,
            position.z - lake_center.z,
        ).length();
        
        distance < (lake_size / 2.0 + buffer)
    }
    
    /// Check for content collision with existing entities
    pub fn has_content_collision(
        &self,
        position: Vec3, 
        content_type: ContentType,
        existing_content: &[(Vec3, ContentType, f32)]
    ) -> bool {
        let min_distance = match content_type {
            ContentType::Building => 35.0,
            ContentType::Vehicle => 25.0,
            ContentType::Tree => 10.0,
            ContentType::NPC => 5.0,
            _ => 15.0,
        };
        
        existing_content.iter().any(|(existing_pos, _, radius)| {
            let required_distance = min_distance + radius + 2.0; // 2.0 buffer
            position.distance(*existing_pos) < required_distance
        })
    }
}

/// CONSOLIDATED SPAWN METHODS - Phase 2.1 Enhanced
/// These methods consolidate all duplicate spawn logic from multiple systems
impl UnifiedEntityFactory {
    /// Master spawn method - automatically determines best spawn logic and handles limits
    pub fn spawn_entity_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        content_type: ContentType,
        position: Vec3,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Option<Entity>, BundleError> {
        // Validate position first
        let validated_position = self.validate_position(position)?;
        
        // Check if position is valid for this content type
        if !self.is_spawn_position_valid(validated_position, content_type, road_network) {
            return Ok(None); // Position not valid, but no error
        }
        
        // Check for collisions with existing content
        if self.has_content_collision(validated_position, content_type, existing_content) {
            return Ok(None); // Collision detected, but no error
        }
        
        // Spawn the appropriate entity type
        let entity = match content_type {
            ContentType::Building => {
                self.spawn_building_consolidated(commands, meshes, materials, validated_position, current_time)?
            }
            ContentType::Vehicle => {
                self.spawn_vehicle_consolidated(commands, meshes, materials, validated_position, current_time)?
            }
            ContentType::NPC => {
                self.spawn_npc_consolidated(commands, meshes, materials, validated_position, current_time)?
            }
            ContentType::Tree => {
                self.spawn_tree_consolidated(commands, meshes, materials, validated_position, current_time)?
            }
            _ => return Ok(None), // Unsupported content type
        };
        
        // Enforce entity limits and track the new entity
        self.entity_limits.enforce_limit(commands, content_type, entity, current_time);
        
        Ok(Some(entity))
    }
    
    /// Consolidated building spawn with all features from dynamic_content.rs and layered_generation.rs
    pub fn spawn_building_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        let mut rng = rand::thread_rng();
        
        // Determine building size and type
        let height = rng.gen_range(8.0..30.0);
        let width = rng.gen_range(8.0..15.0);
        
        // Building positioned with base on terrain surface
        let ground_level = self.get_ground_height(Vec2::new(position.x, position.z));
        let building_mesh_y = ground_level + height / 2.0;
        let final_position = Vec3::new(position.x, building_mesh_y, position.z);
        
        // Create material with random color
        let building_material = materials.add(StandardMaterial {
            base_color: Color::srgb(
                rng.gen_range(0.5..0.9),
                rng.gen_range(0.5..0.9),
                rng.gen_range(0.5..0.9),
            ),
            ..default()
        });
        
        // Create building entity using enhanced bundle system
        let building_entity = commands.spawn((
            // Use dynamic content bundle for compatibility
            DynamicContentBundle {
                dynamic_content: DynamicContent { content_type: ContentType::Building },
                transform: Transform::from_translation(final_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                cullable: UnifiedCullable::building(),
            },
            // Building-specific components
            Building {
                building_type: BuildingType::Generic,
                height,
                scale: Vec3::new(width, height, width),
            },
            // Physics components  
            RigidBody::Fixed,
            Collider::cuboid(width / 2.0, height / 2.0, width / 2.0),
            CollisionGroups::new(self.config.physics.static_group, Group::ALL),
            // Visual components
            Mesh3d(meshes.add(Cuboid::new(width, height, width))),
            MeshMaterial3d(building_material),
            // Debug name
            Name::new(format!("Building_{:.0}_{:.0}_{}", position.x, position.z, current_time)),
        )).id();
        
        Ok(building_entity)
    }
    
    /// Consolidated vehicle spawn with enhanced bundles and physics
    pub fn spawn_vehicle_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        let mut rng = rand::thread_rng();
        
        // Random vehicle color
        let car_colors = [
            Color::srgb(1.0, 0.0, 0.0), Color::srgb(0.0, 0.0, 1.0), Color::srgb(0.0, 1.0, 0.0),
            Color::srgb(1.0, 1.0, 0.0), Color::srgb(1.0, 0.0, 1.0), Color::srgb(0.0, 1.0, 1.0),
            Color::srgb(0.5, 0.5, 0.5), Color::srgb(1.0, 1.0, 1.0), Color::srgb(0.0, 0.0, 0.0),
        ];
        let color = car_colors[rng.gen_range(0..car_colors.len())];
        
        // Position vehicle on ground surface
        let ground_level = self.get_ground_height(Vec2::new(position.x, position.z));
        let final_position = Vec3::new(position.x, ground_level + 0.5, position.z);
        
        // Create vehicle entity using consolidated bundle approach
        let vehicle_entity = commands.spawn((
            // Dynamic content bundle for compatibility
            DynamicPhysicsBundle {
                dynamic_content: DynamicContent { content_type: ContentType::Vehicle },
                transform: Transform::from_translation(final_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                rigid_body: RigidBody::Dynamic,
                collider: Collider::cuboid(1.0, 0.5, 2.0),
                collision_groups: CollisionGroups::new(
                    self.config.physics.vehicle_group,
                    self.config.physics.static_group | self.config.physics.vehicle_group | self.config.physics.character_group
                ),
                velocity: Velocity::default(),
                cullable: UnifiedCullable::vehicle(),
            },
            // Vehicle-specific components
            Car,
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            Damping { linear_damping: 1.0, angular_damping: 5.0 },
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
    
    /// Consolidated NPC spawn using enhanced state system
    pub fn spawn_npc_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        let mut rng = rand::thread_rng();
        
        // Random NPC appearance
        let npc_colors = [
            Color::srgb(0.8, 0.6, 0.4), Color::srgb(0.6, 0.4, 0.3),
            Color::srgb(0.9, 0.7, 0.5), Color::srgb(0.7, 0.5, 0.4),
        ];
        let color = npc_colors[rng.gen_range(0..npc_colors.len())];
        
        // Position NPC on ground
        let ground_level = self.get_ground_height(Vec2::new(position.x, position.z));
        let final_position = Vec3::new(position.x, ground_level + 1.0, position.z);
        
        // Random target position for movement
        let target_x = rng.gen_range(-900.0..900.0);
        let target_z = rng.gen_range(-900.0..900.0);
        let target_position = Vec3::new(target_x, ground_level + 1.0, target_z);
        
        // Create NPC entity using consolidated approach
        let npc_entity = commands.spawn((
            // Dynamic physics bundle
            DynamicPhysicsBundle {
                dynamic_content: DynamicContent { content_type: ContentType::NPC },
                transform: Transform::from_translation(final_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                rigid_body: RigidBody::Dynamic,
                collider: Collider::capsule(Vec3::new(0.0, -0.9, 0.0), Vec3::new(0.0, 0.9, 0.0), 0.3),
                collision_groups: CollisionGroups::new(
                    self.config.physics.character_group,
                    Group::ALL
                ),
                velocity: Velocity::default(),
                cullable: UnifiedCullable::npc(),
            },
            // NPC-specific components
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            MovementTracker::new(final_position, 5.0),
            NPC {
                target_position,
                speed: rng.gen_range(2.0..5.0),
                last_update: current_time,
                update_interval: rng.gen_range(0.05..0.2),
            },
            // Visual mesh
            Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
            MeshMaterial3d(materials.add(color)),
            Name::new(format!("NPC_{:.0}_{:.0}_{}", position.x, position.z, current_time)),
        )).id();
        
        Ok(npc_entity)
    }
    
    /// Consolidated tree spawn with LOD support
    pub fn spawn_tree_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        // Position tree on ground
        let ground_level = self.get_ground_height(Vec2::new(position.x, position.z));
        let final_position = Vec3::new(position.x, ground_level, position.z);
        
        // Create tree entity using vegetation bundle
        let tree_entity = commands.spawn((
            VegetationBundle {
                dynamic_content: DynamicContent { content_type: ContentType::Tree },
                transform: Transform::from_translation(final_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                cullable: UnifiedCullable::vegetation(),
            },
            Name::new(format!("Tree_{:.0}_{:.0}_{}", position.x, position.z, current_time)),
        )).id();
        
        // Add palm tree trunk as child
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.4, 0.25, 0.15))), // Brown trunk
            Transform::from_xyz(0.0, 4.0, 0.0),
            ChildOf(tree_entity),
            VisibleChildBundle::default(),
        ));
        
        // Add palm fronds as children
        for i in 0..4 {
            let angle = (i as f32) * std::f32::consts::PI / 2.0;
            
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(2.5, 0.1, 0.8))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 0.25))), // Green fronds
                Transform::from_xyz(
                    angle.cos() * 1.2, 
                    7.5, 
                    angle.sin() * 1.2
                ).with_rotation(
                    Quat::from_rotation_y(angle) * 
                    Quat::from_rotation_z(-0.2) // Slight droop
                ),
                ChildOf(tree_entity),
                VisibleChildBundle::default(),
            ));
        }
        
        // Add physics collider as child
        commands.spawn((
            RigidBody::Fixed,
            Collider::cylinder(4.0, 0.3),
            CollisionGroups::new(self.config.physics.static_group, Group::ALL),
            Transform::from_xyz(0.0, 4.0, 0.0),
            ChildOf(tree_entity),
        ));
        
        Ok(tree_entity)
    }
    
    /// Batch spawn multiple entities of the same type efficiently
    pub fn spawn_batch_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        content_type: ContentType,
        positions: Vec<Vec3>,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut spawned_entities = Vec::new();
        
        for position in positions {
            if let Some(entity) = self.spawn_entity_consolidated(
                commands,
                meshes,
                materials,
                content_type,
                position,
                road_network,
                existing_content,
                current_time,
            )? {
                spawned_entities.push(entity);
            }
        }
        
        Ok(spawned_entities)
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

/// System to initialize UnifiedEntityFactory as a resource (basic version)
pub fn setup_unified_entity_factory_basic(mut commands: Commands) {
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
