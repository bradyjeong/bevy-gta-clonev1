use bevy::prelude::*;
use crate::components::*;
use crate::factories::{
    BuildingsFactory, VehicleFactory, NPCFactory, VegetationFactory,
    EntityLimitManager, GroundHeightCache
};
use crate::factories::generic_bundle::BundleError;
use crate::world::RoadNetwork;
use crate::plugins::spawn_validation_plugin::SpawnValidation;
use crate::GameConfig;

/// Thin coordinator that delegates to focused factories following AGENT.md simplicity principles
/// Single responsibility: coordinate between focused factories and maintain backward compatibility
/// Under 200 LOC, focused solely on delegation
#[derive(Resource)]
pub struct UnifiedEntityFactory {
    pub config: GameConfig,
    /// Entity limit management delegated to focused module
    pub entity_limits: EntityLimitManager,
    /// Shared ground height cache used by all focused factories
    pub ground_cache: GroundHeightCache,
    /// Focused factory instances (each under 500 LOC with single responsibility)
    pub buildings_factory: BuildingsFactory,
    pub vehicle_factory: VehicleFactory,
    pub npc_factory: NPCFactory,
    pub vegetation_factory: VegetationFactory,
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
    
    /// Validate position is within world bounds (backward compatibility)
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
}

/// Simplified spawn delegation methods following AGENT.md principles
impl UnifiedEntityFactory {
    /// Master spawn method - purely delegates to focused factories
    pub fn spawn_entity_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        content_type: ContentType,
        position: Vec3,
        road_network: Option<&RoadNetwork>,
        _existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Option<Entity>, BundleError> {
        // NOTE: Validation is performed upstream in the event-driven pipeline
        // When called through RequestDynamicSpawn events, position is already validated
        // Only perform basic bounds checking for safety
        if position.x.abs() > self.config.gameplay.physics.max_world_coord ||
           position.z.abs() > self.config.gameplay.physics.max_world_coord {
            debug!("Factory rejected {:?} at {:?} (out of bounds)", content_type, position);
            return Ok(None);
        }
        
        // Pure delegation to focused factories
        let entity_result = match content_type {
            ContentType::Building => {
                self.buildings_factory.spawn_building(
                    commands, meshes, materials, position, &self.config, current_time,
                    road_network, &mut self.ground_cache
                ).map_err(|e| BundleError::InvalidConfiguration(e))
            }
            ContentType::Vehicle => {
                self.vehicle_factory.spawn_vehicle(
                    commands, meshes, materials, position, &self.config, current_time,
                    road_network, &mut self.ground_cache
                ).map_err(|e| BundleError::InvalidConfiguration(e))
            }
            ContentType::NPC => {
                self.npc_factory.spawn_npc(
                    commands, meshes, materials, position, &self.config, current_time,
                    road_network, &mut self.ground_cache
                ).map_err(|e| BundleError::InvalidConfiguration(e))
            }
            ContentType::Tree => {
                self.vegetation_factory.spawn_vegetation(
                    commands, meshes, materials, position, &self.config, current_time,
                    road_network, &mut self.ground_cache
                ).map_err(|e| BundleError::InvalidConfiguration(e))
            }
            _ => return Ok(None),
        };
        
        // Delegate limit enforcement
        match entity_result {
            Ok(entity) => {
                self.entity_limits.enforce_limit(commands, content_type, entity, current_time);
                Ok(Some(entity))
            }
            Err(e) => Err(e),
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

    
    /// Batch spawn - pure delegation to focused factories
    pub fn spawn_batch_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        content_type: ContentType,
        positions: Vec<Vec3>,
        road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Vec<Entity>, BundleError> {
        // Filter valid positions using SpawnValidation
        let valid_positions: Vec<Vec3> = positions.into_iter()
            .filter(|&pos| {
                SpawnValidation::is_spawn_position_valid(
                    pos, 
                    content_type, 
                    self.config.gameplay.physics.max_world_coord,
                    road_network
                ) &&
                !SpawnValidation::has_content_collision(pos, content_type, existing_content)
            })
            .collect();
        
        // Pure delegation to focused factory batch methods
        let spawned_entities = match content_type {
            ContentType::Building => self.buildings_factory.spawn_batch(
                commands, meshes, materials, valid_positions, &self.config, current_time,
                road_network, &mut self.ground_cache
            ),
            ContentType::Vehicle => self.vehicle_factory.spawn_batch(
                commands, meshes, materials, valid_positions, &self.config, current_time,
                road_network, &mut self.ground_cache
            ),
            ContentType::NPC => self.npc_factory.spawn_batch(
                commands, meshes, materials, valid_positions, &self.config, current_time,
                road_network, &mut self.ground_cache
            ),
            ContentType::Tree => self.vegetation_factory.spawn_batch(
                commands, meshes, materials, valid_positions, &self.config, current_time,
                road_network, &mut self.ground_cache
            ),
            _ => Vec::new(),
        };
        
        // Delegate limit enforcement
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
