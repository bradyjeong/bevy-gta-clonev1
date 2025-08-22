use crate::systems::effects::update_waypoint_system;
use crate::systems::ui::{
    SplashScreenState, controls_ui_system, setup_fps_display, setup_splash_screen,
    update_fps_display, update_splash_screen,
};
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SplashScreenState>()
            .add_systems(Startup, (setup_splash_screen, setup_fps_display))
            .add_systems(
                Update,
                (
                    update_splash_screen,
                    controls_ui_system,
                    update_waypoint_system,
                    update_fps_display,
                ),
            );
    }
}
