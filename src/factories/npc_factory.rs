use crate::components::world::NPCGender;
use crate::components::{NPC, NPCAppearance, NPCState};
use crate::config::GameConfig;
use crate::factories::generic_bundle::BundleError;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

/// NPC Factory - Focused factory for NPC spawning only
/// Handles various NPC types with proper physics, AI, and visual components
/// Follows AGENT.md simplicity principles with single responsibility
#[derive(Debug, Clone)]
pub struct NPCFactory {
    pub config: GameConfig,
}

impl NPCFactory {
    /// Create new NPC factory with default configuration
    pub fn new() -> Self {
        Self {
            config: GameConfig::default(),
        }
    }

    /// Create NPC factory with custom configuration
    pub fn with_config(config: GameConfig) -> Self {
        Self { config }
    }

    /// Spawn NPC with automatic appearance generation
    pub fn spawn_npc(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        npc_type: Option<NPCType>,
    ) -> Result<Entity, BundleError> {
        let npc_type = npc_type.unwrap_or_else(|| self.random_npc_type());
        let appearance = self.generate_npc_appearance(npc_type);

        // Position NPC on ground with proper height
        let final_position = Vec3::new(position.x, position.y + 1.0, position.z);

        let npc_material = materials.add(StandardMaterial {
            base_color: appearance.skin_tone,
            ..default()
        });

        let mut entity = commands.spawn((
            Transform::from_translation(final_position),
            Visibility::default(),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            NPC {
                target_position: position + Vec3::new(5.0, 0.0, 0.0),
                speed: self.get_npc_speed(npc_type),
                last_update: 0.0,
                update_interval: 0.5,
            },
        ));

        entity.insert((
            NPCState::new(npc_type.to_world_npc_type()),
            npc_type,
            appearance,
            RigidBody::Dynamic,
            Collider::capsule_y(0.3, 0.9),
        ));

        entity.insert((
            CollisionGroups::new(
                self.config.physics.character_group,
                self.config.physics.static_group | self.config.physics.vehicle_group,
            ),
            Velocity::default(),
            Damping {
                linear_damping: 2.0,
                angular_damping: 5.0,
            },
        ));

        entity.insert((
            Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
            MeshMaterial3d(npc_material),
            Name::new(format!("NPC_{}", npc_type.name())),
        ));

        let npc_entity = entity.id();
        Ok(npc_entity)
    }

    /// Spawn NPC with specific appearance
    pub fn spawn_npc_with_appearance(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        npc_type: NPCType,
        appearance: NPCAppearance,
    ) -> Result<Entity, BundleError> {
        let final_position = Vec3::new(position.x, position.y + 1.0, position.z);

        let npc_material = materials.add(StandardMaterial {
            base_color: appearance.skin_tone,
            ..default()
        });

        let mut entity = commands.spawn((
            Transform::from_translation(final_position),
            Visibility::default(),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            NPC {
                target_position: position + Vec3::new(5.0, 0.0, 0.0),
                speed: self.get_npc_speed(npc_type),
                last_update: 0.0,
                update_interval: 0.5,
            },
        ));

        entity.insert((
            NPCState::new(npc_type.to_world_npc_type()),
            npc_type,
            appearance,
            RigidBody::Dynamic,
            Collider::capsule_y(0.3, 0.9),
        ));

        entity.insert((
            CollisionGroups::new(
                self.config.physics.character_group,
                self.config.physics.static_group | self.config.physics.vehicle_group,
            ),
            Velocity::default(),
            Damping {
                linear_damping: 2.0,
                angular_damping: 5.0,
            },
        ));

        entity.insert((
            Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
            MeshMaterial3d(npc_material),
            Name::new(format!("NPC_{}", npc_type.name())),
        ));

        let npc_entity = entity.id();
        Ok(npc_entity)
    }

    /// Spawn multiple NPCs in batch
    pub fn spawn_npc_batch(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        positions: Vec<Vec3>,
        npc_type: Option<NPCType>,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();

        for position in positions {
            let entity = self.spawn_npc(commands, meshes, materials, position, npc_type)?;
            entities.push(entity);
        }

        Ok(entities)
    }

    /// Generate random NPC appearance
    fn generate_npc_appearance(&self, _npc_type: NPCType) -> NPCAppearance {
        let mut rng = rand::thread_rng();

        let skin_tones = [
            Color::srgb(0.8, 0.6, 0.4),
            Color::srgb(0.6, 0.4, 0.3),
            Color::srgb(0.9, 0.7, 0.5),
            Color::srgb(0.7, 0.5, 0.4),
        ];

        let shirt_colors = [
            Color::srgb(1.0, 0.0, 0.0),
            Color::srgb(0.0, 0.0, 1.0),
            Color::srgb(0.0, 1.0, 0.0),
            Color::srgb(1.0, 1.0, 0.0),
            Color::srgb(0.5, 0.5, 0.5),
        ];

        let hair_colors = [
            Color::srgb(0.1, 0.1, 0.1),
            Color::srgb(0.4, 0.2, 0.1),
            Color::srgb(0.8, 0.6, 0.2),
            Color::srgb(0.6, 0.3, 0.1),
        ];

        let pants_colors = [
            Color::srgb(0.2, 0.2, 0.8),
            Color::srgb(0.1, 0.1, 0.1),
            Color::srgb(0.4, 0.4, 0.4),
            Color::srgb(0.3, 0.2, 0.1),
        ];

        NPCAppearance {
            skin_tone: skin_tones[rng.gen_range(0..skin_tones.len())],
            hair_color: hair_colors[rng.gen_range(0..hair_colors.len())],
            shirt_color: shirt_colors[rng.gen_range(0..shirt_colors.len())],
            pants_color: pants_colors[rng.gen_range(0..pants_colors.len())],
            height: rng.gen_range(1.6..1.9),
            build: rng.gen_range(0.8..1.2),
            gender: if rng.gen_bool(0.5) {
                NPCGender::Male
            } else {
                NPCGender::Female
            },
        }
    }

    /// Get NPC speed based on type
    fn get_npc_speed(&self, npc_type: NPCType) -> f32 {
        let mut rng = rand::thread_rng();
        match npc_type {
            NPCType::Pedestrian => rng.gen_range(1.5..2.5),
            NPCType::Worker => rng.gen_range(2.0..3.0),
            NPCType::Police => rng.gen_range(2.5..3.5),
        }
    }

    /// Generate random NPC type
    fn random_npc_type(&self) -> NPCType {
        let mut rng = rand::thread_rng();
        let npc_types = [NPCType::Pedestrian, NPCType::Worker, NPCType::Police];
        npc_types[rng.gen_range(0..npc_types.len())]
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum NPCType {
    Pedestrian,
    Worker,
    Police,
}

impl NPCType {
    pub fn name(self) -> &'static str {
        match self {
            NPCType::Pedestrian => "Pedestrian",
            NPCType::Worker => "Worker",
            NPCType::Police => "Police",
        }
    }

    pub fn to_world_npc_type(self) -> crate::components::world::NPCType {
        match self {
            NPCType::Pedestrian => crate::components::world::NPCType::Civilian,
            NPCType::Worker => crate::components::world::NPCType::Civilian,
            NPCType::Police => crate::components::world::NPCType::Civilian,
        }
    }
}

impl Default for NPCType {
    fn default() -> Self {
        Self::Pedestrian
    }
}

impl Default for NPCFactory {
    fn default() -> Self {
        Self::new()
    }
}
