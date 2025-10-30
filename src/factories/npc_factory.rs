use crate::bundles::VisibleChildBundle;
use crate::components::world::NPCGender;
use crate::components::{
    BodyPart, HumanAnimation, HumanMovement, NPC, NPCAppearance, NPCHead, NPCLeftArm, NPCLeftFoot,
    NPCLeftLeg, NPCRightArm, NPCRightFoot, NPCRightLeg, NPCState, NPCTorso,
};
use crate::config::GameConfig;
use crate::factories::generic_bundle::BundleError;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_rapier3d::prelude::*;
use rand::Rng;

/// NPC Factory - Focused factory for NPC spawning only
/// Handles various NPC types with proper physics, AI, and visual components
/// Follows AGENT.md simplicity principles with single responsibility
///
/// ## NPC Component Requirements for Movement
///
/// For NPCs to move, they MUST have ALL of:
/// - NPC component (legacy movement state: target_position, speed, update_interval)
/// - Transform + GlobalTransform (position/rotation)
/// - Velocity (for Rapier physics movement)
/// - VisibilityRange (REQUIRED by movement query filter: With<VisibilityRange>)
/// - RigidBody::Dynamic + Collider (for physics)
/// - Mesh + Material (for rendering)
///
/// Movement system query: Query<(..., &mut NPC), With<VisibilityRange>>
/// Missing any component = NPC won't move
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

    /// Get visibility range for NPCs based on config
    fn visibility_range(&self) -> VisibilityRange {
        VisibilityRange::abrupt(0.0, self.config.performance.npc_visibility_distance)
    }

    /// Spawn NPC with automatic appearance generation
    pub fn spawn_npc(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        cache: &mut ResMut<crate::resources::NPCAssetCache>,
        position: Vec3,
        npc_type: Option<NPCType>,
    ) -> Result<Entity, BundleError> {
        let npc_type = npc_type.unwrap_or_else(|| self.random_npc_type());
        let appearance = self.generate_npc_appearance(npc_type);

        // Position NPC on ground with proper height
        let final_position = Vec3::new(position.x, position.y + 0.45, position.z);

        // Player-like collider setup
        const FOOT_LEVEL: f32 = -0.45;
        const CAPSULE_RADIUS: f32 = 0.25;
        const LOWER_SPHERE_Y: f32 = FOOT_LEVEL + CAPSULE_RADIUS;
        const UPPER_SPHERE_Y: f32 = 1.45;

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
            Collider::capsule(
                Vec3::new(0.0, LOWER_SPHERE_Y, 0.0),
                Vec3::new(0.0, UPPER_SPHERE_Y, 0.0),
                CAPSULE_RADIUS,
            ),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        ));

        entity.insert((
            CollisionGroups::new(
                self.config.physics.character_group,
                self.config.physics.static_group | self.config.physics.vehicle_group,
            ),
            Velocity::default(),
            Damping {
                linear_damping: 0.1, // Realistic air resistance for free-fall
                angular_damping: 3.5,
            },
            Sleeping::disabled(),
        ));

        entity.insert((
            Name::new(format!("NPC_{}", npc_type.name())),
            self.visibility_range(),
            HumanMovement::default(),
            HumanAnimation::default(),
            crate::components::unified_water::CurrentWaterRegion::default(),
        ));

        let npc_entity = entity.id();

        // Spawn player-like body parts with NPC appearance colors
        self.spawn_npc_body_parts(commands, meshes, materials, cache, npc_entity, &appearance);

        Ok(npc_entity)
    }

    /// Spawn NPC with specific appearance
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_npc_with_appearance(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        cache: &mut ResMut<crate::resources::NPCAssetCache>,
        position: Vec3,
        npc_type: NPCType,
        appearance: NPCAppearance,
    ) -> Result<Entity, BundleError> {
        let final_position = Vec3::new(position.x, position.y + 0.45, position.z);

        // Player-like collider setup
        const FOOT_LEVEL: f32 = -0.45;
        const CAPSULE_RADIUS: f32 = 0.25;
        const LOWER_SPHERE_Y: f32 = FOOT_LEVEL + CAPSULE_RADIUS;
        const UPPER_SPHERE_Y: f32 = 1.45;

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
            Collider::capsule(
                Vec3::new(0.0, LOWER_SPHERE_Y, 0.0),
                Vec3::new(0.0, UPPER_SPHERE_Y, 0.0),
                CAPSULE_RADIUS,
            ),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        ));

        entity.insert((
            CollisionGroups::new(
                self.config.physics.character_group,
                self.config.physics.static_group | self.config.physics.vehicle_group,
            ),
            Velocity::default(),
            Damping {
                linear_damping: 0.1, // Realistic air resistance for free-fall
                angular_damping: 3.5,
            },
            Sleeping::disabled(),
        ));

        entity.insert((
            Name::new(format!("NPC_{}", npc_type.name())),
            self.visibility_range(),
            HumanMovement::default(),
            HumanAnimation::default(),
            crate::components::unified_water::CurrentWaterRegion::default(),
        ));

        let npc_entity = entity.id();

        // Spawn player-like body parts with NPC appearance colors
        self.spawn_npc_body_parts(commands, meshes, materials, cache, npc_entity, &appearance);

        Ok(npc_entity)
    }

    /// Spawn multiple NPCs in batch
    pub fn spawn_npc_batch(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        cache: &mut ResMut<crate::resources::NPCAssetCache>,
        positions: Vec<Vec3>,
        npc_type: Option<NPCType>,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();

        for position in positions {
            let entity = self.spawn_npc(commands, meshes, materials, cache, position, npc_type)?;
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

    /// Spawn player-like body parts for NPC
    fn spawn_npc_body_parts(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        cache: &mut ResMut<crate::resources::NPCAssetCache>,
        parent: Entity,
        appearance: &NPCAppearance,
    ) {
        // Torso
        commands.spawn((
            Mesh3d(
                cache
                    .get_or_create_mesh(crate::resources::MeshShape::cuboid(0.6, 0.8, 0.3), meshes),
            ),
            MeshMaterial3d(cache.get_or_create_material(appearance.shirt_color, materials)),
            Transform::from_xyz(0.0, 0.6, 0.0),
            ChildOf(parent),
            NPCTorso,
            BodyPart {
                rest_position: Vec3::new(0.0, 0.6, 0.0),
                rest_rotation: Quat::IDENTITY,
                animation_offset: Vec3::ZERO,
                animation_rotation: Quat::IDENTITY,
            },
            VisibleChildBundle::default(),
        ));

        // Head
        commands.spawn((
            Mesh3d(cache.get_or_create_mesh(crate::resources::MeshShape::sphere(0.2), meshes)),
            MeshMaterial3d(cache.get_or_create_material(appearance.skin_tone, materials)),
            Transform::from_xyz(0.0, 1.2, 0.0),
            ChildOf(parent),
            NPCHead,
            BodyPart {
                rest_position: Vec3::new(0.0, 1.2, 0.0),
                rest_rotation: Quat::IDENTITY,
                animation_offset: Vec3::ZERO,
                animation_rotation: Quat::IDENTITY,
            },
            VisibleChildBundle::default(),
        ));

        // Left Arm
        commands.spawn((
            Mesh3d(
                cache.get_or_create_mesh(crate::resources::MeshShape::capsule(0.08, 0.5), meshes),
            ),
            MeshMaterial3d(cache.get_or_create_material(appearance.skin_tone, materials)),
            Transform::from_xyz(-0.4, 0.7, 0.0),
            ChildOf(parent),
            NPCLeftArm,
            BodyPart {
                rest_position: Vec3::new(-0.4, 0.7, 0.0),
                rest_rotation: Quat::IDENTITY,
                animation_offset: Vec3::ZERO,
                animation_rotation: Quat::IDENTITY,
            },
            VisibleChildBundle::default(),
        ));

        // Right Arm
        commands.spawn((
            Mesh3d(
                cache.get_or_create_mesh(crate::resources::MeshShape::capsule(0.08, 0.5), meshes),
            ),
            MeshMaterial3d(cache.get_or_create_material(appearance.skin_tone, materials)),
            Transform::from_xyz(0.4, 0.7, 0.0),
            ChildOf(parent),
            NPCRightArm,
            BodyPart {
                rest_position: Vec3::new(0.4, 0.7, 0.0),
                rest_rotation: Quat::IDENTITY,
                animation_offset: Vec3::ZERO,
                animation_rotation: Quat::IDENTITY,
            },
            VisibleChildBundle::default(),
        ));

        // Left Leg
        commands.spawn((
            Mesh3d(
                cache.get_or_create_mesh(crate::resources::MeshShape::capsule(0.12, 0.6), meshes),
            ),
            MeshMaterial3d(cache.get_or_create_material(appearance.pants_color, materials)),
            Transform::from_xyz(-0.15, 0.0, 0.0),
            ChildOf(parent),
            NPCLeftLeg,
            BodyPart {
                rest_position: Vec3::new(-0.15, 0.0, 0.0),
                rest_rotation: Quat::IDENTITY,
                animation_offset: Vec3::ZERO,
                animation_rotation: Quat::IDENTITY,
            },
            VisibleChildBundle::default(),
        ));

        // Right Leg
        commands.spawn((
            Mesh3d(
                cache.get_or_create_mesh(crate::resources::MeshShape::capsule(0.12, 0.6), meshes),
            ),
            MeshMaterial3d(cache.get_or_create_material(appearance.pants_color, materials)),
            Transform::from_xyz(0.15, 0.0, 0.0),
            ChildOf(parent),
            NPCRightLeg,
            BodyPart {
                rest_position: Vec3::new(0.15, 0.0, 0.0),
                rest_rotation: Quat::IDENTITY,
                animation_offset: Vec3::ZERO,
                animation_rotation: Quat::IDENTITY,
            },
            VisibleChildBundle::default(),
        ));

        // Reuse foot mesh and material for both feet
        let foot_mesh =
            cache.get_or_create_mesh(crate::resources::MeshShape::cuboid(0.2, 0.1, 0.35), meshes);
        let foot_material = cache.get_or_create_material(Color::srgb(0.1, 0.1, 0.1), materials);

        // Left Foot
        commands.spawn((
            Mesh3d(foot_mesh.clone()),
            MeshMaterial3d(foot_material.clone()),
            Transform::from_xyz(-0.15, -0.4, 0.1),
            ChildOf(parent),
            NPCLeftFoot,
            BodyPart {
                rest_position: Vec3::new(-0.15, -0.4, 0.1),
                rest_rotation: Quat::IDENTITY,
                animation_offset: Vec3::ZERO,
                animation_rotation: Quat::IDENTITY,
            },
            VisibleChildBundle::default(),
        ));

        // Right Foot
        commands.spawn((
            Mesh3d(foot_mesh),
            MeshMaterial3d(foot_material),
            Transform::from_xyz(0.15, -0.4, 0.1),
            ChildOf(parent),
            NPCRightFoot,
            BodyPart {
                rest_position: Vec3::new(0.15, -0.4, 0.1),
                rest_rotation: Quat::IDENTITY,
                animation_offset: Vec3::ZERO,
                animation_rotation: Quat::IDENTITY,
            },
            VisibleChildBundle::default(),
        ));
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
