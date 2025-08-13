use bevy::prelude::*;
use crate::components::world::*;
#[cfg(feature = "weather")]
use crate::components::weather::*;
use crate::components::Player;

/// Plugin for sky rendering and time management
pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TimeOfDay>()
            .add_systems(Startup, setup_time_ui)
            .add_systems(Update, (
                time_progression_system,
                time_control_system,
                update_time_ui,
                sky_dome_system.run_if(resource_changed::<TimeOfDay>),
                sun_system,
                celestial_bodies_system,
                cloud_system,
            ).chain());
    }
}

/// Resource for managing time of day
#[derive(Resource)]
pub struct TimeOfDay {
    /// Current time in hours (0.0 = midnight, 12.0 = noon, 24.0 = midnight)
    pub current_time: f32,
    /// Speed multiplier for time progression
    pub time_speed: f32,
    /// Time direction (1.0 = forward, -1.0 = backward)
    pub time_direction: f32,
    /// Whether time progression is enabled
    pub enabled: bool,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self {
            current_time: 12.0, // Start at noon
            time_speed: 1.0,    // 1 hour per minute by default
            time_direction: 1.0,
            enabled: false,     // Time progression disabled by default
        }
    }
}

impl TimeOfDay {
    pub fn is_day(&self) -> bool {
        self.current_time >= 6.0 && self.current_time <= 18.0
    }
    
    pub fn is_night(&self) -> bool {
        !self.is_day()
    }
    
    pub fn get_day_night_factor(&self) -> f32 {
        // Returns 0.0 for night, 1.0 for day, with smooth transitions
        if self.current_time >= 6.0 && self.current_time <= 18.0 {
            // Day time
            if self.current_time <= 7.0 {
                // Sunrise transition
                (self.current_time - 6.0) / 1.0
            } else if self.current_time >= 17.0 {
                // Sunset transition
                1.0 - ((self.current_time - 17.0) / 1.0)
            } else {
                // Full day
                1.0
            }
        } else {
            // Night time
            0.0
        }
    }
    
    pub fn get_sun_angle(&self) -> f32 {
        // Convert time to sun angle in radians
        // 6 AM = -π/2 (sunrise), 12 PM = 0 (noon), 6 PM = π/2 (sunset)
        let normalized_time = (self.current_time - 6.0) / 12.0; // 0 to 1 for 6AM to 6PM
        (normalized_time - 0.5) * std::f32::consts::PI
    }
}

/// Component for sky dome
#[derive(Component)]
pub struct SkyDomeRenderer {
    pub gradient_colors: [Color; 4], // Bottom, horizon, zenith, space
}

/// Component for celestial bodies
#[derive(Component)]
pub struct CelestialBody {
    pub body_type: CelestialBodyType,
    pub orbital_radius: f32,
    pub size: f32,
}

#[derive(Clone)]
pub enum CelestialBodyType {
    Sun,
    Moon,
    Star { brightness: f32 },
}

/// Component for animated clouds
#[derive(Component)]
pub struct CloudLayer {
    pub altitude: f32,
    pub speed: Vec3,
    pub opacity: f32,
    pub scale: f32,
}

/// System to update time of day
pub fn time_progression_system(
    mut time_of_day: ResMut<TimeOfDay>,
    time: Res<Time>,
) {
    // Only update time if enabled
    if time_of_day.enabled {
        time_of_day.current_time += time.delta_secs() * time_of_day.time_speed * time_of_day.time_direction / 60.0;
        
        // Wrap around 24 hours
        if time_of_day.current_time >= 24.0 {
            time_of_day.current_time -= 24.0;
        } else if time_of_day.current_time < 0.0 {
            time_of_day.current_time += 24.0;
        }
    }
}

