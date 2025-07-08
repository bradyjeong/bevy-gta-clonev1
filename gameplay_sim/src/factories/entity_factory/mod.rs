use bevy::prelude::*;
use game_core::prelude::*;

pub mod limits;
pub mod validation;
pub mod spawner_building;
pub mod spawner_vehicle;
pub mod spawner_npc;
pub mod spawner_tree;

pub use limits::EntityLimitManager;

/// Unified Entity Factory - orchestrates all entity spawning
#[derive(Resource)]
#[derive(Default)]
pub struct UnifiedEntityFactory {
    pub limit_manager: EntityLimitManager,
}


impl UnifiedEntityFactory {
    #[must_use] pub fn new() -> Self {
        Self::default()
    }

    #[must_use] pub fn with_config(config: GameConfig) -> Self {
        let mut factory = Self::new();
        factory.configure_from_config(&config);
        factory
    }

    pub fn configure_from_config(&mut self, config: &GameConfig) {
        self.limit_manager.configure_from_config(config);
    }

    pub fn spawn_entity_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        content_type: ContentType,
        position: Vec3,
        _road_network: Option<&RoadNetwork>,
        existing_content: &[(Vec3, ContentType, f32)],
        current_time: f32,
    ) -> Result<Option<Entity>, BundleError> {
        let entity = match content_type {
            ContentType::Building => spawner_building::spawn_building(
                commands, position, meshes, materials, existing_content, current_time
            )?,
            ContentType::Vehicle => spawner_vehicle::spawn_vehicle(
                commands, position, meshes, materials, existing_content, current_time
            )?,
            ContentType::NPC => spawner_npc::spawn_npc(
                commands, position, meshes, materials, existing_content, current_time
            )?,
            ContentType::Tree => spawner_tree::spawn_tree(
                commands, position, meshes, materials, existing_content, current_time
            )?,
            _ => return Err(BundleError::NotImplemented),
        };

        if let Some(entity) = entity {
            self.limit_manager.enforce_limit(commands, content_type, entity, current_time);
        }

        Ok(entity)
    }

    /// Spawn NPC with consolidated logic
    pub fn spawn_npc_consolidated(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        current_time: f32,
    ) -> Result<Entity, BundleError> {
        let existing_content = Vec::new(); // Empty for now
        self.spawn_entity_consolidated(
            commands,
            meshes,
            materials,
            ContentType::NPC,
            position,
            None,
            &existing_content,
            current_time,
        ).map(|opt| opt.unwrap_or(Entity::from_raw(0)))
    }
}
