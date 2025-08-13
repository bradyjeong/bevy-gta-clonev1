use bevy::prelude::*;
use crate::components::*;

/// Setup initial weather components for existing world objects
pub fn setup_weather_components(
    mut commands: Commands,
    terrain_query: Query<Entity, With<DynamicTerrain>>,
    palm_trees: Query<(Entity, &Transform), (With<Cullable>, Without<Player>, Without<Car>)>,
    vehicles: Query<Entity, (With<Car>, Without<Player>)>,
) {
    // Add rain collectors to terrain
    for entity in terrain_query.iter() {
        commands.entity(entity).insert(RainCollector::default());
    }
    
    // Make palm trees wind-responsive
    for (entity, transform) in palm_trees.iter() {
        // Only add to entities that look like trees (have certain height characteristics)
        if transform.translation.y > 3.0 { // Likely a tree if positioned above ground
            commands.entity(entity).insert((
                WindResponsive {
                    sway_amount: 0.15,
                    sway_frequency: 0.8,
                    base_transform: *transform,
                    current_sway: 0.0,
                },
                WeatherAffected::default(),
            ));
        }
    }
    
    // Add weather effects to vehicles
    for entity in vehicles.iter() {
        commands.entity(entity).insert(WeatherAffected {
            visibility_modifier: 1.0,
            wind_resistance: 0.8, // Vehicles are somewhat resistant to wind
        });
    }
}

/// Setup weather-responsive materials and effects
pub fn setup_weather_materials(
    mut commands: Commands,
    _materials: ResMut<Assets<StandardMaterial>>,
    building_query: Query<Entity, With<Building>>,
) {
    // Create weather-responsive materials for buildings
    for entity in building_query.iter() {
        // Add rain collector to building surfaces
        commands.entity(entity).insert((
            RainCollector {
                wetness: 0.0,
                puddle_depth: 0.0,
                dry_rate: 0.05, // Buildings dry slower than terrain
            },
            WeatherAffected::default(),
        ));
        
        // In a full implementation, you might want to:
        // - Create wet/dry material variants
        // - Add reflective puddle materials
        // - Implement dynamic material switching based on wetness
    }
}

/// Create initial weather UI elements
pub fn setup_weather_ui(
    mut commands: Commands,
) {
    // Weather indicator UI
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                right: Val::Px(20.0),
                width: Val::Px(200.0),
                height: Val::Auto,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            BorderRadius::all(Val::Px(4.0)),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Weather: Clear"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 1.0)),
                WeatherDisplay,
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ));
        });
    
    // Debug weather controls info (only when debug-ui feature is enabled)
    #[cfg(feature = "debug-ui")]
    {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Px(20.0),
                    width: Val::Px(300.0),
                    height: Val::Auto,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                BorderRadius::all(Val::Px(4.0)),
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Debug Weather Controls:\n1-Clear 2-Light Rain 3-Heavy Rain\n4-Storm 5-Fog 6-Sandstorm"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    Visibility::Visible,
                    InheritedVisibility::VISIBLE,
                    ViewVisibility::default(),
                ));
            });
    }
}

/// Component for weather display UI
#[derive(Component)]
pub struct WeatherDisplay;

/// System to update weather display UI
pub fn update_weather_ui(
    weather: Res<WeatherManager>,
    mut weather_text: Query<&mut Text, With<WeatherDisplay>>,
) {
    if !weather.is_changed() {
        return;
    }
    
    for mut text in weather_text.iter_mut() {
        let weather_name = match weather.current_weather {
            WeatherType::Clear => "Clear",
            WeatherType::LightRain => "Light Rain",
            WeatherType::HeavyRain => "Heavy Rain",
            WeatherType::Storm => "Storm",
            WeatherType::Fog => "Fog",
            WeatherType::Sandstorm => "Sandstorm",
        };
        
        let status = if let Some(ref target) = weather.target_weather {
            let target_name = match target {
                WeatherType::Clear => "Clear",
                WeatherType::LightRain => "Light Rain", 
                WeatherType::HeavyRain => "Heavy Rain",
                WeatherType::Storm => "Storm",
                WeatherType::Fog => "Fog",
                WeatherType::Sandstorm => "Sandstorm",
            };
            format!("Weather: {} → {} ({:.0}%)", weather_name, target_name, weather.transition_timer / weather.transition_duration * 100.0)
        } else {
            format!("Weather: {} ({:.0}%)", weather_name, weather.intensity * 100.0)
        };
        
        text.0 = status;
    }
}

/// Setup ambient weather environment
pub fn setup_weather_environment(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create invisible wind zones for better wind simulation
    for i in 0..4 {
        let angle = (i as f32) * std::f32::consts::TAU / 4.0;
        let distance = 200.0;
        let position = Vec3::new(
            angle.cos() * distance,
            50.0,
            angle.sin() * distance,
        );
        
        commands.spawn((
            Transform::from_translation(position),
            WeatherAffected {
                visibility_modifier: 1.0,
                wind_resistance: 0.1, // Very susceptible to wind
            },
            Visibility::Hidden, // Invisible wind zone
        ));
    }
}
