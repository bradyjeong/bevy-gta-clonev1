use bevy::prelude::*;
use gta_game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(GamePlugin)
        .run();
}
