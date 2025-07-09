use bevy::prelude::*;
use crate::systems::input::{
    InputConfig, InputManager, ControlManager, VehicleControlConfig,
    process_input_system, update_control_manager_system, apply_ai_controls_system,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<InputConfig>()
            .init_resource::<InputManager>()

            .init_resource::<ControlManager>()
            .init_resource::<VehicleControlConfig>();
        
        // Core input processing system - runs first in PreUpdate
        app.add_systems(PreUpdate, (
            process_input_system,

        ));
        
        // New unified control systems - run in Update
        app.add_systems(Update, (
            update_control_manager_system,
            apply_ai_controls_system.after(update_control_manager_system),
        ));
        
        info!("Input Plugin initialized with unified controls");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::game_state::GameState;
    use bevy::input::InputPlugin as BevyInputPlugin;
    
    #[test]
    fn test_input_plugin_initialization() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            BevyInputPlugin,
        ))
        .init_state::<GameState>()
        .add_plugins(InputPlugin);
        
        // Verify resources are initialized
        assert!(app.world().get_resource::<InputConfig>().is_some());
        assert!(app.world().get_resource::<InputManager>().is_some());

        assert!(app.world().get_resource::<ControlManager>().is_some());
        assert!(app.world().get_resource::<VehicleControlConfig>().is_some());
    }
}
