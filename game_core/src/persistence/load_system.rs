use bevy::prelude::*;
use std::collections::HashMap;


use super::save_system::*;

#[derive(Resource, Default)]
pub struct LoadState {
    pub entity_mapping: HashMap<u32, Entity>,
    pub pending_load: bool,
}

impl LoadState {
    pub fn new() -> Self {
        Self {
            entity_mapping: HashMap::new(),
            pending_load: false,
        }
    }
    
    pub fn clear(&mut self) {
        self.entity_mapping.clear();
        self.pending_load = false;
    }
    
    pub fn add_entity_mapping(&mut self, old_id: u32, new_entity: Entity) {
        self.entity_mapping.insert(old_id, new_entity);
    }
    
    pub fn get_entity(&self, old_id: u32) -> Option<Entity> {
        self.entity_mapping.get(&old_id).copied()
    }
}

pub fn load_save_file() -> Result<SaveGameState, String> {
    let save_path = "saves/savegame.ron";
    
    let content = std::fs::read_to_string(save_path)
        .map_err(|e| format!("Failed to read save file: {}", e))?;
    
    let save_data: SaveGameState = ron::from_str(&content)
        .map_err(|e| format!("Failed to parse save file: {}", e))?;

    Ok(save_data)
}
