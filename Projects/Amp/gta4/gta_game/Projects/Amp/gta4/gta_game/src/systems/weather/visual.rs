use bevy::prelude::*;
use crate::components::*;

/// System that manages weather visual effects (particles, fog, etc.)
pub fn weather_visual_effects_system(
    mut commands: Commands,
    weather: Res<WeatherManager>,
    weather_config: Res<WeatherConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, (With<Player>, With<ActiveEntity>)>,
    existing_effects: Query<Entity, With<WeatherEffect>>,
    time: Res<Time>,
) {
    // Only update visual effects every few frames for performance, but always when weather changes
    if !weather.is_changed() && time.elapsed_secs() % 0.5 > 0.1 {
        return;
    }
    
    // Clear existing weather effects when weather changes significantly
    if weather.is_changed() {
        for entity in existing_effects.iter() {
            commands.entity(entity).despawn();
        }
    }
    
    if weather.intensity <= 0.01 {
        return;
    }
    
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    
    // Determine which weather to show visuals for
    let active_weather = if let Some(ref target) = weather.target_weather {
        // During transition, show target weather effects if intensity > 0.5
        if weather.intensity > 0.5 {
            target.clone()
        } else {
            weather.current_weather.clone()
        }
    } else {
        weather.current_weather.clone()
    };
    
    let Some(config) = weather_config.configs.get(&active_weather) else {
        return;
    };
    
    let player_pos = player_transform.translation;
    
    // Debug logging
    #[cfg(feature = "debug-weather")]
    info!("Weather visuals: {:?} -> {:?} intensity: {:.2}", 
          weather.current_weather, active_weather, weather.intensity);
    
    match active_weather {
        WeatherType::LightRain | WeatherType::HeavyRain | WeatherType::Storm => {
            spawn_rain_effects(&mut commands, &mut meshes, &mut materials, player_pos, weather.intensity, config);
        },
        WeatherType::Fog => {
            spawn_fog_effects(&mut commands, &mut meshes, &mut materials, player_pos, weather.intensity);
        },
        WeatherType::Sandstorm => {
            spawn_sandstorm_effects(&mut commands, &mut meshes, &mut materials, player_pos, weather.intensity, &weather.wind_direction);
        },
        WeatherType::Clear => {
            // No visual effects for clear weather
        }
    }
}

fn spawn_rain_effects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_pos: Vec3,
    intensity: f32,
    config: &WeatherSettings,
) {
    let drop_count = (config.particle_density * intensity * 300.0) as usize; // Balanced particle count
    let area_size = 60.0; // Smaller area for better density
    
    for _ in 0..drop_count {
        let offset = Vec3::new(
            rand::random::<f32>() * area_size - area_size / 2.0,
            rand::random::<f32>() * 50.0 + 20.0, // Spawn above player
            rand::random::<f32>() * area_size - area_size / 2.0,
        );
        
        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.1, 2.0))), // Even larger and longer
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.5, 1.0, 1.0), // Bright blue, fully opaque
                alpha_mode: AlphaMode::Opaque, // No transparency
                unlit: true, // Make sure it's visible
                ..default()
            })),
            Transform::from_translation(player_pos + offset),
            WeatherEffect {
                effect_type: WeatherEffectType::RainParticles,
                intensity,
                lifetime: Some(5.0), // Longer lifetime
            },
            Cullable { max_distance: 150.0, is_culled: false },
        ));
    }
}

fn spawn_fog_effects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_pos: Vec3,
    intensity: f32,
) {
    // Create many small fog particles at ground level for realistic fog
    let fog_particle_count = (intensity * 100.0) as usize;
    let fog_area = 80.0;
    
    for _ in 0..fog_particle_count {
        let offset = Vec3::new(
            rand::random::<f32>() * fog_area - fog_area / 2.0,
            rand::random::<f32>() * 5.0, // Keep fog low to ground
            rand::random::<f32>() * fog_area - fog_area / 2.0,
        );
        
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(3.0 + rand::random::<f32>() * 4.0))), // Smaller particles
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.9, 0.9, 0.9, 0.15 * intensity), // Much more transparent
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(player_pos + offset),
            WeatherEffect {
                effect_type: WeatherEffectType::FogVolume,
                intensity,
                lifetime: None,
            },
            Cullable { max_distance: 120.0, is_culled: false },
        ));
    }
    
    // Add some larger, more transparent fog layers for atmosphere
    for i in 0..4 {
        let angle = (i as f32) * std::f32::consts::TAU / 4.0;
        let distance = 40.0 + rand::random::<f32>() * 30.0;
        let offset = Vec3::new(
            angle.cos() * distance,
            2.0 + rand::random::<f32>() * 3.0, // Slightly elevated
            angle.sin() * distance,
        );
        
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(25.0 + rand::random::<f32>() * 20.0))), // Large atmospheric fog
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.85, 0.85, 0.85, 0.08 * intensity), // Very transparent
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(player_pos + offset),
            WeatherEffect {
                effect_type: WeatherEffectType::FogVolume,
                intensity,
                lifetime: None,
            },
            Cullable { max_distance: 150.0, is_culled: false },
        ));
    }
}

