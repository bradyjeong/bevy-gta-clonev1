// Simplified entity builder for now - can be enhanced later
use bevy::prelude::*;
use game_core::prelude::*;

/// Entity builder extension trait - placeholder for now
pub trait EntityBuilderExt {
    fn new_entity(&mut self, commands: &mut Commands) -> Entity;
}

impl EntityBuilderExt for crate::factories::entity_factory::UnifiedEntityFactory {
    fn new_entity(&mut self, commands: &mut Commands) -> Entity {
        commands.spawn_empty().id()
    }
}
