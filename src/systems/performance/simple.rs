/// Simple performance plugin that replaces the complex 780-line system
/// Uses Bevy's built-in diagnostics and provides a basic F3 debug overlay
use bevy::prelude::*;

/// Simple replacement for the old performance system
pub struct SimplePerformancePlugin;

impl Plugin for SimplePerformancePlugin {
    fn build(&self, _app: &mut App) {
        // Note: DiagnosticsPlugin and FrameTimeDiagnosticsPlugin already added elsewhere
        // The performance system now relies on existing diagnostic plugins
    }
}

/// Component to mark debug text
#[derive(Component)]
pub struct DebugText;

/// Resource to track debug state
#[derive(Resource, Default)]
pub struct DebugOverlayState {
    pub visible: bool,
}

/// System to toggle debug overlay with F3
pub fn toggle_debug_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugOverlayState>,
    mut query: Query<&mut Visibility, With<DebugText>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::F3) {
        state.visible = !state.visible;

        if let Ok(mut visibility) = query.single_mut() {
            *visibility = if state.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        } else if state.visible {
            // Create debug text if it doesn't exist
            spawn_debug_text(&mut commands);
        }
    }
}

/// Spawn debug text overlay
fn spawn_debug_text(commands: &mut Commands) {
    commands.spawn((
        Text::new("Debug Info (F3)"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        DebugText,
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    ));
}

/// Update debug text with basic info
pub fn update_debug_text(
    state: Res<DebugOverlayState>,
    mut query: Query<&mut Text, With<DebugText>>,
    entities: Query<Entity>,
    time: Res<Time>,
) {
    if !state.visible {
        return;
    }

    if let Ok(mut text) = query.single_mut() {
        let entity_count = entities.iter().count();
        let fps = 1.0 / time.delta_secs();

        text.0 = format!(
            "Debug Info (F3)\n\
            FPS: {fps:.1}\n\
            Entities: {entity_count}"
        );
    }
}

/// Debug UI plugin
pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOverlayState>()
            .add_systems(Update, (toggle_debug_overlay, update_debug_text));
    }
}

// Export the plugins
pub use SimplePerformancePlugin as PerformancePlugin;
