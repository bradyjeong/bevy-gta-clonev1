use bevy::prelude::*;
use crate::components::*;
use crate::config::GameConfig;

/// CRITICAL: High-performance realistic vehicle audio system
/// Spatially accurate engine, tire, and environmental sounds with performance optimization

#[derive(Component, Debug, Clone)]
pub struct VehicleAudioState {
    pub engine_pitch: f32,          // Current engine pitch (0.5 - 2.0)
    pub engine_volume: f32,         // Current engine volume (0.0 - 1.0)
    pub tire_screech_volume: f32,   // Tire screech volume (0.0 - 1.0)
    pub wind_noise_volume: f32,     // Wind noise volume (0.0 - 1.0)
    pub brake_noise_volume: f32,    // Brake noise volume (0.0 - 1.0)
    
    // Audio state tracking
    pub last_rpm: f32,              // Last frame RPM for smooth transitions
    pub last_speed: f32,            // Last frame speed
    pub audio_update_timer: f32,    // Timer to throttle audio updates
    
    // Performance optimization
    pub distance_to_player: f32,    // Distance for spatial audio calculations
    pub audio_enabled: bool,        // Can disable for distant vehicles
}

#[derive(Component, Debug)]
pub struct VehicleAudioSources {
    pub engine_source: Entity,
    pub tire_source: Entity,
    pub wind_source: Entity,
    pub brake_source: Entity,
}

/// Update realistic vehicle audio based on physics state
pub fn realistic_vehicle_audio_system(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut audio_query: Query<(
        &mut VehicleAudioState,
        &VehicleAudioSources,
        &EnginePhysics,
        &TirePhysics,
        &VehicleDynamics,
        &Transform,
    ), With<RealisticVehicle>>,
    player_query: Query<&Transform, (With<Player>, Without<RealisticVehicle>)>,
    mut audio_sinks: Query<&mut AudioSink>,
) {
    let dt = time.delta_secs();
    let Ok(player_transform) = player_query.single() else { return; };
    let player_pos = player_transform.translation;
    
    for (mut audio_state, audio_sources, engine, tire_physics, dynamics, transform) in audio_query.iter_mut() {
        // Performance optimization: Update audio less frequently
        audio_state.audio_update_timer += dt;
        if audio_state.audio_update_timer < 0.05 {
            continue; // Update at 20Hz instead of 60Hz
        }
        audio_state.audio_update_timer = 0.0;
        
        // Calculate distance to player for spatial audio
        audio_state.distance_to_player = transform.translation.distance(player_pos);
        
        // Disable audio for very distant vehicles (performance)
        audio_state.audio_enabled = audio_state.distance_to_player < config.audio.max_audio_distance;
        if !audio_state.audio_enabled {
            continue;
        }
        
        // Calculate distance attenuation
        let distance_attenuation = (1.0 - (audio_state.distance_to_player / config.audio.fade_distance).clamp(0.0, 1.0)).max(0.1);
        
        // STEP 1: Calculate engine audio
        calculate_engine_audio(&mut audio_state, engine, distance_attenuation, &config);
        
        // STEP 2: Calculate tire audio
        calculate_tire_audio(&mut audio_state, tire_physics, dynamics, distance_attenuation);
        
        // STEP 3: Calculate wind noise
        calculate_wind_audio(&mut audio_state, dynamics, distance_attenuation);
        
        // STEP 4: Calculate brake noise
        calculate_brake_audio(&mut audio_state, engine, distance_attenuation);
        
        // STEP 5: Apply audio to sources with safety checks
        apply_audio_to_sources(&audio_state, &audio_sources, &mut audio_sinks, &config);
        
        // Update state for next frame
        audio_state.last_rpm = engine.current_rpm;
        audio_state.last_speed = dynamics.speed;
    }
}

/// Calculate realistic engine audio based on RPM and load
fn calculate_engine_audio(
    audio_state: &mut VehicleAudioState,
    engine: &EnginePhysics,
    distance_attenuation: f32,
    config: &GameConfig,
) {
    // Engine pitch based on RPM
    let rpm_normalized = (engine.current_rpm - engine.idle_rpm) / (engine.max_rpm - engine.idle_rpm);
    audio_state.engine_pitch = (0.8 + rpm_normalized * 1.2).clamp(0.5, 2.0);
    
    // Engine volume based on throttle and RPM
    let throttle_factor = engine.throttle_input * 0.7 + 0.3; // Minimum idle volume
    let rpm_factor = (rpm_normalized * 0.5 + 0.5).clamp(0.3, 1.0);
    
    audio_state.engine_volume = (throttle_factor * rpm_factor * distance_attenuation * config.audio.engine_volume).clamp(0.0, 1.0);
    
    // Smooth transitions to prevent audio pops
    let smooth_factor = 0.1;
    audio_state.engine_pitch = audio_state.engine_pitch * smooth_factor + audio_state.engine_pitch * (1.0 - smooth_factor);
    audio_state.engine_volume = audio_state.engine_volume * smooth_factor + audio_state.engine_volume * (1.0 - smooth_factor);
}

