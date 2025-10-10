use bevy::prelude::*;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "debug-ui")]
        {
            use bevy_inspector_egui::quick::WorldInspectorPlugin;
            _app.add_plugins(WorldInspectorPlugin::default().run_if(
                |state: Res<crate::states::AppState>| {
                    matches!(*state, crate::states::AppState::InGame)
                },
            ));
        }
    }
}
