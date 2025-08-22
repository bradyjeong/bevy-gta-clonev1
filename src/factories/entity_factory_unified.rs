use crate::bundles::{
    DynamicContentBundle, DynamicPhysicsBundle, VegetationBundle, VisibleChildBundle,
};
use crate::components::*;
use crate::factories::generic_bundle::{
    BundleError, ColliderShape, GenericBundleFactory, ParticleEffectType,
};
use crate::factories::{MaterialFactory, MeshFactory};
use bevy::{prelude::*, render::view::visibility::VisibilityRange};
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::systems::{MovementTracker, RoadNetwork, is_on_road_spline};

use crate::GameConfig;

/// Unified Entity Factory - Single point of all entity creation
/// Consolidates EntityFactory, UnifiedEntityFactory functionality
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
    pub fn enforce_limit(
        &mut self,
        commands: &mut Commands,
        entity_type: ContentType,
        entity: Entity,
        timestamp: f32,
    ) {
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
            self.tree_entities.len(),
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

    /// Entities are now spawned directly in world space

    /// Validate position is safe (no more hard bounds, just safety check)
    pub fn validate_position(&self, position: Vec3) -> Result<Vec3, BundleError> {
        if !position.is_finite() {
            return Err(BundleError::InvalidEntityType {
                entity_type: "Position contains NaN/Infinity".to_string(),
            });
        }

        // No hard bounds anymore - floating origin handles large distances smoothly
        Ok(position)
    }

    /// Get ground height at position with caching for performance
    pub fn get_ground_height(&mut self, position: Vec2) -> f32 {
        let grid_x = (position.x / 10.0) as i32; // 10m grid resolution
        let grid_z = (position.y / 10.0) as i32;

        if let Some(&cached_height) = self.position_cache.get(&(grid_x, grid_z)) {
            return cached_height;
        }

        // Simple ground detection - would be enhanced with actual terrain data
        let ground_height = -0.15; // Match terrain level at y = -0.15

        // Cache for future use
        self.position_cache.insert((grid_x, grid_z), ground_height);
        ground_height
    }

    /// Check if position is valid for spawning (not on roads, water, etc.)
    pub fn is_spawn_position_valid(
        &self,
        position: Vec3,
        content_type: ContentType,
        road_network: Option<&RoadNetwork>,
    ) -> bool {
        // Check if on road (invalid for buildings and trees)
        if let Some(roads) = road_network {
            let road_tolerance: f32 = match content_type {
                ContentType::Building => 25.0,
                ContentType::Tree => 15.0,
                ContentType::Vehicle => -8.0, // Negative means vehicles NEED roads
                ContentType::NPC => 0.0,      // NPCs can be anywhere
                _ => 10.0,
            };

            let on_road = is_on_road_spline(position, roads, road_tolerance.abs());

            match content_type {
                ContentType::Vehicle => {
                    if !on_road {
                        return false;
                    } // Vehicles need roads
                }
                ContentType::Building | ContentType::Tree => {
                    if on_road {
                        return false;
                    } // Buildings/trees avoid roads
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

        let distance = Vec2::new(position.x - lake_center.x, position.z - lake_center.z).length();

        distance < (lake_size / 2.0 + buffer)
    }

    /// Check for content collision with existing entities
    pub fn has_content_collision(
        &self,
        position: Vec3,
        content_type: ContentType,
        existing_content: &[(Vec3, ContentType, f32)],
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
            ContentType::Building => self.spawn_building_consolidated(
                commands,
                meshes,
                materials,
                validated_position,
                current_time,
            )?,
            ContentType::Vehicle => self.spawn_vehicle_consolidated(
                commands,
                meshes,
                materials,
                validated_position,
                current_time,
            )?,
            ContentType::NPC => self.spawn_npc_consolidated(
                commands,
                meshes,
                materials,
                validated_position,
                current_time,
            )?,
            ContentType::Tree => self.spawn_tree_consolidated(
                commands,
                meshes,
                materials,
                validated_position,
                current_time,
            )?,
            _ => return Ok(None), // Unsupported content type
        };

        // Enforce entity limits and track the new entity
        self.entity_limits
            .enforce_limit(commands, content_type, entity, current_time);

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
        let building_entity = commands
            .spawn((
                // Use dynamic content bundle for compatibility
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
                Name::new(format!(
                    "Building_{:.0}_{:.0}_{}",
                    position.x, position.z, current_time
                )),
            ))
            .id();

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
            Color::srgb(1.0, 0.0, 0.0),
            Color::srgb(0.0, 0.0, 1.0),
            Color::srgb(0.0, 1.0, 0.0),
            Color::srgb(1.0, 1.0, 0.0),
            Color::srgb(1.0, 0.0, 1.0),
            Color::srgb(0.0, 1.0, 1.0),
            Color::srgb(0.5, 0.5, 0.5),
            Color::srgb(1.0, 1.0, 1.0),
            Color::srgb(0.0, 0.0, 0.0),
        ];
        let color = car_colors[rng.gen_range(0..car_colors.len())];

        // Position vehicle on ground surface
        let ground_level = self.get_ground_height(Vec2::new(position.x, position.z));
        let final_position = Vec3::new(position.x, ground_level + 0.5, position.z);

        // Create vehicle entity using consolidated bundle approach
        let vehicle_entity = commands
            .spawn((
                // Dynamic content bundle for compatibility
                DynamicPhysicsBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::Vehicle,
                    },
                    transform: Transform::from_translation(final_position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::cuboid(1.0, 0.5, 2.0),
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
                // Vehicle-specific components
                Car,
                LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
                Damping {
                    linear_damping: 1.0,
                    angular_damping: 5.0,
                },
                MovementTracker::new(final_position, 10.0),
                Name::new(format!(
                    "Vehicle_{:.0}_{:.0}_{}",
                    position.x, position.z, current_time
                )),
            ))
            .id();

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
            Color::srgb(0.8, 0.6, 0.4),
            Color::srgb(0.6, 0.4, 0.3),
            Color::srgb(0.9, 0.7, 0.5),
            Color::srgb(0.7, 0.5, 0.4),
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
        let npc_entity = commands
            .spawn((
                // Dynamic physics bundle
                DynamicPhysicsBundle {
                    dynamic_content: DynamicContent {
                        content_type: ContentType::NPC,
                    },
                    transform: Transform::from_translation(final_position),
                    visibility: Visibility::default(),
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::capsule(
                        Vec3::new(0.0, -0.9, 0.0),
                        Vec3::new(0.0, 0.9, 0.0),
                        0.3,
                    ),
                    collision_groups: CollisionGroups::new(
                        self.config.physics.character_group,
                        Group::ALL,
                    ),
                    velocity: Velocity::default(),
                    visibility_range: VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 130.0..150.0,
                        use_aabb: false,
                    },
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
                Name::new(format!(
                    "NPC_{:.0}_{:.0}_{}",
                    position.x, position.z, current_time
                )),
            ))
            .id();

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
                Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.4, 0.2, 0.1))),
                Name::new(format!(
                    "Tree_{:.0}_{:.0}_{}",
                    position.x, position.z, current_time
                )),
            ))
            .id();

        Ok(tree_entity)
    }
}

