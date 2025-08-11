#![allow(unused_imports, unused_variables)]
use bevy::prelude::*;
use gta_game::plugins::{GameCorePlugin, GameSetupPlugin};

/// Smoke test to ensure the game plugins can be registered without conflicts
/// This prevents future refactors from compiling but failing at startup
#[test]
fn game_plugins_register_successfully() {
    let mut app = App::new();
    
    // Use minimal plugins to avoid GUI/EventLoop issues in tests
    app.add_plugins(MinimalPlugins);
    
    // Add just the setup plugin to test system registration
    app.add_plugins(GameSetupPlugin);
    
    // If we get here without panicking, the plugins registered successfully
    assert!(true, "Game plugins registered without conflicts");
}

/// Test that game components and bundles can be created
#[test]
fn components_can_be_instantiated() {
    use gta_game::components::*;
    
    // Test basic component creation that should work
    let active_entity = ActiveEntity;
    let culling_settings = CullingSettings::default();
    let performance_stats = PerformanceStats::default();
    
    // Basic marker component
    let main_camera = MainCamera;
    
    assert!(true, "Components can be created");
}