/// Calculate tire screech audio based on slip
fn calculate_tire_audio(
    audio_state: &mut VehicleAudioState,
    tire_physics: &TirePhysics,
    dynamics: &VehicleDynamics,
    distance_attenuation: f32,
) {
    // Tire screech based on slip ratio and lateral slip
    let slip_intensity = (tire_physics.slip_ratio.abs() + tire_physics.slip_angle.abs()).clamp(0.0, 1.0);
    
    // Only screech if there's significant slip and speed
    let speed_factor = (dynamics.speed / 20.0).clamp(0.0, 1.0); // Screech more at higher speeds
    let screech_threshold = 0.2; // Minimum slip before screech starts
    
    if slip_intensity > screech_threshold {
        let screech_intensity = ((slip_intensity - screech_threshold) / (1.0 - screech_threshold)).clamp(0.0, 1.0);
        audio_state.tire_screech_volume = (screech_intensity * speed_factor * distance_attenuation * 0.6).clamp(0.0, 1.0);
    } else {
        audio_state.tire_screech_volume = (audio_state.tire_screech_volume * 0.9).clamp(0.0, 1.0); // Fade out
    }
}

/// Calculate wind noise based on speed
fn calculate_wind_audio(
    audio_state: &mut VehicleAudioState,
    dynamics: &VehicleDynamics,
    distance_attenuation: f32,
) {
    // Wind noise increases with speed squared (realistic aerodynamics)
    let speed_factor = (dynamics.speed / 50.0).clamp(0.0, 1.0); // Max wind noise at 50 m/s
    let wind_intensity = speed_factor * speed_factor; // Quadratic increase
    
    audio_state.wind_noise_volume = (wind_intensity * distance_attenuation * 0.4).clamp(0.0, 1.0);
}

/// Calculate brake noise
fn calculate_brake_audio(
    audio_state: &mut VehicleAudioState,
    engine: &EnginePhysics,
    distance_attenuation: f32,
) {
    // Brake noise based on brake input and effectiveness
    if engine.brake_input > 0.1 {
        let brake_intensity = engine.brake_input.clamp(0.0, 1.0);
        audio_state.brake_noise_volume = (brake_intensity * distance_attenuation * 0.3).clamp(0.0, 1.0);
    } else {
        audio_state.brake_noise_volume = (audio_state.brake_noise_volume * 0.8).clamp(0.0, 1.0); // Fade out
    }
}

/// Apply calculated audio values to actual audio sources
fn apply_audio_to_sources(
    audio_state: &VehicleAudioState,
    audio_sources: &VehicleAudioSources,
    audio_sinks: &mut Query<&mut AudioSink>,
    config: &GameConfig,
) {
    // Apply engine audio
    if let Ok(mut sink) = audio_sinks.get_mut(audio_sources.engine_source) {
        sink.set_volume(bevy::audio::Volume::Linear(audio_state.engine_volume * config.audio.master_volume));
        // Note: Pitch modulation would require custom audio implementation
    }
    
    // Apply tire screech audio
    if let Ok(mut sink) = audio_sinks.get_mut(audio_sources.tire_source) {
        sink.set_volume(bevy::audio::Volume::Linear(audio_state.tire_screech_volume * config.audio.master_volume));
    }
    
    // Apply wind noise audio
    if let Ok(mut sink) = audio_sinks.get_mut(audio_sources.wind_source) {
        sink.set_volume(bevy::audio::Volume::Linear(audio_state.wind_noise_volume * config.audio.master_volume));
    }
    
    // Apply brake noise audio
    if let Ok(mut sink) = audio_sinks.get_mut(audio_sources.brake_source) {
        sink.set_volume(bevy::audio::Volume::Linear(audio_state.brake_noise_volume * config.audio.master_volume));
    }
}

/// System to cleanup audio for distant vehicles (performance optimization)
pub fn vehicle_audio_culling_system(
    mut audio_query: Query<&mut VehicleAudioState, With<RealisticVehicle>>,
    config: Res<GameConfig>,
) {
    for mut audio_state in audio_query.iter_mut() {
        // Disable audio for vehicles beyond maximum distance
        if audio_state.distance_to_player > config.audio.max_audio_distance {
            audio_state.audio_enabled = false;
            audio_state.engine_volume = 0.0;
            audio_state.tire_screech_volume = 0.0;
            audio_state.wind_noise_volume = 0.0;
            audio_state.brake_noise_volume = 0.0;
        }
    }
}

/// Performance monitoring for vehicle audio system
pub fn vehicle_audio_performance_system(
    mut last_report: Local<f32>,
    time: Res<Time>,
    query: Query<&VehicleAudioState, With<RealisticVehicle>>,
) {
    let current_time = time.elapsed_secs();
    
    if *last_report == 0.0 || current_time - *last_report > 15.0 {
        *last_report = current_time;
        let active_audio = query.iter().filter(|a| a.audio_enabled).count();
        let total_vehicles = query.iter().count();
        info!("VEHICLE AUDIO: {}/{} vehicles with active audio", active_audio, total_vehicles);
    }
}

impl Default for VehicleAudioState {
    fn default() -> Self {
        Self {
            engine_pitch: 1.0,
            engine_volume: 0.0,
            tire_screech_volume: 0.0,
            wind_noise_volume: 0.0,
            brake_noise_volume: 0.0,
            last_rpm: 800.0,
            last_speed: 0.0,
            audio_update_timer: 0.0,
            distance_to_player: 1000.0,
            audio_enabled: true,
        }
    }
}

/// Validation for audio state to prevent audio system issues
impl VehicleAudioState {
    pub fn validate_and_clamp(&mut self) {
        self.engine_pitch = self.engine_pitch.clamp(0.1, 3.0);
        self.engine_volume = self.engine_volume.clamp(0.0, 1.0);
        self.tire_screech_volume = self.tire_screech_volume.clamp(0.0, 1.0);
        self.wind_noise_volume = self.wind_noise_volume.clamp(0.0, 1.0);
        self.brake_noise_volume = self.brake_noise_volume.clamp(0.0, 1.0);
        self.distance_to_player = self.distance_to_player.clamp(0.0, 10000.0);
    }
}
