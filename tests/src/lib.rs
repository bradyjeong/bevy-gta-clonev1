//! Integration tests for the GTA game
#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

/// Simple addition function for testing
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
    
    /// Smoke test to verify the complete game runs without crashes in headless mode
    #[test]
    fn game_runs_for_a_few_frames() {
        use bevy::prelude::*;
        use gta_game::GamePlugin;
        
        // Create a minimal headless app
        let mut app = App::new();
        
        // Add minimal plugins to run in headless mode
        app.add_plugins(MinimalPlugins);
        
        // Add our game plugin
        app.add_plugins(GamePlugin);
        
        // Run for several frames to catch initialization and basic runtime issues
        for frame in 0..5 {
            println!("Running frame {}", frame);
            app.update();
        }
        
        println!("✅ Game successfully ran for 5 frames without crashes");
    }

    #[test]
    fn game_plugin_can_be_constructed() {
        use bevy::prelude::*;
        use gta_game::GamePlugin;
        
        // Verify the plugin can be constructed without panics
        let plugin = GamePlugin;
        
        // Create a minimal app to test plugin addition
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin);
        
        // Run one frame to trigger plugin initialization
        app.update();
        
        println!("✅ GamePlugin constructed and initialized successfully");
    }

    #[test]
    fn game_config_is_properly_initialized() {
        use bevy::prelude::*;
        use gta_game::GamePlugin;
        use game_core::config::GameConfig;
        
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(GamePlugin);
        
        // Run one frame to ensure all setup is complete
        app.update();
        
        // Verify GameConfig was inserted as a resource
        let config = app.world().get_resource::<GameConfig>();
        assert!(config.is_some(), "GameConfig should be inserted as a resource");
        
        println!("✅ GameConfig properly initialized");
    }
}
