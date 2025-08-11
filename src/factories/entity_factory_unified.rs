use bevy::prelude::*;
use crate::components::*;
use crate::factories::{
    BuildingsFactory, VehicleFactory, NPCFactory, VegetationFactory
};
use crate::factories::common::GroundHeightCache;
use crate::factories::generic_bundle::BundleError;
use crate::systems::{RoadNetwork, is_on_road_spline};
use crate::GameConfig;

/// Thin coordinator that delegates to focused factories following AGENT.md simplicity principles
/// Single responsibility: coordinate between focused factories and maintain compatibility
#[derive(Resource)]
pub struct UnifiedEntityFactory {
    pub config: GameConfig,
    /// Entity limit management with configurable thresholds  
    pub entity_limits: EntityLimitManager,
    /// Shared ground height cache used by all focused factories
    pub ground_cache: GroundHeightCache,
    /// Focused factory instances
    pub buildings_factory: BuildingsFactory,
    pub vehicle_factory: VehicleFactory,
    pub npc_factory: NPCFactory,
    pub vegetation_factory: VegetationFactory,
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
            ground_cache: GroundHeightCache::default(),
            buildings_factory: BuildingsFactory::new(),
            vehicle_factory: VehicleFactory::new(),
            npc_factory: NPCFactory::new(),
            vegetation_factory: VegetationFactory::new(),
        }
    }
    
    /// Create factory with custom configuration
    pub fn with_config(config: GameConfig) -> Self {
        Self {
            config,
            entity_limits: EntityLimitManager::default(),
            ground_cache: GroundHeightCache::default(),
            buildings_factory: BuildingsFactory::new(),
            vehicle_factory: VehicleFactory::new(),
            npc_factory: NPCFactory::new(),
            vegetation_factory: VegetationFactory::new(),
        }
    }
    
    /// Validate position is within world bounds
    pub fn validate_position(&self, position: Vec3) -> Result<Vec3, BundleError> {
        if position.x.abs() > self.config.gameplay.physics.max_world_coord ||
           position.z.abs() > self.config.gameplay.physics.max_world_coord {
            return Err(BundleError::PositionOutOfBounds {
                position,
                max_coord: self.config.gameplay.physics.max_world_coord,
            });
        }
        
        Ok(position.clamp(
            Vec3::splat(self.config.gameplay.physics.min_world_coord),
            Vec3::splat(self.config.gameplay.physics.max_world_coord),
        ))
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
    /// Master spawn method - delegates to focused factories and handles limits
    pub fn spawn_entity_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        content_type: ContentType,
        position: Vec3,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Option<Entity>, BundleError> {
        // Validate position first
        let validated_position = self.validate_position(position)?;
        
        // Check for collisions with existing content
        if self.has_content_collision(validated_position, content_type, existing_content) {
            return Ok(None); // Collision detected, but no error
        }
        
        // Delegate to appropriate focused factory with shared ground cache and road network
        let entity_result = match content_type {
            ContentType::Building => {
                self.buildings_factory.spawn_building(
                    commands, meshes, materials, validated_position, &self.config, current_time,
                    road_network, &mut self.ground_cache
                ).map_err(|e| BundleError::InvalidConfiguration(e))
            }
            ContentType::Vehicle => {
                self.vehicle_factory.spawn_vehicle(
                    commands, meshes, materials, validated_position, &self.config, current_time,
                    road_network, &mut self.ground_cache
                ).map_err(|e| BundleError::InvalidConfiguration(e))
            }
            ContentType::NPC => {
                self.npc_factory.spawn_npc(
                    commands, meshes, materials, validated_position, &self.config, current_time,
                    road_network, &mut self.ground_cache
                ).map_err(|e| BundleError::InvalidConfiguration(e))
            }
            ContentType::Tree => {
                self.vegetation_factory.spawn_vegetation(
                    commands, meshes, materials, validated_position, &self.config, current_time,
                    road_network, &mut self.ground_cache
                ).map_err(|e| BundleError::InvalidConfiguration(e))
            }
            _ => return Ok(None), // Unsupported content type
        };
        
        // Handle result and enforce limits
        match entity_result {
            Ok(entity) => {
                self.entity_limits.enforce_limit(commands, content_type, entity, current_time);
                Ok(Some(entity))
            }
            Err(bundle_error) => Err(bundle_error),
        }
    }
    
    /// Legacy building spawn for backward compatibility - delegates to focused factory
    pub fn spawn_building_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        self.buildings_factory.spawn_building(
            commands, meshes, materials, position, &self.config, current_time,
            None, &mut self.ground_cache
        ).map_err(|e| BundleError::InvalidConfiguration(e))
    }
    
    /// Legacy vehicle spawn for backward compatibility - delegates to focused factory
    pub fn spawn_vehicle_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        self.vehicle_factory.spawn_vehicle(
            commands, meshes, materials, position, &self.config, current_time,
            None, &mut self.ground_cache
        ).map_err(|e| BundleError::InvalidConfiguration(e))
    }
    
    /// Legacy NPC spawn for backward compatibility - delegates to focused factory
    pub fn spawn_npc_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        self.npc_factory.spawn_npc(
            commands, meshes, materials, position, &self.config, current_time,
            None, &mut self.ground_cache
        ).map_err(|e| BundleError::InvalidConfiguration(e))
    }
    
    /// Legacy tree spawn for backward compatibility - delegates to focused factory
    pub fn spawn_tree_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        self.vegetation_factory.spawn_vegetation(
            commands, meshes, materials, position, &self.config, current_time,
            None, &mut self.ground_cache
        ).map_err(|e| BundleError::InvalidConfiguration(e))
    }

    
    /// Batch spawn multiple entities - delegates to focused factories
    pub fn spawn_batch_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        content_type: ContentType,
        positions: Vec<Vec3>,
        _road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Vec<Entity>, BundleError> {
        // Filter valid positions to avoid collision checks inside factories
        let valid_positions: Vec<Vec3> = positions.into_iter()
            .filter_map(|pos| {
                let validated_pos = self.validate_position(pos).ok()?;
                if !self.has_content_collision(validated_pos, content_type, existing_content) {
                    Some(validated_pos)
                } else {
                    None
                }
            })
            .collect();
        
        // Use focused factory batch methods
        let spawned_entities = match content_type {
            ContentType::Building => self.buildings_factory.spawn_batch(
                commands, meshes, materials, valid_positions, &self.config, current_time,
                None, &mut self.ground_cache
            ),
            ContentType::Vehicle => self.vehicle_factory.spawn_batch(
                commands, meshes, materials, valid_positions, &self.config, current_time,
                None, &mut self.ground_cache
            ),
            ContentType::NPC => self.npc_factory.spawn_batch(
                commands, meshes, materials, valid_positions, &self.config, current_time,
                None, &mut self.ground_cache
            ),
            ContentType::Tree => self.vegetation_factory.spawn_batch(
                commands, meshes, materials, valid_positions, &self.config, current_time,
                None, &mut self.ground_cache
            ),
            _ => Vec::new(),
        };
        
        // Enforce limits for all spawned entities
        for entity in &spawned_entities {
            self.entity_limits.enforce_limit(commands, content_type, *entity, current_time);
        }
        
        Ok(spawned_entities)
    }
}

impl Default for UnifiedEntityFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// System to initialize UnifiedEntityFactory as a resource
pub fn setup_unified_entity_factory_basic(mut commands: Commands) {
    commands.insert_resource(UnifiedEntityFactory::default());
}
