use bevy::prelude::*;
use crate::systems::effects::update_waypoint_system;
use crate::systems::ui::{setup_fps_display, update_fps_display, controls_ui_system};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_fps_display)
           .add_systems(Update, (
               controls_ui_system,
               update_waypoint_system,
               update_fps_display,
           ));
    }
}
