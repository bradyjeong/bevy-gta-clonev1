use bevy::prelude::*;
use crate::systems::effects::update_waypoint_system;
use crate::systems::ui::{
    setup_fps_display, update_fps_display, controls_ui_system,
    bugatti_telemetry_input_system, update_bugatti_telemetry_system,
    BugattiTelemetryState
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BugattiTelemetryState>()
           .add_systems(Startup, setup_fps_display)
           .add_systems(Update, (
               controls_ui_system,
               update_waypoint_system,
               update_fps_display,
               bugatti_telemetry_input_system,
               update_bugatti_telemetry_system,
           ));
    }
}