/// NPC CREATION METHODS
impl UnifiedEntityFactory {
    /// UNUSED: Add NPC visual components (head, body, limbs) - TODO: Remove after refactoring
    #[allow(dead_code)]
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
        let head_material =
            MaterialFactory::create_simple_material(materials, appearance.skin_tone);

        commands.spawn((
            Mesh3d(head_mesh),
            MeshMaterial3d(head_material),
            Transform::from_xyz(0.0, appearance.height * 0.85, 0.0),
            VisibilityRange::abrupt(0.0, self.config.world.lod_distances.get(0).copied().unwrap_or(50.0)),
            ChildOf(parent_entity),
        ));

        // Body
        let body_mesh = meshes.add(Cuboid::new(
            0.4 * appearance.build,
            0.6 * appearance.height,
            0.2 * appearance.build,
        ));
        let body_material =
            MaterialFactory::create_simple_material(materials, appearance.shirt_color);

        commands.spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_material),
            Transform::from_xyz(0.0, appearance.height * 0.5, 0.0),
            VisibilityRange::abrupt(0.0, self.config.world.lod_distances.get(0).copied().unwrap_or(50.0)),
            ChildOf(parent_entity),
        ));

        Ok(())
    }
}

/// BUILDING & ENVIRONMENT CREATION METHODS
impl UnifiedEntityFactory {
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

