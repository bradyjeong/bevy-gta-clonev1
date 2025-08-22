use bevy::prelude::*;
use std::collections::HashMap;

/// Resource for tracking entity limits across all factory types
#[derive(Resource, Default)]
pub struct EntityLimitManager {
    pub limits: HashMap<EntityType, EntityLimit>,
    pub current_counts: HashMap<EntityType, usize>,
}

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
    pub spawn_rate: f32, // Percentage (0.0 to 1.0)
}

impl EntityLimitManager {
    pub fn new() -> Self {
        let mut limits = HashMap::new();

        // Set conservative limits as per AGENT.md performance guidelines
        limits.insert(
            EntityType::Building,
            EntityLimit {
                max_count: 200,
                spawn_rate: 0.08,
            },
        );
        limits.insert(
            EntityType::Vehicle,
            EntityLimit {
                max_count: 50,
                spawn_rate: 0.04,
            },
        );
        limits.insert(
            EntityType::NPC,
            EntityLimit {
                max_count: 20,
                spawn_rate: 0.01,
            },
        );
        limits.insert(
            EntityType::Tree,
            EntityLimit {
                max_count: 100,
                spawn_rate: 0.05,
            },
        );

        Self {
            limits,
            current_counts: HashMap::new(),
        }
    }

    pub fn can_spawn(&self, entity_type: EntityType) -> bool {
        let current = self.current_counts.get(&entity_type).unwrap_or(&0);
        let limit = self
            .limits
            .get(&entity_type)
            .map(|l| l.max_count)
            .unwrap_or(0);
        *current < limit
    }

    pub fn register_spawn(&mut self, entity_type: EntityType) {
        *self.current_counts.entry(entity_type).or_insert(0) += 1;
    }

    pub fn register_despawn(&mut self, entity_type: EntityType) {
        if let Some(count) = self.current_counts.get_mut(&entity_type) {
            *count = count.saturating_sub(1);
        }
    }

    pub fn get_spawn_rate(&self, entity_type: EntityType) -> f32 {
        self.limits
            .get(&entity_type)
            .map(|l| l.spawn_rate)
            .unwrap_or(0.0)
    }
}
