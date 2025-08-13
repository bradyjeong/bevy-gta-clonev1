use bevy::prelude::*;
use crate::components::weather::*;

/// System that manages weather audio effects
pub fn weather_audio_system(
    mut commands: Commands,
    weather: Res<WeatherManager>,
    weather_config: Res<WeatherConfig>,
    existing_audio: Query<Entity, With<WeatherAudio>>,
    player_query: Query<&Transform, (With<crate::components::Player>, With<crate::components::ActiveEntity>)>,
) {
    if !weather.is_changed() {
        return;
    }
    
    // Clear existing weather audio
    for entity in existing_audio.iter() {
        commands.entity(entity).despawn();
    }
    
    if weather.intensity <= 0.01 {
        return;
    }
    
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    
    let Some(config) = weather_config.configs.get(&weather.current_weather) else {
        return;
    };
    
    let volume = config.sound_volume * weather.intensity;
    
    match weather.current_weather {
        WeatherType::LightRain | WeatherType::HeavyRain => {
            spawn_rain_audio(&mut commands, player_transform.translation, volume);
        },
        WeatherType::Storm => {
            spawn_rain_audio(&mut commands, player_transform.translation, volume);
            spawn_thunder_audio(&mut commands, player_transform.translation, volume);
        },
        WeatherType::Sandstorm => {
            spawn_wind_audio(&mut commands, player_transform.translation, volume);
        },
        WeatherType::Fog => {
            // Fog is generally quiet, maybe subtle wind
            spawn_wind_audio(&mut commands, player_transform.translation, volume * 0.3);
        },
        WeatherType::Clear => {
            // No weather audio for clear weather
        }
    }
}

fn spawn_rain_audio(commands: &mut Commands, player_pos: Vec3, volume: f32) {
    commands.spawn((
        Transform::from_translation(player_pos),
        WeatherAudio {
            sound_type: WeatherSoundType::RainAmbient,
            volume,
            fade_distance: 100.0,
        },
    ));
}

fn spawn_thunder_audio(commands: &mut Commands, player_pos: Vec3, volume: f32) {
    // Spawn thunder at random intervals during storms
    if rand::random::<f32>() < 0.02 { // 2% chance per frame during storm
        let thunder_distance = 50.0 + rand::random::<f32>() * 200.0;
        let thunder_direction = Vec3::new(
            rand::random::<f32>() - 0.5,
            0.0,
            rand::random::<f32>() - 0.5,
        ).normalize();
        
        commands.spawn((
            Transform::from_translation(player_pos + thunder_direction * thunder_distance),
            WeatherAudio {
                sound_type: WeatherSoundType::Thunder,
                volume: volume * (1.0 - thunder_distance / 250.0).max(0.1), // Volume decreases with distance
                fade_distance: thunder_distance,
            },
        ));
    }
}

fn spawn_wind_audio(commands: &mut Commands, player_pos: Vec3, volume: f32) {
    commands.spawn((
        Transform::from_translation(player_pos),
        WeatherAudio {
            sound_type: WeatherSoundType::WindAmbient,
            volume,
            fade_distance: 80.0,
        },
    ));
}

/// System that updates audio volume based on weather intensity and distance
pub fn weather_audio_update_system(
    mut audio_query: Query<(&Transform, &mut WeatherAudio)>,
    weather: Res<WeatherManager>,
    player_query: Query<&Transform, (With<crate::components::Player>, With<crate::components::ActiveEntity>, Without<WeatherAudio>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    
    for (audio_transform, mut weather_audio) in audio_query.iter_mut() {
        let distance = player_transform.translation.distance(audio_transform.translation);
        let distance_factor = (1.0 - distance / weather_audio.fade_distance).max(0.0);
        
        // Update volume based on weather intensity and distance
        let base_volume = match weather_audio.sound_type {
            WeatherSoundType::RainAmbient => weather.intensity,
            WeatherSoundType::Thunder => 1.0, // Thunder is always full volume when it happens
            WeatherSoundType::WindAmbient => weather.intensity * weather.wind_strength / 10.0,
            WeatherSoundType::SandstormAmbient => weather.intensity,
        };
        
        weather_audio.volume = base_volume * distance_factor;
    }
}

/// System for cleaning up old thunder audio effects
pub fn cleanup_thunder_system(
    mut commands: Commands,
    thunder_query: Query<(Entity, &WeatherAudio)>,
    _time: Res<Time>,
) {
    // Thunder sounds should be cleaned up after a few seconds
    for (entity, audio) in thunder_query.iter() {
        if matches!(audio.sound_type, WeatherSoundType::Thunder) {
            // Remove thunder after 3 seconds (assuming thunder sound length)
            // In a real implementation, you'd want to track when the thunder was spawned
            if rand::random::<f32>() < 0.01 { // Simple cleanup - in real game you'd track spawn time
                commands.entity(entity).despawn();
            }
        }
    }
}
