use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod gameplay;
pub mod performance;
pub mod debug;

use gameplay::GameplayConfig;
use performance::PerformanceConfig;
use debug::DebugConfig;

/// Main game configuration resource
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub gameplay: GameplayConfig,
    pub performance: PerformanceConfig,
    pub debug: DebugConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            gameplay: GameplayConfig::default(),
            performance: PerformanceConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

/// Event fired when configuration is reloaded
#[derive(Event, Clone, Debug)]
pub struct ConfigReloadedEvent {
    pub config: GameConfig,
}

/// Plugin for managing game configuration
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_event::<ConfigReloadedEvent>()
            .add_systems(PreUpdate, load_config_from_assets.run_if(resource_changed::<GameConfig>()));
        
        #[cfg(debug_assertions)]
        app.add_systems(Update, hot_reload_config);
    }
}

fn load_config_from_assets(
    _asset_server: Res<AssetServer>,
    config: ResMut<GameConfig>,
    mut reload_events: EventWriter<ConfigReloadedEvent>,
) {
    // Asset-based loading will be implemented with RON files
    // For now, just emit reload event on config changes
    reload_events.send(ConfigReloadedEvent {
        config: config.clone(),
    });
}

#[cfg(debug_assertions)]
fn hot_reload_config(
    _asset_server: Res<AssetServer>,
    config: ResMut<GameConfig>,
    mut reload_events: EventWriter<ConfigReloadedEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // F9 to reload configuration
    if keyboard.just_pressed(KeyCode::F9) {
        info!("Hot-reloading configuration...");
        // Reload logic will be implemented with asset loading
        reload_events.send(ConfigReloadedEvent {
            config: config.clone(),
        });
    }
}
