//! ───────────────────────────────────────────────
//! System:   Config Loader
//! Purpose:  Loads and manages configuration settings
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use crate::config::GameConfig;

/// Configuration loader system for loading game config from RON files
pub struct ConfigLoaderPlugin;

impl Plugin for ConfigLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_game_config);
    }
}

/// Load game configuration from assets/config/game_config.ron
fn load_game_config(mut commands: Commands) {
    let config_path = "assets/config/game_config.ron";
    
    match GameConfig::load_from_file(config_path) {
        Ok(mut config) => {
            // Validate configuration values
            config.validate();
            
            info!("Successfully loaded game configuration from {}", config_path);
            
            // Insert as resource for global access
            commands.insert_resource(config);
        }
        Err(e) => {
            warn!("Failed to load game config from {}: {}. Using defaults.", config_path, e);
            
            // Fall back to default configuration
            let mut config = GameConfig::default();
            config.validate();
            commands.insert_resource(config);
        }
    }
}

/// System to hot-reload configuration during development
#[allow(dead_code)]
pub fn hot_reload_config(
    mut config: ResMut<GameConfig>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F5) {
        let config_path = "assets/config/game_config.ron";
        
        match GameConfig::load_from_file(config_path) {
            Ok(mut new_config) => {
                new_config.validate();
                *config = new_config;
                info!("Hot-reloaded game configuration");
            }
            Err(e) => {
                warn!("Failed to hot-reload config: {}", e);
            }
        }
    }
}
