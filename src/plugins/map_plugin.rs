use crate::components::{ActiveEntity, MapCamera, MapConfig, MinimapUI, PlayerMapIcon};
use bevy::math::{EulerRot, FloatOrd};
use bevy::prelude::*;
use bevy::render::camera::{ImageRenderTarget, RenderTarget};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

pub struct MapPlugin;

#[derive(Resource)]
struct MapSetupTimer(Timer);

impl Default for MapSetupTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3.0, TimerMode::Once))
    }
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapSetupTimer>()
            .add_systems(Startup, load_map_config)
            .add_systems(Update, delayed_setup_minimap)
            .add_systems(Update, update_map_camera)
            .add_systems(Update, update_player_icon);
    }
}

fn load_map_config(mut commands: Commands) {
    let config: MapConfig =
        ron::from_str(&std::fs::read_to_string("assets/config/map.ron").expect(
            "Failed to read map config at 'assets/config/map.ron'.\n\
             Troubleshooting:\n\
             1. Verify file exists in assets/config/ directory\n\
             2. Check file permissions (should be readable)\n\
             3. If in release build, verify assets/ is copied to executable location",
        ))
        .expect(
            "Failed to parse RON config at 'assets/config/map.ron'.\n\
         Common RON syntax issues:\n\
         1. Missing comma between fields\n\
         2. Typo in field name (must match MapConfig struct)\n\
         3. Wrong value type (e.g., string instead of number)\n\
         4. Missing parentheses or brackets\n\
         See https://github.com/ron-rs/ron for syntax guide.\n\
         Run 'cargo run --features debug-ui' for detailed validation.",
        );

    commands.insert_resource(config);
}

fn delayed_setup_minimap(
    time: Res<Time>,
    mut timer: ResMut<MapSetupTimer>,
    commands: Commands,
    images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    config: Res<MapConfig>,
    minimap_query: Query<Entity, With<MinimapUI>>,
) {
    if timer.0.tick(time.delta()).just_finished() && minimap_query.is_empty() {
        setup_minimap(commands, images, asset_server, config);
    }
}

fn setup_minimap(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    config: Res<MapConfig>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let image_handle = images.add(image);

    commands.spawn((
        MapCamera,
        Camera3d::default(),
        Camera {
            order: -1,
            target: RenderTarget::Image(ImageRenderTarget {
                handle: image_handle.clone(),
                scale_factor: FloatOrd(1.0),
            }),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::FixedVertical {
                viewport_height: config.map_size,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, config.map_height, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    let ui_pos = config.ui_position;
    let ui_size = config.ui_size;

    commands
        .spawn((
            MinimapUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(ui_pos.1),
                right: Val::Px(ui_pos.0),
                width: Val::Px(ui_size.0),
                height: Val::Px(ui_size.1),
                border: UiRect::all(Val::Px(config.border_width)),
                ..default()
            },
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, config.background_alpha)),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: image_handle.clone(),
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));

            if config.show_player_icon {
                let arrow_image = asset_server.load("ui/arrow.png");
                let half_size = config.player_icon_size / 2.0;
                parent.spawn((
                    PlayerMapIcon,
                    ImageNode {
                        image: arrow_image,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(config.player_icon_size),
                        height: Val::Px(config.player_icon_size),
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-half_size),
                            top: Val::Px(-half_size),
                            ..default()
                        },
                        ..default()
                    },
                    Transform::default(),
                    GlobalTransform::default(),
                ));
            }

            // Cardinal direction labels - match actual minimap orientation
            // North at top
            parent.spawn((
                Text::new("N"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(5.0),
                    left: Val::Percent(50.0),
                    margin: UiRect {
                        left: Val::Px(-7.0),
                        ..default()
                    },
                    ..default()
                },
            ));

            // West on right (camera orientation)
            parent.spawn((
                Text::new("W"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(50.0),
                    right: Val::Px(5.0),
                    margin: UiRect {
                        top: Val::Px(-7.0),
                        ..default()
                    },
                    ..default()
                },
            ));

            // South (bottom)
            parent.spawn((
                Text::new("S"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(5.0),
                    left: Val::Percent(50.0),
                    margin: UiRect {
                        left: Val::Px(-7.0),
                        ..default()
                    },
                    ..default()
                },
            ));

            // East on left (camera orientation)
            parent.spawn((
                Text::new("E"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(50.0),
                    left: Val::Px(5.0),
                    margin: UiRect {
                        top: Val::Px(-7.0),
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

fn update_map_camera(
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut camera_query: Query<&mut Transform, (With<MapCamera>, Without<ActiveEntity>)>,
    config: Res<MapConfig>,
) {
    if let (Ok(active_transform), Ok(mut camera_transform)) =
        (active_query.single(), camera_query.single_mut())
    {
        let target_pos = active_transform.translation;
        camera_transform.translation.x = target_pos.x;
        camera_transform.translation.z = target_pos.z;
        camera_transform.translation.y = config.map_height;
    }
}

fn update_player_icon(
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut icon_query: Query<&mut Transform, (With<PlayerMapIcon>, Without<ActiveEntity>)>,
) {
    if let (Ok(active_transform), Ok(mut icon_transform)) =
        (active_query.single(), icon_query.single_mut())
    {
        let (yaw, _pitch, _roll) = active_transform.rotation.to_euler(EulerRot::YXZ);
        icon_transform.rotation = Quat::from_rotation_z(-yaw + std::f32::consts::PI);
    }
}
