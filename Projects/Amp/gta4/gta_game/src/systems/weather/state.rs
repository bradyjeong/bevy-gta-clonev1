use bevy::prelude::*;
use crate::components::weather::*;
use crate::systems::timing_service::{TimingService, SystemType};
use rand::Rng;

/// System that manages weather state transitions
pub fn weather_state_system(
    mut weather: ResMut<WeatherManager>,
    mut timing_service: ResMut<TimingService>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    
    // Handle weather transitions
    if let Some(target_weather) = weather.target_weather.clone() {
        weather.transition_timer += delta;
        
        // Calculate transition progress (0.0 to 1.0)
        let progress = (weather.transition_timer / weather.transition_duration).clamp(0.0, 1.0);
        
        // Update intensity based on transition
        weather.intensity = match (&weather.current_weather, &target_weather) {
            (WeatherType::Clear, _) => progress,
            (_, WeatherType::Clear) => 1.0 - progress,
            _ => {
                // Transition between two weather types: fade out then fade in
                if progress < 0.5 {
                    1.0 - (progress * 2.0)
                } else {
                    (progress - 0.5) * 2.0
                }
            }
        };
        
        // Complete transition
        if progress >= 1.0 {
            #[cfg(debug_assertions)]
            info!("Weather transition completed: {:?} at intensity {:.2}", target_weather, weather.intensity);
            weather.current_weather = target_weather.clone();
            weather.target_weather = None;
            weather.transition_timer = 0.0;
            weather.intensity = if weather.current_weather == WeatherType::Clear { 0.0 } else { 1.0 };
        }
    } else {
        // No transition - log current state occasionally
        #[cfg(feature = "debug-weather")]
        if timing_service.should_run_system(SystemType::WeatherDebug) {
            info!("Current weather: {:?} intensity: {:.2}", weather.current_weather, weather.intensity);
        }
    }
    
    // Random weather changes (every 2-5 minutes)
    if weather.target_weather.is_none() && rand::thread_rng().gen_bool(0.0001) {
        weather.trigger_random_weather();
    }
    
    // Update wind direction gradually
    let wind_change = Vec3::new(
        rand::thread_rng().gen_range(-0.1..0.1),
        0.0,
        rand::thread_rng().gen_range(-0.1..0.1),
    );
    weather.wind_direction = (weather.wind_direction + wind_change * delta).normalize();
}

impl WeatherManager {
    pub fn trigger_weather(&mut self, new_weather: WeatherType) {
        if new_weather != self.current_weather {
            #[cfg(feature = "debug-weather")]
            info!("Weather changing from {:?} to {:?}", self.current_weather, new_weather);
            self.target_weather = Some(new_weather);
            self.transition_timer = 0.0;
        }
    }
    
    pub fn trigger_random_weather(&mut self) {
        use rand::seq::SliceRandom;
        let possible_weather = [
            WeatherType::Clear,
            WeatherType::LightRain,
            WeatherType::HeavyRain,
            WeatherType::Storm,
            WeatherType::Fog,
            WeatherType::Sandstorm,
        ];
        
        let new_weather = possible_weather.choose(&mut rand::thread_rng()).unwrap().clone();
        self.trigger_weather(new_weather);
    }
}

/// System that updates weather effects on lighting
pub fn weather_lighting_system(
    weather: Res<WeatherManager>,
    weather_config: Res<WeatherConfig>,
    mut sun_query: Query<(&mut DirectionalLight, &mut Transform), With<DirectionalLight>>,
) {
    if !weather.is_changed() {
        return;
    }
    
    let Some(config) = weather_config.configs.get(&weather.current_weather) else {
        return;
    };
    
    for (mut light, mut transform) in sun_query.iter_mut() {
        // Interpolate lighting based on weather intensity
        let clear_config = &weather_config.configs[&WeatherType::Clear];
        
        light.color = clear_config.light_color.mix(&config.light_color, weather.intensity);
        light.illuminance = clear_config.light_intensity + 
            (config.light_intensity - clear_config.light_intensity) * weather.intensity;
        
        // Adjust sun angle for stormy weather (more dramatic shadows)
        if matches!(weather.current_weather, WeatherType::Storm | WeatherType::Sandstorm) {
            let storm_angle = -0.8 * weather.intensity;
            let base_rotation = Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0);
            let storm_rotation = Quat::from_euler(EulerRot::XYZ, storm_angle, -0.5, 0.0);
            transform.rotation = base_rotation.lerp(storm_rotation, weather.intensity);
        }
    }
}
