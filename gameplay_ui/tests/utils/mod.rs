//! Test utilities for UI tests

use bevy::prelude::*;
use game_core::prelude::*;
use gameplay_ui::prelude::*;

/// Create a basic UI test app
pub fn create_ui_test_app() -> App {
    let mut app = App::new();
    
    // Add minimal plugins for testing
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::ui::UiPlugin::default(),
        bevy::text::TextPlugin::default(),
        bevy::input::InputPlugin::default(),
        bevy::time::TimePlugin::default(),
        bevy::diagnostic::DiagnosticsPlugin::default(),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
    ));
    
    // Add UI plugin
    app.add_plugins(UiPlugin);
    
    app
}

/// Set up a test scene with basic entities
pub fn setup_ui_test_scene(app: &mut App) -> (Entity, Entity) {
    let world = app.world_mut();
    
    // Create a basic supercar entity
    let supercar = world.spawn((
        SuperCar::default(),
        Car,
        ActiveEntity,
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Create a basic NPC entity
    let npc = world.spawn((
        NPC {
            target_position: Vec3::new(10.0, 0.0, 0.0),
            speed: 5.0,
            last_update: 0.0,
            update_interval: 0.1,
            health: Some(100.0),
            max_health: Some(100.0),
            behavior_state: Some(NPCBehaviorState::Idle),
            spawn_time: Some(0.0),
        },
        Transform::from_xyz(10.0, 0.0, 0.0),
    )).id();
    
    (supercar, npc)
}
