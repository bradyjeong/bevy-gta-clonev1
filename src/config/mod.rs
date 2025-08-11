use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod gameplay;
pub mod performance;
pub mod debug;

use gameplay::GameplayConfig;
use debug::DebugConfig;

// Re-export for compatibility
pub use gameplay::{
    PhysicsGameplay as PhysicsConfig,
    WorldGameplay as WorldConfig,
    VehicleGameplay as VehicleConfig,
    NPCGameplay as NPCConfig,
    CameraGameplay as CameraConfig,
};

// Re-export AudioConfig from debug module
pub use debug::AudioDebug as AudioConfig;

// Re-export PerformanceConfig
pub use performance::PerformanceConfig;

// UIConfig compatibility alias
pub use debug::OverlayConfig as UIConfig;

/// Main game configuration resource
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub gameplay: GameplayConfig,
    pub performance: performance::PerformanceConfig,
    pub debug: DebugConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            gameplay: GameplayConfig::default(),
            performance: performance::PerformanceConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

impl GameConfig {
    /// Validate and clamp configuration values to safe ranges
    pub fn validate_and_clamp(&mut self) {
        // Clamp physics values
        self.gameplay.physics.min_mass = self.gameplay.physics.min_mass.max(0.1);
        self.gameplay.physics.max_mass = self.gameplay.physics.max_mass.min(100000.0);
        self.gameplay.physics.max_velocity = self.gameplay.physics.max_velocity.clamp(1.0, 10000.0);
        self.gameplay.physics.min_collider_size = self.gameplay.physics.min_collider_size.max(0.01);
        self.gameplay.physics.max_collider_size = self.gameplay.physics.max_collider_size.min(1000.0);
        
        // Clamp vehicle values
        self.gameplay.vehicle.car_top_speed = self.gameplay.vehicle.car_top_speed.clamp(10.0, 300.0);
        self.gameplay.vehicle.supercar_top_speed = self.gameplay.vehicle.supercar_top_speed.clamp(50.0, 500.0);
        self.gameplay.vehicle.f16_max_speed = self.gameplay.vehicle.f16_max_speed.clamp(100.0, 2000.0);
        
        // Clamp camera values
        self.gameplay.camera.distance = self.gameplay.camera.distance.clamp(1.0, 100.0);
        self.gameplay.camera.height = self.gameplay.camera.height.clamp(0.5, 50.0);
        self.gameplay.camera.smoothing = self.gameplay.camera.smoothing.clamp(0.0, 1.0);
        
        // Clamp world values
        self.gameplay.world.gravity = self.gameplay.world.gravity.clamp(0.1, 50.0);
        self.gameplay.world.time_scale = self.gameplay.world.time_scale.clamp(0.01, 10.0);
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
            .add_systems(PreUpdate, load_config_from_assets.run_if(resource_changed::<GameConfig>));
        
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
    reload_events.write(ConfigReloadedEvent {
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
        reload_events.write(ConfigReloadedEvent {
            config: config.clone(),
        });
    }
}
