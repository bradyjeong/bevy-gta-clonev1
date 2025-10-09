use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct MapCamera;

#[derive(Component)]
pub struct MinimapUI;

#[derive(Component)]
pub struct PlayerMapIcon;

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct MapConfig {
    pub map_size: f32,
    pub map_height: f32,
    pub ui_position: (f32, f32),
    pub ui_size: (f32, f32),
    pub background_alpha: f32,
    pub border_width: f32,
    pub zoom_level: f32,
    pub show_player_icon: bool,
    pub player_icon_size: f32,
    pub player_icon_color: (f32, f32, f32, f32),
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            map_size: 200.0,
            map_height: 150.0,
            ui_position: (20.0, 20.0),
            ui_size: (200.0, 200.0),
            background_alpha: 0.8,
            border_width: 2.0,
            zoom_level: 1.0,
            show_player_icon: true,
            player_icon_size: 10.0,
            player_icon_color: (1.0, 0.0, 0.0, 1.0),
        }
    }
}
