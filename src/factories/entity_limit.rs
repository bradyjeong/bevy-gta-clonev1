use bevy::prelude::*;
use crate::components::ContentType;

/// Entity limit manager with configurable thresholds and automatic FIFO cleanup
/// Following AGENT.md simplicity principles: single responsibility, clear boundaries
#[derive(Debug, Clone, Resource)]
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
            // Configurable limits based on AGENT.md performance guidelines
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
    pub fn enforce_limit(&mut self, commands: &mut Commands, content_type: ContentType, entity: Entity, timestamp: f32) {
        match content_type {
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
    
    /// Check if we can spawn more entities of a given type
    pub fn can_spawn(&self, content_type: ContentType) -> bool {
        match content_type {
            ContentType::Building => self.building_entities.len() < self.max_buildings,
            ContentType::Vehicle => self.vehicle_entities.len() < self.max_vehicles,
            ContentType::NPC => self.npc_entities.len() < self.max_npcs,
            ContentType::Tree => self.tree_entities.len() < self.max_trees,
            _ => true,
        }
    }
}

// Legacy enum kept for backward compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Building,
    Vehicle,
    NPC,
    Tree,
}

#[derive(Debug, Clone, Copy)]
pub struct EntityLimit {
    pub max_count: usize,
    pub spawn_rate: f32,
}
