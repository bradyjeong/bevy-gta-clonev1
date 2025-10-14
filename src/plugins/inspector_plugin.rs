use bevy::prelude::*;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, #[allow(unused_variables)] app: &mut App) {
        #[cfg(feature = "debug-ui")]
        {
            use bevy_inspector_egui::quick::WorldInspectorPlugin;
            app.add_plugins(WorldInspectorPlugin::default());
        }
    }
}