/// System to create and manage the sky dome
#[cfg(feature = "weather")]
pub fn sky_dome_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: Res<TimeOfDay>,
    weather: Res<WeatherManager>,
    existing_sky: Query<Entity, With<SkyDome>>,
) {
    // Remove existing sky dome if it exists
    for entity in existing_sky.iter() {
        commands.entity(entity).despawn();
    }
    
    let _day_factor = time_of_day.get_day_night_factor();
    
    // Sky colors based on time of day and weather
    let (bottom_color, horizon_color, zenith_color, space_color) = match weather.current_weather {
        WeatherType::Clear => {
            if time_of_day.is_day() {
                (
                    Color::srgb(0.7, 0.8, 0.9), // Light blue bottom
                    Color::srgb(0.6, 0.7, 0.9), // Horizon blue
                    Color::srgb(0.4, 0.6, 0.9), // Deep sky blue
                    Color::srgb(0.2, 0.3, 0.6), // Space blue
                )
            } else {
                (
                    Color::srgb(0.05, 0.05, 0.15), // Dark bottom
                    Color::srgb(0.1, 0.1, 0.2),   // Dark horizon
                    Color::srgb(0.02, 0.02, 0.1), // Very dark zenith
                    Color::srgb(0.01, 0.01, 0.05), // Space black
                )
            }
        },
        WeatherType::Storm => (
            Color::srgb(0.3, 0.3, 0.35), // Gray bottom
            Color::srgb(0.25, 0.25, 0.3), // Dark gray horizon
            Color::srgb(0.2, 0.2, 0.25),  // Stormy zenith
            Color::srgb(0.15, 0.15, 0.2), // Dark space
        ),
        WeatherType::Fog => (
            Color::srgb(0.7, 0.7, 0.75), // Foggy white bottom
            Color::srgb(0.65, 0.65, 0.7), // Misty horizon
            Color::srgb(0.6, 0.6, 0.65),  // Hazy zenith
            Color::srgb(0.5, 0.5, 0.55),  // Obscured space
        ),
        _ => (
            Color::srgb(0.5, 0.5, 0.6),
            Color::srgb(0.4, 0.4, 0.5),
            Color::srgb(0.3, 0.3, 0.4),
            Color::srgb(0.2, 0.2, 0.3),
        ),
    };
    
    // Blend colors based on day/night factor
    let blended_colors = [
        bottom_color,
        horizon_color,
        zenith_color,
        space_color,
    ];
    
    // Create sky dome mesh (large inverted sphere)
    commands.spawn((
        SkyDome,
        SkyDomeRenderer {
            gradient_colors: blended_colors,
        },
        Mesh3d(meshes.add(Sphere::new(2000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: zenith_color,
            unlit: true,
            cull_mode: Some(bevy::render::render_resource::Face::Front), // Render inside
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// System to create and manage the sky dome (no weather version)
#[cfg(not(feature = "weather"))]
pub fn sky_dome_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: Res<TimeOfDay>,
    existing_sky: Query<Entity, With<SkyDome>>,
) {
    // Remove existing sky dome if it exists
    for entity in existing_sky.iter() {
        commands.entity(entity).despawn();
    }
    
    let _day_factor = time_of_day.get_day_night_factor();
    
    // Sky colors based on time of day only (no weather)
    let (bottom_color, horizon_color, zenith_color, space_color) = if time_of_day.is_day() {
        (
            Color::srgb(0.7, 0.8, 0.9), // Light blue bottom
            Color::srgb(0.6, 0.7, 0.9), // Horizon blue
            Color::srgb(0.4, 0.6, 0.9), // Deep sky blue
            Color::srgb(0.2, 0.3, 0.6), // Space blue
        )
    } else {
        (
            Color::srgb(0.05, 0.05, 0.15), // Dark bottom
            Color::srgb(0.1, 0.1, 0.2),   // Dark horizon
            Color::srgb(0.02, 0.02, 0.1), // Very dark zenith
            Color::srgb(0.01, 0.01, 0.05), // Space black
        )
    };

    // Create sky dome (sphere rendered from inside)
    commands.spawn((
        SkyDome,
        Mesh3d(meshes.add(Sphere::new(2000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: zenith_color,
            unlit: true,
            cull_mode: Some(bevy::render::render_resource::Face::Front), // Render inside
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// System to manage the sun
#[cfg(feature = "weather")]
pub fn sun_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: Res<TimeOfDay>,
    weather: Res<WeatherManager>,
    mut sun_query: Query<(Entity, &mut Transform, &mut DirectionalLight), With<SunLight>>,
    existing_suns: Query<Entity, (With<CelestialBody>, Without<SunLight>)>,
) {
    // Remove existing visual sun
    for entity in existing_suns.iter() {
        commands.entity(entity).despawn();
    }
    
    let sun_angle = time_of_day.get_sun_angle();
    let day_factor = time_of_day.get_day_night_factor();
    
    // Update directional light (sun lighting)
    for (_, mut transform, mut light) in sun_query.iter_mut() {
        // Position sun based on time
        let sun_direction = Vec3::new(
            sun_angle.cos(),
            sun_angle.sin(),
            0.0,
        ).normalize();
        
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, sun_direction);
        
        // Adjust light intensity and color based on time and weather
        let base_intensity = if time_of_day.is_day() {
            15000.0 * day_factor
        } else {
            100.0 // Minimal moonlight
        };
        
        light.illuminance = base_intensity * weather.get_light_modifier();
        
        // Sun color changes throughout day
        light.color = if time_of_day.is_day() {
            if day_factor < 0.3 { // Sunrise/sunset
                Color::srgb(1.0, 0.6, 0.3) // Orange
            } else {
                Color::srgb(1.0, 0.95, 0.8) // Warm white
            }
        } else {
            Color::srgb(0.7, 0.8, 1.0) // Cool moonlight
        };
    }
    
    // Create visual sun if it's day
    if time_of_day.is_day() && day_factor > 0.1 {
        let sun_distance = 1500.0;
        let sun_position = Vec3::new(
            sun_angle.cos() * sun_distance,
            sun_angle.sin() * sun_distance,
            0.0,
        );
        
        commands.spawn((
            CelestialBody {
                body_type: CelestialBodyType::Sun,
                orbital_radius: sun_distance,
                size: 50.0,
            },
            Mesh3d(meshes.add(Sphere::new(50.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.9, 0.7),
                emissive: LinearRgba::rgb(10.0, 8.0, 5.0),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(sun_position),
        ));
    }
}

/// System to manage the sun (no weather version)
#[cfg(not(feature = "weather"))]
pub fn sun_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: Res<TimeOfDay>,
    mut sun_query: Query<(Entity, &mut Transform, &mut DirectionalLight), With<SunLight>>,
    existing_suns: Query<Entity, (With<CelestialBody>, Without<SunLight>)>,
) {
    if time_of_day.is_changed() {
        // Calculate sun position based on time of day
        let day_factor = time_of_day.get_day_night_factor();
        let time_radians = time_of_day.current_time * std::f32::consts::PI * 2.0;
        
        let sun_angle = time_radians - std::f32::consts::PI / 2.0; // Offset so sun is at zenith at noon
        
        let sun_position = Vec3::new(
            1000.0 * sun_angle.cos(),
            1000.0 * sun_angle.sin().max(0.0), // Don't go below horizon
            100.0,
        );
        
        // Update existing sun or create new one
        if let Ok((sun_entity, mut transform, mut light)) = sun_query.get_single_mut() {
            transform.translation = sun_position;
            
            let sun_direction = Vec3::new(
                sun_angle.cos(),
                sun_angle.sin(),
                0.0,
            ).normalize();
            
            transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, sun_direction);
            
            // Adjust light intensity based on time only (no weather)
            let base_intensity = if time_of_day.is_day() {
                15000.0 * day_factor
            } else {
                0.0
            };
            
            light.illuminance = base_intensity;
            light.color = if time_of_day.is_day() {
                Color::srgb(1.0, 0.95, 0.8) // Warm sunlight
            } else {
                Color::srgb(0.3, 0.4, 0.7) // Cool moonlight
            };
        } else {
            // Create new sun
            commands.spawn((
                SunLight,
                DirectionalLight {
                    illuminance: if time_of_day.is_day() { 15000.0 * day_factor } else { 0.0 },
                    color: if time_of_day.is_day() {
                        Color::srgb(1.0, 0.95, 0.8)
                    } else {
                        Color::srgb(0.3, 0.4, 0.7)
                    },
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_translation(sun_position),
            ));
        }
    }
}

/// System to manage the moon and stars
#[cfg(feature = "weather")]
pub fn celestial_bodies_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: Res<TimeOfDay>,
    weather: Res<WeatherManager>,
    existing_celestial: Query<Entity, With<CelestialBody>>,
) {
    if time_of_day.is_changed() {
        // Clean up existing celestial bodies (except sun, handled separately)
        for entity in existing_celestial.iter() {
            commands.entity(entity).despawn();
        }
        
        // Only show moon and stars at night or during twilight
        if time_of_day.is_night() || time_of_day.get_day_night_factor() < 0.3 {
            let night_factor = 1.0 - time_of_day.get_day_night_factor();
            let weather_clarity = match weather.current_weather {
                WeatherType::Clear => 1.0,
                WeatherType::LightRain => 0.3,
                WeatherType::HeavyRain | WeatherType::Storm => 0.1,
                WeatherType::Fog => 0.2,
                WeatherType::Sandstorm => 0.05,
            };
            
            // Create moon
            let moon_angle = time_of_day.get_sun_angle() + std::f32::consts::PI; // Opposite to sun
            let moon_distance = 1400.0;
            let moon_position = Vec3::new(
                moon_angle.cos() * moon_distance,
                moon_angle.sin() * moon_distance,
                100.0,
            );
            
            commands.spawn((
                CelestialBody {
                    body_type: CelestialBodyType::Moon,
                    orbital_radius: moon_distance,
                    size: 30.0,
                },
                Mesh3d(meshes.add(Sphere::new(30.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.9, 0.9, 0.8, night_factor * weather_clarity),
                    emissive: LinearRgba::rgb(0.5, 0.5, 0.4) * night_factor * weather_clarity,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(moon_position),
            ));
            
            // Create stars
            let star_count = (200.0 * night_factor * weather_clarity) as usize;
            for _ in 0..star_count {
                let theta = rand::random::<f32>() * std::f32::consts::TAU;
                let phi = rand::random::<f32>() * std::f32::consts::PI;
                let distance = 1800.0 + rand::random::<f32>() * 200.0;
                
                let star_position = Vec3::new(
                    theta.cos() * phi.sin() * distance,
                    phi.cos() * distance,
                    theta.sin() * phi.sin() * distance,
                );
                
                let brightness = 0.1 + rand::random::<f32>() * 0.9;
                let star_size = 0.5 + brightness * 1.5;
                
                commands.spawn((
                    CelestialBody {
                        body_type: CelestialBodyType::Star { brightness },
                        orbital_radius: distance,
                        size: star_size,
                    },
                    Mesh3d(meshes.add(Sphere::new(star_size))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(1.0, 1.0, 0.9, brightness * night_factor * weather_clarity),
                        emissive: LinearRgba::rgb(brightness, brightness, brightness * 0.9) * night_factor * weather_clarity,
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_translation(star_position),
                ));
            }
        }
    }
}

/// System to manage the moon and stars (no weather version)
#[cfg(not(feature = "weather"))]
pub fn celestial_bodies_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: Res<TimeOfDay>,
    existing_celestial: Query<Entity, With<CelestialBody>>,
) {
    if time_of_day.is_changed() {
        // Clean up existing celestial bodies (except sun, handled separately)
        for entity in existing_celestial.iter() {
            commands.entity(entity).despawn();
        }
        
        let night_factor = 1.0 - time_of_day.get_day_night_factor();
        
        if !time_of_day.is_day() {
            // Create moon
            let moon_position = Vec3::new(
                -800.0 * (time_of_day.current_time * std::f32::consts::PI * 2.0).cos(),
                800.0 * (time_of_day.current_time * std::f32::consts::PI * 2.0).sin(),
                200.0,
            );
            
            commands.spawn((
                CelestialBody {
                    body_type: CelestialBodyType::Moon,
                    orbital_radius: 800.0,
                    size: 50.0,
                },
                Mesh3d(meshes.add(Sphere::new(50.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.9, 0.9, 0.95, night_factor),
                    emissive: LinearRgba::rgb(0.3, 0.3, 0.4) * night_factor,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(moon_position),
            ));
            
            // Create stars (simplified version without weather)
            for _ in 0..200 {
                let distance = 1500.0 + rand::random::<f32>() * 500.0;
                let azimuth = rand::random::<f32>() * std::f32::consts::PI * 2.0;
                let elevation = rand::random::<f32>() * std::f32::consts::PI * 0.3 + std::f32::consts::PI * 0.2;
                
                let star_position = Vec3::new(
                    distance * elevation.sin() * azimuth.cos(),
                    distance * elevation.cos(),
                    distance * elevation.sin() * azimuth.sin(),
                );
                
                let brightness = 0.1 + rand::random::<f32>() * 0.9;
                let star_size = 0.5 + brightness * 1.5;
                
                commands.spawn((
                    CelestialBody {
                        body_type: CelestialBodyType::Star { brightness },
                        orbital_radius: distance,
                        size: star_size,
                    },
                    Mesh3d(meshes.add(Sphere::new(star_size))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(1.0, 1.0, 0.9, brightness * night_factor),
                        emissive: LinearRgba::rgb(brightness, brightness, brightness * 0.9) * night_factor,
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_translation(star_position),
                ));
            }
        }
    }
}

/// System to manage animated clouds
#[cfg(feature = "weather")]
pub fn cloud_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_of_day: Res<TimeOfDay>,
    weather: Res<WeatherManager>,
    time: Res<Time>,
    mut existing_clouds: Query<(Entity, &mut Transform, &CloudLayer), With<Clouds>>,
    player_query: Query<&Transform, (With<Player>, Without<Clouds>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    
    let player_pos = player_transform.translation;
    
    // Determine cloud coverage based on weather
    let (cloud_density, cloud_opacity, cloud_height) = match weather.current_weather {
        WeatherType::Clear => (0.3, 0.6, 200.0),
        WeatherType::LightRain => (0.7, 0.8, 150.0),
        WeatherType::HeavyRain | WeatherType::Storm => (0.9, 0.95, 100.0),
        WeatherType::Fog => (0.5, 0.4, 50.0),
        WeatherType::Sandstorm => (0.8, 0.7, 80.0),
    };
    
    // Animate existing clouds
    for (entity, mut transform, cloud_layer) in existing_clouds.iter_mut() {
        transform.translation += cloud_layer.speed * time.delta_secs();
        
        // Remove clouds that are too far from player
        let distance = transform.translation.distance(player_pos);
        if distance > 1500.0 {
            commands.entity(entity).despawn();
        }
    }
    
    // Spawn new clouds periodically
    if time.elapsed_secs() % 2.0 < 0.1 { // Every 2 seconds
        let current_cloud_count = existing_clouds.iter().count();
        let target_cloud_count = (cloud_density * 50.0) as usize;
        
        if current_cloud_count < target_cloud_count {
            // Create new cloud layer
            let spawn_distance = 1000.0 + rand::random::<f32>() * 200.0;
            let spawn_angle = rand::random::<f32>() * std::f32::consts::TAU;
            
            let cloud_position = Vec3::new(
                player_pos.x + spawn_angle.cos() * spawn_distance,
                cloud_height + rand::random::<f32>() * 50.0,
                player_pos.z + spawn_angle.sin() * spawn_distance,
            );
            
            let cloud_scale = 20.0 + rand::random::<f32>() * 40.0;
            let wind_speed = weather.wind_direction * weather.wind_strength * 5.0;
            
            // Cloud color changes with time of day
            let day_factor = time_of_day.get_day_night_factor();
            let cloud_color = if time_of_day.is_day() {
                Color::srgba(1.0, 1.0, 1.0, cloud_opacity * day_factor)
            } else {
                Color::srgba(0.3, 0.3, 0.4, cloud_opacity * (1.0 - day_factor))
            };
            
            commands.spawn((
                Clouds,
                CloudLayer {
                    altitude: cloud_height,
                    speed: wind_speed + Vec3::new(rand::random::<f32>() - 0.5, 0.0, rand::random::<f32>() - 0.5),
                    opacity: cloud_opacity,
                    scale: cloud_scale,
                },
                Mesh3d(meshes.add(Sphere::new(cloud_scale))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: cloud_color,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(cloud_position)
                    .with_scale(Vec3::new(2.0, 0.5, 1.5)), // Flatten clouds
            ));
        }
    }
}

/// System to manage animated clouds (no-op version without weather)
#[cfg(not(feature = "weather"))]
pub fn cloud_system() {
    // No clouds when weather is disabled
}

#[cfg(feature = "weather")]
impl WeatherManager {
    fn get_light_modifier(&self) -> f32 {
        match self.current_weather {
            WeatherType::Clear => 1.0,
            WeatherType::LightRain => 0.7,
            WeatherType::HeavyRain => 0.4,
            WeatherType::Storm => 0.2,
            WeatherType::Fog => 0.5,
            WeatherType::Sandstorm => 0.3,
        }
    }
}

// UI Components
#[derive(Component)]
pub struct TimeDisplay;

#[derive(Component)]
pub struct TimeControlPanel;

/// System to handle time control inputs
pub fn time_control_system(
    input: Res<ButtonInput<KeyCode>>,
    mut time_of_day: ResMut<TimeOfDay>,
) {
    // Toggle time progression entirely with T key
    if input.just_pressed(KeyCode::KeyT) {
        time_of_day.enabled = !time_of_day.enabled;
        info!("Time of day progression: {}", if time_of_day.enabled { "ENABLED" } else { "DISABLED" });
    }
    

}

/// System to setup time UI
pub fn setup_time_ui(mut commands: Commands) {
    // Time display panel
    commands
        .spawn((
            TimeControlPanel,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(350.0),
                height: Val::Auto,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderRadius::all(Val::Px(5.0)),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            // Time display
            parent.spawn((
                TimeDisplay,
                Text::new("Time: 12:00 PM (Day)"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ));
            
            // Controls text
            parent.spawn((
                Text::new("\nTIME CONTROLS:\nT: Toggle Time On/Off\n+/- : Speed Up/Down\nSpace: Pause/Resume\n6: Dawn  0: Midnight\n12: Noon  18: Dusk"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ));
        });
}

/// System to update time UI
pub fn update_time_ui(
    time_of_day: Res<TimeOfDay>,
    mut time_display_query: Query<&mut Text, With<TimeDisplay>>,
) {
    if time_of_day.is_changed() {
        for mut text in time_display_query.iter_mut() {
            let hours = time_of_day.current_time as i32 % 24;
            let minutes = ((time_of_day.current_time % 1.0) * 60.0) as i32;
            
            let (display_hour, period) = if hours == 0 {
                (12, "AM")
            } else if hours <= 12 {
                (hours, if hours == 12 { "PM" } else { "AM" })
            } else {
                (hours - 12, "PM")
            };
            
            let day_night = if time_of_day.is_day() { "Day" } else { "Night" };
            let status = if !time_of_day.enabled { 
                " [DISABLED]" 
            } else if time_of_day.time_direction == 0.0 { 
                " [PAUSED]" 
            } else { 
                "" 
            };
            
            **text = format!("Time: {}:{:02} {} ({}) Speed: {:.1}x{}", 
                display_hour, minutes, period, day_night, time_of_day.time_speed, status);
        }
    }
}
