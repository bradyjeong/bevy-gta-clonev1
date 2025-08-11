use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::bundles::DynamicPhysicsBundle;
use crate::systems::{UnifiedCullable, MovementTracker};
use crate::factories::common::{FocusedFactory, GroundHeightCache, SpawnValidation, PhysicsSetup, EntityPhysicsType};
use crate::systems::RoadNetwork;
use crate::GameConfig;

/// Focused factory for NPC entities following AGENT.md simplicity principles
/// Single responsibility: NPC creation with consistent AI and physics
pub struct NPCFactory {}

impl Default for NPCFactory {
    fn default() -> Self {
        Self {}
    }
}

impl FocusedFactory for NPCFactory {
    fn name() -> &'static str {
        "NPCFactory"
    }
    
    fn entity_limit() -> usize {
        2 // From AGENT.md: 1% of 200 = 2 NPCs
    }
}

impl NPCFactory {
    /// Create new NPC factory
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Spawn NPC entity with complete setup
    pub fn spawn_npc(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Vec3,
        config: &GameConfig,
        current_time: f32,
        road_network: Option<&RoadNetwork>,
        ground_cache: &mut GroundHeightCache,
    ) -> Result<Entity, String> {
        let mut rng = rand::thread_rng();
        
        // Validate spawn position
        if !SpawnValidation::is_position_valid(position, ContentType::NPC, road_network) {
            return Err("Invalid position for NPC".to_string());
        }
        
        // Position NPC on ground
        let ground_level = ground_cache.get_ground_height(Vec2::new(position.x, position.z));
        let final_position = Vec3::new(position.x, ground_level + 1.0, position.z);
        
        // Random NPC appearance
        let npc_colors = [
            Color::srgb(0.8, 0.6, 0.4), Color::srgb(0.6, 0.4, 0.3),
            Color::srgb(0.9, 0.7, 0.5), Color::srgb(0.7, 0.5, 0.4),
        ];
        let color = npc_colors[rng.gen_range(0..npc_colors.len())];
        
        // Random target position for movement
        let target_x = rng.gen_range(-900.0..900.0);
        let target_z = rng.gen_range(-900.0..900.0);
        let target_position = Vec3::new(target_x, ground_level + 1.0, target_z);
        
        // Create NPC entity
        let npc_entity = commands.spawn((
            DynamicPhysicsBundle {
                dynamic_content: DynamicContent { content_type: ContentType::NPC },
                transform: Transform::from_translation(final_position),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
                rigid_body: RigidBody::Dynamic,
                collider: Collider::capsule(Vec3::new(0.0, -0.9, 0.0), Vec3::new(0.0, 0.9, 0.0), 0.3),
                collision_groups: PhysicsSetup::collision_groups_for_entity(EntityPhysicsType::Character, config),
                velocity: Velocity::default(),
                cullable: UnifiedCullable::npc(),
            },
            // NPC-specific components
            PhysicsSetup::locked_axes_for_entity(EntityPhysicsType::Character).unwrap(),
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
    
    /// Spawn batch of NPCs
    pub fn spawn_batch(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        positions: Vec<Vec3>,
        config: &GameConfig,
        current_time: f32,
        road_network: Option<&RoadNetwork>,
        ground_cache: &mut GroundHeightCache,
    ) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for position in positions {
            if let Ok(entity) = self.spawn_npc(
                commands, 
                meshes, 
                materials, 
                position, 
                config, 
                current_time,
                road_network,
                ground_cache
            ) {
                entities.push(entity);
            }
        }
        
        entities
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum NPCType {
    Pedestrian,
    Worker,
    Police,
}

impl Default for NPCType {
    fn default() -> Self {
        Self::Pedestrian
    }
}
