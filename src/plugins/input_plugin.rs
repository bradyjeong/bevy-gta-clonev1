use bevy::prelude::*;
use crate::systems::input::{
    InputConfig, InputManager, InputCompatLayer,
    process_input_system, update_input_compat_layer
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<InputConfig>()
            .init_resource::<InputManager>()
            .init_resource::<InputCompatLayer>();
        
        // Core input processing system - runs first in PreUpdate
        app.add_systems(PreUpdate, (
            process_input_system,
            update_input_compat_layer.after(process_input_system),
        ));
        
        info!("Input Plugin initialized with backwards compatibility layer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;
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
        assert!(app.world().get_resource::<InputCompatLayer>().is_some());
    }
}
