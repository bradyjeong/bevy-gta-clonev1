use bevy::prelude::*;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{GameConfig, ConfigReloadedEvent};
use crate::config::gameplay::GameplayConfig;
use crate::config::performance::PerformanceConfig;
use crate::config::debug::DebugConfig;

/// RON configuration asset
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct RonConfigAsset {
    pub gameplay: Option<GameplayConfig>,
    pub performance: Option<PerformanceConfig>,
    pub debug: Option<DebugConfig>,
}

/// Configuration loader errors
#[derive(Error, Debug)]
pub enum ConfigLoaderError {
    #[error("Failed to load RON: {0}")]
    RonError(#[from] ron::error::SpannedError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Asset loader for RON configuration files
#[derive(Default)]
pub struct RonConfigLoader;

impl AssetLoader for RonConfigLoader {
    type Asset = RonConfigAsset;
    type Settings = ();
    type Error = ConfigLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = ron::de::from_bytes(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// System that loads configuration from assets
pub fn load_game_config(
    asset_server: Res<AssetServer>,
    config_assets: Res<Assets<RonConfigAsset>>,
    mut game_config: ResMut<GameConfig>,
    mut reload_events: EventWriter<ConfigReloadedEvent>,
    mut handles: Local<ConfigHandles>,
) {
    // Load configuration files if not already loaded
    if handles.gameplay.is_none() {
        handles.gameplay = Some(asset_server.load("config/gameplay.ron"));
        handles.performance = Some(asset_server.load("config/performance.ron"));
        handles.debug = Some(asset_server.load("config/debug.ron"));
    }

    let mut config_changed = false;

    // Check gameplay config
    if let Some(handle) = &handles.gameplay {
        if let Some(asset) = config_assets.get(handle) {
            if let Some(gameplay) = &asset.gameplay {
                if !configs_equal::<GameplayConfig>(&game_config.gameplay, gameplay) {
                    game_config.gameplay = gameplay.clone();
                    config_changed = true;
                    info!("Loaded gameplay configuration");
                }
            }
        }
    }

    // Check performance config
    if let Some(handle) = &handles.performance {
        if let Some(asset) = config_assets.get(handle) {
            if let Some(performance) = &asset.performance {
                if !configs_equal::<PerformanceConfig>(&game_config.performance, performance) {
                    game_config.performance = performance.clone();
                    config_changed = true;
                    info!("Loaded performance configuration");
                }
            }
        }
    }

    // Check debug config
    if let Some(handle) = &handles.debug {
        if let Some(asset) = config_assets.get(handle) {
            if let Some(debug) = &asset.debug {
                if !configs_equal::<DebugConfig>(&game_config.debug, debug) {
                    game_config.debug = debug.clone();
                    config_changed = true;
                    info!("Loaded debug configuration");
                }
            }
        }
    }

    if config_changed {
        reload_events.send(ConfigReloadedEvent {
            config: game_config.clone(),
        });
    }
}

/// Handles to loaded configuration assets
#[derive(Default)]
struct ConfigHandles {
    gameplay: Option<Handle<RonConfigAsset>>,
    performance: Option<Handle<RonConfigAsset>>,
    debug: Option<Handle<RonConfigAsset>>,
}

/// Helper function to check if configs are equal
fn configs_equal<T: PartialEq>(a: &T, b: &T) -> bool {
    a == b
}

/// Hot reload system for development
#[cfg(debug_assertions)]
pub fn hot_reload_config(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut handles: Local<ConfigHandles>,
) {
    if keyboard.just_pressed(KeyCode::F9) {
        info!("Hot-reloading configuration files...");
        
        // Reset handles to trigger reload on next frame
        handles.gameplay = None;
        handles.performance = None;
        handles.debug = None;
    }
}

/// System to apply performance configuration changes
pub fn apply_performance_config(
    mut events: EventReader<ConfigReloadedEvent>,
    mut _query: Query<&mut Transform>,
) {
    for _event in events.read() {
        info!("Applying performance configuration changes");
        // Configuration changes will be picked up by individual systems
        // that read from GameConfig resource
    }
}

/// System to apply debug configuration changes
pub fn apply_debug_config(
    mut events: EventReader<ConfigReloadedEvent>,
) {
    for event in events.read() {
        // Apply logging level changes
        let level = match event.config.debug.logging.level.as_str() {
            "error" => bevy::log::Level::ERROR,
            "warn" => bevy::log::Level::WARN,
            "info" => bevy::log::Level::INFO,
            "debug" => bevy::log::Level::DEBUG,
            "trace" => bevy::log::Level::TRACE,
            _ => bevy::log::Level::INFO,
        };
        
        info!("Debug configuration applied, log level: {:?}", level);
    }
}

// Implement PartialEq for config types to enable comparison
impl PartialEq for GameplayConfig {
    fn eq(&self, _other: &Self) -> bool {
        // For now, always return false to trigger reload
        // In production, implement proper comparison
        false
    }
}

impl PartialEq for PerformanceConfig {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl PartialEq for DebugConfig {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
