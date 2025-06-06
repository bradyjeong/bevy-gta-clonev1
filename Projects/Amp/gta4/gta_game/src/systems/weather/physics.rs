use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;

/// System that applies weather effects to vehicle physics
pub fn weather_vehicle_physics_system(
    weather: Res<WeatherManager>,
    weather_config: Res<WeatherConfig>,
    mut vehicle_query: Query<(&mut Friction, Option<&SuperCar>), (With<Car>, With<ActiveEntity>)>,
    mut helicopter_query: Query<&mut Damping, (With<Helicopter>, With<ActiveEntity>)>,
) {
    if !weather.is_changed() {
        return;
    }
    
    let Some(config) = weather_config.configs.get(&weather.current_weather) else {
        return;
    };
    
    let physics_modifier = config.physics_modifier;
    let wind_effect = weather.wind_strength * weather.intensity / 10.0;
    
    // Update vehicle friction (both regular cars and supercars)
    for (mut friction, supercar) in vehicle_query.iter_mut() {
        if supercar.is_some() {
            // Supercars are more affected by weather
            friction.coefficient = 0.3 * physics_modifier * 0.8;
        } else {
            // Regular cars
            friction.coefficient = 0.3 * physics_modifier;
        }
    }
    
    // Update helicopter damping (wind affects helicopter more)
    for mut damping in helicopter_query.iter_mut() {
        damping.linear_damping = 2.0 + wind_effect;
        damping.angular_damping = 8.0 + wind_effect * 2.0;
    }
}

/// System that applies weather effects to player movement
pub fn weather_player_physics_system(
    weather: Res<WeatherManager>,
    weather_config: Res<WeatherConfig>,
    mut player_query: Query<&mut Damping, (With<Player>, With<ActiveEntity>)>,
) {
    if !weather.is_changed() {
        return;
    }
    
    let Some(_config) = weather_config.configs.get(&weather.current_weather) else {
        return;
    };
    
    let wind_resistance = weather.wind_strength * weather.intensity / 15.0;
    
    for mut damping in player_query.iter_mut() {
        // Player movement is slightly hindered by strong winds
        damping.linear_damping = 2.0 + wind_resistance;
    }
}

/// System that updates rain collection on surfaces
pub fn rain_collection_system(
    weather: Res<WeatherManager>,
    mut rain_collectors: Query<&mut RainCollector>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    
    for mut collector in rain_collectors.iter_mut() {
        match weather.current_weather {
            WeatherType::LightRain | WeatherType::HeavyRain | WeatherType::Storm => {
                // Accumulate wetness during rain
                let rain_rate = match weather.current_weather {
                    WeatherType::LightRain => 0.1,
                    WeatherType::HeavyRain => 0.3,
                    WeatherType::Storm => 0.5,
                    _ => 0.0,
                } * weather.intensity;
                
                collector.wetness = (collector.wetness + rain_rate * delta).min(1.0);
                collector.puddle_depth = (collector.puddle_depth + rain_rate * delta * 0.5).min(0.1);
            },
            _ => {
                // Dry out during non-rain weather
                collector.wetness = (collector.wetness - collector.dry_rate * delta).max(0.0);
                collector.puddle_depth = (collector.puddle_depth - collector.dry_rate * delta * 0.3).max(0.0);
            }
        }
    }
}

/// System that applies weather visibility effects
pub fn weather_visibility_system(
    weather: Res<WeatherManager>,
    weather_config: Res<WeatherConfig>,
    mut affected_query: Query<&mut WeatherAffected>,
    mut culling_settings: ResMut<CullingSettings>,
) {
    if !weather.is_changed() {
        return;
    }
    
    let Some(config) = weather_config.configs.get(&weather.current_weather) else {
        return;
    };
    
    let visibility_modifier = (config.visibility_range / 1000.0).min(1.0);
    
    // Update all weather-affected entities
    for mut affected in affected_query.iter_mut() {
        affected.visibility_modifier = visibility_modifier;
    }
    
    // Update culling distances based on visibility
    culling_settings._npc_cull_distance = 200.0 * visibility_modifier;
    culling_settings._car_cull_distance = 300.0 * visibility_modifier;
    culling_settings._building_cull_distance = 800.0 * visibility_modifier;
    culling_settings._tree_cull_distance = 400.0 * visibility_modifier;
}

/// System that applies wind forces to physics objects
pub fn wind_physics_system(
    _weather: Res<WeatherManager>,
    _wind_affected: Query<(&mut ExternalForce, &Transform), (With<WeatherAffected>, Without<Car>, Without<Helicopter>, Without<F16>)>,
    _vehicle_wind_affected: Query<(&mut ExternalForce, &Transform), (With<WeatherAffected>, Or<(With<Car>, With<Helicopter>, With<F16>)>, With<ActiveEntity>)>,
    _time: Res<Time>,
) {
    // WIND DISABLED - no forces applied
    return;
    
    let wind_force_magnitude = weather.wind_strength * weather.intensity * 0.5;
    
    // Apply wind to non-vehicle objects
    for (mut external_force, transform) in wind_affected.iter_mut() {
        // Calculate wind force based on object's surface area (simplified)
        let height_factor = (transform.translation.y / 10.0).min(1.0).max(0.1);
        let force = weather.wind_direction * wind_force_magnitude * height_factor;
        
        // Add some turbulence
        let turbulence = Vec3::new(
            (time.elapsed_secs() * 2.0).sin(),
            0.0,
            (time.elapsed_secs() * 1.5).cos(),
        ) * wind_force_magnitude * 0.2;
        
        external_force.force = force + turbulence;
    }
    
    // Apply wind to vehicles only when they are actively being driven
    for (mut external_force, transform) in vehicle_wind_affected.iter_mut() {
        // Calculate wind force based on object's surface area (simplified)
        let height_factor = (transform.translation.y / 10.0).min(1.0).max(0.1);
        let force = weather.wind_direction * wind_force_magnitude * height_factor * 0.3; // Reduced for vehicles
        
        // Add some turbulence
        let turbulence = Vec3::new(
            (time.elapsed_secs() * 2.0).sin(),
            0.0,
            (time.elapsed_secs() * 1.5).cos(),
        ) * wind_force_magnitude * 0.1; // Less turbulence for vehicles
        
        external_force.force = force + turbulence;
    }
}
