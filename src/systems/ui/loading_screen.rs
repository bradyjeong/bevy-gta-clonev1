use bevy::prelude::*;

/// Marker component for loading screen UI
#[derive(Component)]
pub struct LoadingScreenUI;

/// Marker for the progress text element
#[derive(Component)]
pub struct LoadingProgressText;

/// Marker for the progress bar fill element
#[derive(Component)]
pub struct LoadingProgressBar;

/// Create loading screen UI on entering Loading state
pub fn setup_loading_screen(mut commands: Commands) {
    commands
        .spawn((
            LoadingScreenUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.12)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("GTA-Style World Generation"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Progress text
            parent.spawn((
                LoadingProgressText,
                Text::new("Initializing..."),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Progress bar container
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(30.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                    BorderColor(Color::srgb(0.5, 0.5, 0.6)),
                ))
                .with_children(|parent| {
                    // Progress bar fill
                    parent.spawn((
                        LoadingProgressBar,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.6, 0.9)),
                    ));
                });
        });

    info!("Loading screen UI created");
}

/// Update loading progress display in real-time
pub fn update_loading_progress(
    mut text_query: Query<&mut Text, With<LoadingProgressText>>,
    mut bar_query: Query<&mut Node, With<LoadingProgressBar>>,
    queue: Option<Res<crate::plugins::static_world_generation_plugin::StaticGenerationQueue>>,
) {
    let Some(queue) = queue else {
        return;
    };

    let progress = (queue.completed_chunks as f32 / queue.total_chunks as f32) * 100.0;
    let elapsed = queue.start_time.elapsed().as_secs_f32();
    let rate = if elapsed > 0.0 {
        queue.completed_chunks as f32 / elapsed
    } else {
        0.0
    };
    let eta = if rate > 0.0 {
        (queue.total_chunks - queue.completed_chunks) as f32 / rate
    } else {
        0.0
    };

    // Update text
    if let Ok(mut text) = text_query.single_mut() {
        **text = format!(
            "Generating World: {}/{} chunks ({:.1}%)\n{:.0} chunks/sec - ETA: {:.0}s",
            queue.completed_chunks, queue.total_chunks, progress, rate, eta
        );
    }

    // Update progress bar
    if let Ok(mut bar_node) = bar_query.single_mut() {
        bar_node.width = Val::Percent(progress);
    }
}

/// Cleanup loading screen when entering InGame state
pub fn cleanup_loading_screen(
    mut commands: Commands,
    loading_ui_query: Query<Entity, With<LoadingScreenUI>>,
) {
    for entity in &loading_ui_query {
        commands.entity(entity).despawn();
    }
    info!("Loading screen UI cleaned up");
}
