use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;

#[derive(Component)]
pub struct SplashScreen;

#[derive(Component)]
pub struct LoadingBar;

#[derive(Component)]
pub struct LoadingText;

#[derive(Resource)]
pub struct AssetLoadingState {
    pub handles: Vec<UntypedHandle>,
    pub total_assets: usize,
    pub min_display_timer: Timer,
}

impl Default for AssetLoadingState {
    fn default() -> Self {
        Self {
            handles: Vec::new(),
            total_assets: 0,
            min_display_timer: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

pub fn setup_splash_screen(mut commands: Commands) {
    commands
        .spawn((
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
        ))
        .with_children(|parent| {
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
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(4.0),
                        border: UiRect::all(Val::Px(1.0)),
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|progress_parent| {
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
                LoadingText,
                Text::new("Loading assets..."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

pub fn load_initial_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = vec![asset_server.load::<Image>("ui/arrow.png").untyped()];

    let total = handles.len();
    commands.insert_resource(AssetLoadingState {
        handles,
        total_assets: total,
        min_display_timer: Timer::from_seconds(2.0, TimerMode::Once),
    });
}

pub fn update_asset_loading(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut loading_state: ResMut<AssetLoadingState>,
    mut loading_bar_query: Query<&mut Node, With<LoadingBar>>,
    mut loading_text_query: Query<&mut Text, With<LoadingText>>,
    mut next_state: ResMut<NextState<crate::states::AppState>>,
) {
    loading_state.min_display_timer.tick(time.delta());

    let mut loaded = 0;
    for handle in &loading_state.handles {
        match asset_server.get_recursive_dependency_load_state(handle.id()) {
            Some(RecursiveDependencyLoadState::Loaded) => loaded += 1,
            Some(RecursiveDependencyLoadState::Failed(_)) => loaded += 1,
            _ => {}
        }
    }

    let progress = if loading_state.total_assets > 0 {
        loaded as f32 / loading_state.total_assets as f32
    } else {
        0.0
    };

    for mut node in loading_bar_query.iter_mut() {
        node.width = Val::Px(300.0 * progress);
    }

    for mut text in loading_text_query.iter_mut() {
        if loaded >= loading_state.total_assets {
            **text = "Ready!".to_string();
        } else {
            **text = format!(
                "Loading assets... {}/{}",
                loaded, loading_state.total_assets
            );
        }
    }

    if loaded >= loading_state.total_assets && loading_state.min_display_timer.finished() {
        next_state.set(crate::states::AppState::WorldGeneration);
    }
}

pub fn cleanup_splash_screen(
    mut commands: Commands,
    splash_query: Query<Entity, With<SplashScreen>>,
) {
    for entity in splash_query.iter() {
        commands.entity(entity).despawn();
    }
}
