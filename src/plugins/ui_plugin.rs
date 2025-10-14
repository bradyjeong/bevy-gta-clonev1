use crate::states::AppState;
use crate::systems::effects::update_waypoint_system;
use crate::systems::ui::{
    controls_ui_system, load_initial_assets, setup_fps_display, update_asset_loading,
    update_fps_display,
};
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::systems::ui::splash_screen::AssetLoadingState>()
            .add_systems(OnEnter(AppState::AssetLoading), load_initial_assets)
            .add_systems(
                Update,
                update_asset_loading.run_if(in_state(AppState::AssetLoading)),
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