        let terrain_entity = commands
            .spawn((
                bundle,
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                DynamicTerrain,
            ))
            .id();

        Ok(terrain_entity)
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
        let validated_position =
            self.validate_position(position - Vec3::new(0.0, depth / 2.0, 0.0))?;

        // Create physics bundle for water bottom
        let bottom_bundle = GenericBundleFactory::physics_object(
            validated_position,
            ColliderShape::Cylinder {
                radius: size.x / 2.0,
                height: depth,
            },
            10000.0, // Heavy water body
            self.config.physics.static_group,
            false, // Static water body
            &self.config,
        )?;

        // Water bottom
        let bottom_mesh = meshes.add(Cylinder::new(size.x / 2.0, depth));
        let bottom_material =
            MaterialFactory::create_water_bottom_material(materials, Color::srgb(0.3, 0.25, 0.2));

        let water_entity = commands
            .spawn((
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
            ))
            .id();

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
            VisibilityRange::abrupt(0.0, self.config.world.lod_distances.get(2).copied().unwrap_or(300.0)),
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

        let effect_entity = commands
            .spawn((
                bundle,
                Mesh3d(mesh),
                MeshMaterial3d(material_handle),
                ExhaustFlame,
                VisibilityRange::abrupt(0.0, self.config.world.lod_distances.get(0).copied().unwrap_or(50.0)),
            ))
            .id();

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
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                transform,
                ChildOf(parent),
                VisibleChildBundle::default(),
            ))
            .id()
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
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                transform,
                ChildOf(parent),
                VisibleChildBundle::default(),
                Name::new(name.to_string()),
            ))
            .id()
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

        let road_entity = commands
            .spawn((
                Transform::from_translation(validated_position),
                RigidBody::Fixed,
                Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
                CollisionGroups::new(
                    self.config.physics.static_group,
                    self.config.physics.vehicle_group | self.config.physics.character_group,
                ),
                RoadEntity { road_id: 0 },
                Name::new("Road Segment"),
            ))
            .id();

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
    /// Spawn multiple vehicles in batch using consolidated methods
    pub fn spawn_vehicle_batch(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vehicle_specs: Vec<(VehicleType, Vec3, Color)>,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();

        for (_vehicle_type, position, _color) in vehicle_specs {
            if let Some(entity) = self.spawn_entity_consolidated(
                commands,
                meshes,
                materials,
                ContentType::Vehicle,
                position,
                road_network,
                existing_content,
                current_time,
            )? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    /// Spawn multiple NPCs in batch using consolidated methods
    pub fn spawn_npc_batch(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        npc_specs: Vec<(Vec3, NPCAppearance)>,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();

        for (position, _appearance) in npc_specs {
            if let Some(entity) = self.spawn_entity_consolidated(
                commands,
                meshes,
                materials,
                ContentType::NPC,
                position,
                road_network,
                existing_content,
                current_time,
            )? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    /// Spawn multiple buildings in batch using consolidated methods
    pub fn spawn_building_batch(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        building_specs: Vec<(Vec3, Vec3, BuildingType, Color)>,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();

        for (position, _size, _building_type, _color) in building_specs {
            if let Some(entity) = self.spawn_entity_consolidated(
                commands,
                meshes,
                materials,
                ContentType::Building,
                position,
                road_network,
                existing_content,
                current_time,
            )? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }
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
