use bevy::prelude::*;
use game_core::prelude::*;

#[derive(Debug, Clone)]
pub struct EntityLimitManager {
    pub max_buildings: usize,
    pub max_vehicles: usize,
    pub max_npcs: usize,
    pub max_trees: usize,
    pub max_particles: usize,
    pub building_entities: Vec<(Entity, f32)>,
    pub vehicle_entities: Vec<(Entity, f32)>,
    pub npc_entities: Vec<(Entity, f32)>,
    pub tree_entities: Vec<(Entity, f32)>,
    pub particle_entities: Vec<(Entity, f32)>,
}

impl Default for EntityLimitManager {
    fn default() -> Self {
        Self {
            max_buildings: 80,
            max_vehicles: 20,
            max_npcs: 2,
            max_trees: 100,
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
    pub fn configure_from_config(&mut self, cfg: &GameConfig) {
        self.max_buildings = cfg.entity_limits.buildings;
        self.max_vehicles = cfg.entity_limits.vehicles;
        self.max_npcs = cfg.entity_limits.npcs;
        self.max_trees = cfg.entity_limits.trees;
        self.max_particles = cfg.entity_limits.particles;
    }

    pub fn enforce_limit(
        &mut self,
        commands: &mut Commands,
        kind: ContentType,
        entity: Entity,
        now: f32,
    ) {
        let (vec, limit) = match kind {
            ContentType::Building => (&mut self.building_entities, self.max_buildings),
            ContentType::Vehicle => (&mut self.vehicle_entities, self.max_vehicles),
            ContentType::NPC => (&mut self.npc_entities, self.max_npcs),
            ContentType::Tree => (&mut self.tree_entities, self.max_trees),
            _ => return,
        };
        if vec.len() >= limit {
            if let Some((oldest, _)) = vec.first().copied() {
                commands.entity(oldest).despawn();
                vec.remove(0);
            }
        }
        vec.push((entity, now));
    }

    pub fn counts(&self) -> (usize, usize, usize, usize) {
        (
            self.building_entities.len(),
            self.vehicle_entities.len(),
            self.npc_entities.len(),
            self.tree_entities.len(),
        )
    }
}
