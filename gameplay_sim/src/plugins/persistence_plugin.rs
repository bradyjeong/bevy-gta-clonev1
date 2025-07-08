use bevy::prelude::*;
use crate::systems::persistence::{LoadState, load_game_system, save_game_system};

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LoadState>()
            .add_systems(Update, (
                save_game_system,
                load_game_system,
            ));
    }
}
