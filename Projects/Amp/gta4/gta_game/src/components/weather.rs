use bevy::prelude::*;

/// Current weather state for the world
#[derive(Resource, Debug, Clone)]
pub struct WeatherManager {
    pub current_weather: WeatherType,
    pub intensity: f32, // 0.0 to 1.0
    pub transition_timer: f32,
    pub transition_duration: f32,
    pub target_weather: Option<WeatherType>,
    pub wind_direction: Vec3,
    pub wind_strength: f32,
}

impl Default for WeatherManager {
    fn default() -> Self {
        Self {
            current_weather: WeatherType::Clear,
            intensity: 0.0,
            transition_timer: 0.0,
            transition_duration: 30.0, // 30 seconds for weather transitions
            target_weather: None,
            wind_direction: Vec3::new(1.0, 0.0, 0.5).normalize(),
            wind_strength: 2.0,
        }
    }
}

/// Types of weather in the game
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WeatherType {
    Clear,
    LightRain,
    HeavyRain,
    Storm,
    Fog,
    Sandstorm, // Desert-themed for Dubai setting
}

/// Component for entities affected by weather
#[derive(Component)]
pub struct WeatherAffected {
    pub visibility_modifier: f32,
    pub wind_resistance: f32,
}

impl Default for WeatherAffected {
    fn default() -> Self {
        Self {
            visibility_modifier: 1.0,
            wind_resistance: 1.0,
        }
    }
}

/// Component for surfaces that collect rain (show puddles, wetness)
#[derive(Component)]
pub struct RainCollector {
    pub wetness: f32, // 0.0 to 1.0
    pub puddle_depth: f32,
    pub dry_rate: f32, // How fast it dries when rain stops
}

impl Default for RainCollector {
    fn default() -> Self {
        Self {
            wetness: 0.0,
            puddle_depth: 0.0,
            dry_rate: 0.1, // Dries over 10 seconds
        }
    }
}

/// Component for objects that respond to wind (trees, signs, etc)
#[derive(Component)]
pub struct WindResponsive {
    pub sway_amount: f32,
    pub sway_frequency: f32,
    pub base_transform: Transform,
    pub current_sway: f32,
}

impl Default for WindResponsive {
    fn default() -> Self {
        Self {
            sway_amount: 0.1,
            sway_frequency: 1.0,
            base_transform: Transform::IDENTITY,
            current_sway: 0.0,
        }
    }
}

/// Component for weather-related visual effects
#[derive(Component)]
pub struct WeatherEffect {
    pub effect_type: WeatherEffectType,
    pub intensity: f32,
    pub lifetime: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum WeatherEffectType {
    RainParticles,
    FogVolume,
    SandParticles,
    Lightning,
    WindLines,
}

/// Component for weather audio zones
#[derive(Component)]
pub struct WeatherAudio {
    pub sound_type: WeatherSoundType,
    pub volume: f32,
    pub fade_distance: f32,
}

#[derive(Debug, Clone)]
pub enum WeatherSoundType {
    RainAmbient,
    Thunder,
    WindAmbient,
    SandstormAmbient,
}

/// Weather configuration for different weather types
#[derive(Resource)]
pub struct WeatherConfig {
    pub configs: std::collections::HashMap<WeatherType, WeatherSettings>,
}

#[derive(Debug, Clone)]
pub struct WeatherSettings {
    pub visibility_range: f32,
    pub light_color: Color,
    pub light_intensity: f32,
    pub wind_strength: f32,
    pub particle_density: f32,
    pub sound_volume: f32,
    pub physics_modifier: f32, // For vehicle traction, etc.
}

impl Default for WeatherConfig {
    fn default() -> Self {
        let mut configs = std::collections::HashMap::new();
        
        configs.insert(WeatherType::Clear, WeatherSettings {
            visibility_range: 1000.0,
            light_color: Color::srgb(1.0, 0.9, 0.7),
            light_intensity: 10000.0,
            wind_strength: 1.0,
            particle_density: 0.0,
            sound_volume: 0.0,
            physics_modifier: 1.0,
        });
        
        configs.insert(WeatherType::LightRain, WeatherSettings {
            visibility_range: 500.0,
            light_color: Color::srgb(0.7, 0.8, 0.9),
            light_intensity: 6000.0,
            wind_strength: 2.0,
            particle_density: 0.3,
            sound_volume: 0.4,
            physics_modifier: 0.8, // Slightly reduced traction
        });
        
        configs.insert(WeatherType::HeavyRain, WeatherSettings {
            visibility_range: 200.0,
            light_color: Color::srgb(0.5, 0.6, 0.8),
            light_intensity: 4000.0,
            wind_strength: 4.0,
            particle_density: 0.8,
            sound_volume: 0.8,
            physics_modifier: 0.6, // Reduced traction
        });
        
        configs.insert(WeatherType::Storm, WeatherSettings {
            visibility_range: 100.0,
            light_color: Color::srgb(0.3, 0.4, 0.6),
            light_intensity: 2000.0,
            wind_strength: 8.0,
            particle_density: 1.0,
            sound_volume: 1.0,
            physics_modifier: 0.4, // Very reduced traction
        });
        
        configs.insert(WeatherType::Fog, WeatherSettings {
            visibility_range: 50.0,
            light_color: Color::srgb(0.8, 0.8, 0.8),
            light_intensity: 5000.0,
            wind_strength: 0.5,
            particle_density: 0.6,
            sound_volume: 0.2,
            physics_modifier: 0.9,
        });
        
        configs.insert(WeatherType::Sandstorm, WeatherSettings {
            visibility_range: 80.0,
            light_color: Color::srgb(0.9, 0.7, 0.4),
            light_intensity: 3000.0,
            wind_strength: 10.0,
            particle_density: 0.9,
            sound_volume: 0.9,
            physics_modifier: 0.7,
        });
        
        Self { configs }
    }
}
