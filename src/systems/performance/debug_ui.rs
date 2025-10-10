use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use super::metrics::{ENTITY_COUNT, CHUNK_COUNT, ACTIVE_CHUNKS};

/// Component to mark the debug overlay
#[derive(Component)]
pub struct DebugOverlay;

/// Resource to track debug state
#[derive(Resource, Default)]
pub struct DebugState {
    pub show_debug: bool,
}

/// System to handle F3 debug key toggle
pub fn debug_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugState>,
    mut overlay_query: Query<&mut Visibility, With<DebugOverlay>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::F3) {
        debug_state.show_debug = !debug_state.show_debug;
        
        if let Ok(mut visibility) = overlay_query.single_mut() {
            *visibility = if debug_state.show_debug {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        } else if debug_state.show_debug {
            // Create overlay if it doesn't exist
            spawn_debug_overlay(&mut commands);
        }
    }
}

/// Spawn the debug overlay UI
fn spawn_debug_overlay(commands: &mut Commands) {
    commands.spawn((
        Text::new("Performance Debug (F3)"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // Yellow text
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        DebugOverlay,
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Semi-transparent background
    ));
}

/// System to update the debug overlay with performance metrics
pub fn update_debug_overlay_system(
    diagnostics: Res<Diagnostics>,
    debug_state: Res<DebugState>,
    mut overlay_query: Query<&mut Text, With<DebugOverlay>>,
    car_query: Query<(&crate::components::ControlState, &bevy_rapier3d::prelude::Velocity, &crate::components::SimpleCarSpecs), With<crate::components::Car>>,
) {
    if !debug_state.show_debug {
        return;
    }

    if let Ok(mut text) = overlay_query.single_mut() {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.smoothed())
            .unwrap_or(0.0);

        let frame_time = if fps > 0.0 { 1000.0 / fps } else { 0.0 };

        let entities = diagnostics
            .get(ENTITY_COUNT)
            .and_then(|d| d.value())
            .unwrap_or(0.0) as u32;

        let chunks = diagnostics
            .get(CHUNK_COUNT)
            .and_then(|d| d.value())
            .unwrap_or(0.0) as u32;

        let active_chunks = diagnostics
            .get(ACTIVE_CHUNKS)
            .and_then(|d| d.value())
            .unwrap_or(0.0) as u32;

        // Get car physics info if player is in a car
        let car_info = if let Ok((control_state, velocity, specs)) = car_query.get_single() {
            let speed = velocity.linvel.length();
            format!(
                "\n\nVehicle:\n\
                Speed: {:.1} m/s ({:.0} km/h)\n\
                Steering: {:.2} | E-Brake: {}\n\
                Grip: {:.1} | Drift Grip: {:.1}\n\
                Stability: {:.1}",
                speed,
                speed * 3.6,
                control_state.steering,
                if control_state.emergency_brake { "ON" } else { "OFF" },
                specs.grip,
                specs.drift_grip,
                specs.stability
            )
        } else {
            String::new()
        };

        text.0 = format!(
            "Performance Debug (F3)\n\
            FPS: {:.1} | Frame: {:.2}ms\n\
            Entities: {}\n\
            Chunks: {} (Active: {}){}",
            fps, frame_time, entities, chunks, active_chunks, car_info
        );
    }
}

/// Plugin for debug UI
pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>()
            .add_systems(Update, (debug_toggle_system, update_debug_overlay_system));
    }
}
