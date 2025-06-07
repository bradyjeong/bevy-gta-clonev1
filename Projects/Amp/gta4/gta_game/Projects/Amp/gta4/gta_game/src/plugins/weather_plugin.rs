#[cfg(feature = "weather")]
use bevy::prelude::*;
#[cfg(feature = "weather")]
use crate::components::weather::*;
#[cfg(feature = "weather")]
use crate::systems::weather::*;

#[cfg(feature = "weather")]
pub struct WeatherPlugin;

#[cfg(feature = "weather")]
impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add weather resources
            .init_resource::<WeatherManager>()
            .init_resource::<WeatherConfig>()
            
            // Add weather events
            .add_event::<WeatherChangeEvent>()
            
            // Weather state management systems
            .add_systems(Update, (
                weather_state_system,
                weather_lighting_system,
                handle_weather_events,
            ))
            
            // Visual effects systems
            .add_systems(Update, (
                weather_visual_effects_system,
                weather_particle_animation_system,
                wind_responsive_system,
            ))
            
            // Audio systems
            .add_systems(Update, (
                weather_audio_system,
                weather_audio_update_system,
                cleanup_thunder_system,
            ))
            
            // Physics systems - run in FixedUpdate for consistent physics
            .add_systems(FixedUpdate, (
                weather_vehicle_physics_system,
                weather_player_physics_system,
                rain_collection_system,
                weather_visibility_system,
                wind_physics_system,
            ));
            
        // Add debug controls in debug builds
        #[cfg(debug_assertions)]
        app.add_systems(Update, debug_weather_controls);
    }
}

// Weather control events (for debugging or scripted weather changes)
#[cfg(feature = "weather")]
#[derive(Event)]
pub struct WeatherChangeEvent {
    pub new_weather: WeatherType,
    pub immediate: bool, // Skip transition if true
}

// System to handle weather change events
#[cfg(feature = "weather")]
pub fn handle_weather_events(
    mut events: EventReader<WeatherChangeEvent>,
    mut weather: ResMut<WeatherManager>,
) {
    for event in events.read() {
        if event.immediate {
            weather.current_weather = event.new_weather.clone();
            weather.target_weather = None;
            weather.intensity = if weather.current_weather == WeatherType::Clear { 0.0 } else { 1.0 };
        } else {
            weather.trigger_weather(event.new_weather.clone());
        }
    }
}

// Debug system for weather controls (optional - can be enabled for testing)
#[cfg(all(debug_assertions, feature = "weather"))]
pub fn debug_weather_controls(
    input: Res<ButtonInput<KeyCode>>,
    mut weather_events: EventWriter<WeatherChangeEvent>,
) {
    if input.just_pressed(KeyCode::Digit1) {
        weather_events.write(WeatherChangeEvent {
            new_weather: WeatherType::Clear,
            immediate: true, // Immediate for debugging
        });
    }
    if input.just_pressed(KeyCode::Digit2) {
        weather_events.write(WeatherChangeEvent {
            new_weather: WeatherType::LightRain,
            immediate: true, // Immediate for debugging
        });
    }
    if input.just_pressed(KeyCode::Digit3) {
        weather_events.write(WeatherChangeEvent {
            new_weather: WeatherType::HeavyRain,
            immediate: true, // Immediate for debugging
        });
    }
    if input.just_pressed(KeyCode::Digit4) {
        weather_events.write(WeatherChangeEvent {
            new_weather: WeatherType::Storm,
            immediate: true, // Immediate for debugging
        });
    }
    if input.just_pressed(KeyCode::Digit5) {
        weather_events.write(WeatherChangeEvent {
            new_weather: WeatherType::Fog,
            immediate: true, // Immediate for debugging
        });
    }
    if input.just_pressed(KeyCode::Digit6) {
        weather_events.write(WeatherChangeEvent {
            new_weather: WeatherType::Sandstorm,
            immediate: true, // Immediate for debugging
        });
    }
}