fn spawn_sandstorm_effects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_pos: Vec3,
    intensity: f32,
    _wind_direction: &Vec3,
) {
    let particle_count = (intensity * 100.0) as usize; // Reduced for performance
    let area_size = 70.0;
    
    for _ in 0..particle_count {
        let offset = Vec3::new(
            rand::random::<f32>() * area_size - area_size / 2.0,
            rand::random::<f32>() * 30.0,
            rand::random::<f32>() * area_size - area_size / 2.0,
        );
        
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.2 + rand::random::<f32>() * 0.3))), // Larger sand particles
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.9, 0.7, 0.4, 0.9), // More opaque
                alpha_mode: AlphaMode::Blend,
                unlit: true, // Make sure it's visible
                ..default()
            })),
            Transform::from_translation(player_pos + offset),
            WeatherEffect {
                effect_type: WeatherEffectType::SandParticles,
                intensity,
                lifetime: Some(8.0), // Longer lifetime
            },
            Cullable { max_distance: 120.0, is_culled: false },
        ));
    }
}

/// System that animates weather particles
pub fn weather_particle_animation_system(
    mut commands: Commands,
    mut weather_effects: Query<(Entity, &mut Transform, &mut WeatherEffect), With<WeatherEffect>>,
    weather: Res<WeatherManager>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    
    for (entity, mut transform, mut effect) in weather_effects.iter_mut() {
        match effect.effect_type {
            WeatherEffectType::RainParticles => {
                // Rain falls down with some wind influence
                let fall_speed = 15.0;
                let wind_influence = weather.wind_direction * weather.wind_strength * 0.5;
                transform.translation.y -= fall_speed * delta;
                transform.translation += wind_influence * delta;
                
                // Remove when hits ground
                if transform.translation.y < 0.0 {
                    commands.entity(entity).despawn();
                }
            },
            WeatherEffectType::SandParticles => {
                // Sand moves with wind
                let wind_force = weather.wind_direction * weather.wind_strength * 2.0;
                transform.translation += wind_force * delta;
                
                // Add some random turbulence
                let turbulence = Vec3::new(
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>() - 0.5,
                ) * 0.5;
                transform.translation += turbulence * delta;
            },
            WeatherEffectType::FogVolume => {
                // Fog slowly drifts with wind
                let drift = weather.wind_direction * weather.wind_strength * 0.1;
                transform.translation += drift * delta;
                
                // Slight scale pulsing for fog movement
                let scale_pulse = 1.0 + (time.elapsed_secs() * 0.5).sin() * 0.1;
                transform.scale = Vec3::splat(scale_pulse);
            },
            _ => {}
        }
        
        // Handle lifetime
        if let Some(ref mut lifetime) = effect.lifetime {
            *lifetime -= delta;
            if *lifetime <= 0.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System that updates wind-responsive objects
pub fn wind_responsive_system(
    mut wind_objects: Query<(&mut Transform, &mut WindResponsive)>,
    weather: Res<WeatherManager>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    
    for (mut transform, mut wind_responsive) in wind_objects.iter_mut() {
        // Calculate wind sway
        wind_responsive.current_sway += delta * wind_responsive.sway_frequency;
        
        let wind_strength = weather.wind_strength * weather.intensity;
        let sway_offset = (wind_responsive.current_sway).sin() * wind_responsive.sway_amount * wind_strength;
        
        // Apply sway in wind direction
        let sway_vector = weather.wind_direction * sway_offset;
        
        // Create rotation from sway
        let sway_rotation = Quat::from_axis_angle(
            Vec3::new(-weather.wind_direction.z, 0.0, weather.wind_direction.x).normalize(),
            sway_offset * 0.1
        );
        
        // Apply to transform
        transform.rotation = wind_responsive.base_transform.rotation * sway_rotation;
        transform.translation = wind_responsive.base_transform.translation + sway_vector * 0.1;
    }
}
