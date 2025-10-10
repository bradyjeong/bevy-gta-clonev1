use crate::states::AppState;
use crate::systems::effects::update_waypoint_system;
use crate::systems::ui::{
    controls_ui_system, setup_fps_display, update_fps_display, setup_splash_screen,
    load_initial_assets, update_asset_loading, cleanup_splash_screen,
};
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<crate::systems::ui::splash_screen::AssetLoadingState>()
            .add_systems(OnEnter(AppState::AssetLoading), (setup_splash_screen, load_initial_assets))
            .add_systems(Update, update_asset_loading.run_if(in_state(AppState::AssetLoading)))
            .add_systems(
                OnEnter(AppState::WorldGeneration), 
                cleanup_splash_screen.after(crate::systems::ui::loading_screen::setup_loading_screen)
            )
            .add_systems(Startup, setup_fps_display)
            .add_systems(
                Update,
                (
                    controls_ui_system,
                    update_waypoint_system,
                    update_fps_display,
                ),
            );
    }
}
