use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(Component)]
pub struct FpsText;

pub fn setup_fps_display(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            // Set fixed width to prevent position shift when digits change
            width: Val::Px(100.0),
            ..default()
        },
        FpsText,
    ));
}

pub fn update_fps_display(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text_query: Query<&mut Text, With<FpsText>>,
) {
    if let Ok(mut text) = fps_text_query.single_mut()
        && let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        && let Some(fps) = fps_diag.smoothed()
    {
        text.0 = format!("FPS: {fps:.0}");
    }
}
