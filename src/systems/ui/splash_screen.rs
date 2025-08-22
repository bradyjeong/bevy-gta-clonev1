use bevy::prelude::*;

#[derive(Component)]
pub struct SplashScreen;

#[derive(Component)]
pub struct LoadingBar;

#[derive(Resource)]
pub struct SplashScreenState {
    pub timer: Timer,
}

impl Default for SplashScreenState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

pub fn setup_splash_screen(mut commands: Commands) {
    commands.spawn((
        SplashScreen,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
        ZIndex(1000),
    )).with_children(|parent| {
        // Main title with modern styling
        parent.spawn((
            Text::new("VICE CITY"),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.85, 0.15)),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));
        
        // Loading container
        parent.spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        )).with_children(|progress_parent| {
            // Animated progress bar
            progress_parent.spawn((
                LoadingBar,
                Node {
                    width: Val::Px(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.85, 0.15)),
            ));
        });
        
        parent.spawn((
            Text::new("Loading world..."),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}

pub fn update_splash_screen(
    time: Res<Time>,
    mut splash_state: ResMut<SplashScreenState>,
    mut commands: Commands,
    splash_query: Query<Entity, With<SplashScreen>>,
    mut loading_bar_query: Query<&mut Node, With<LoadingBar>>,
) {
    splash_state.timer.tick(time.delta());

    // Animate progress bar
    let progress = splash_state.timer.elapsed_secs() / splash_state.timer.duration().as_secs_f32();
    let progress = progress.clamp(0.0, 1.0);
    
    for mut node in loading_bar_query.iter_mut() {
        node.width = Val::Px(300.0 * progress);
    }

    if splash_state.timer.finished() {
        for entity in splash_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
