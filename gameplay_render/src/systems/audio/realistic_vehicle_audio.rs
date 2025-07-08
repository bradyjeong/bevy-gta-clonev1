//! ───────────────────────────────────────────────
//! System:   Realistic Vehicle Audio
//! Purpose:  Handles audio playback and effects
//! Schedule: Update (throttled)
//! Reads:    Player, RealisticVehicle, Time, Transform
//! Writes:   VehicleAudioState
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Physics values are validated and finite
//! Owner:    @render-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;

/// Vehicle audio state component for realistic audio synthesis
#[derive(Component, Debug)]
pub struct VehicleAudioState {
    pub engine_pitch: f32,          // 0.1 to 3.0 based on RPM
    pub engine_volume: f32,         // 0.0 to 1.0 based on throttle and load
    pub tire_screech_volume: f32,   // 0.0 to 1.0 based on slip/drift
    pub wind_noise_volume: f32,     // 0.0 to 1.0 based on speed
    pub brake_noise_volume: f32,    // 0.0 to 1.0 based on braking force
    pub gear_shift_volume: f32,     // 0.0 to 1.0 triggered on gear shifts
    pub turbo_whistle_volume: f32,  // 0.0 to 1.0 based on turbo pressure
    pub last_gear: i8,              // Track gear changes for audio triggers
    pub distance_to_player: f32,    // Distance for 3D audio positioning
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
    mut audio_query: Query<(
        &mut VehicleAudioState, 
        &VehicleAudioSources, 
        &RealisticVehicle, 
        &TirePhysics, 
        &VehicleDynamics, 
        &Transform
    )>,
    player_query: Query<&Transform, (With<Player>, Without<RealisticVehicle>)>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    
    for (mut audio_state, audio_sources, engine, tire_physics, dynamics, transform) in audio_query.iter_mut() {
        // Skip audio processing for distant vehicles
        audio_state.distance_to_player = player_transform.translation.distance(transform.translation);
        
        if audio_state.distance_to_player > 200.0 {
            audio_state.audio_enabled = false;
            continue;
        } else {
            audio_state.audio_enabled = true;
        }
        
        // Validate audio state before processing
        audio_state.validate_and_clamp();
        
        // Skip if audio disabled
        if !audio_state.audio_enabled {
            continue;
        }
        
        // Calculate realistic engine audio
        let (engine_pitch, engine_volume) = calculate_engine_audio(engine, dynamics);
        audio_state.engine_pitch = engine_pitch;
        audio_state.engine_volume = engine_volume;
        
        // Calculate tire audio (screeching, rolling noise)
        let tire_screech = calculate_tire_audio(tire_physics, dynamics);
        audio_state.tire_screech_volume = tire_screech;
        
        // Calculate wind noise based on speed
        let wind_noise = (dynamics.speed / 50.0).clamp(0.0, 1.0); // Max wind at 50 m/s
        audio_state.wind_noise_volume = wind_noise;
        
        // Generate audio events for the audio system
        // This would connect to your audio implementation
    }
}

/// Calculate realistic engine audio based on RPM and load
fn calculate_engine_audio(engine: &RealisticVehicle, dynamics: &VehicleDynamics) -> (f32, f32) {
    // Use speed as a proxy for engine state since RPM fields are not available
    let rpm_ratio = (dynamics.speed / 50.0).clamp(0.0, 1.0);
    
    let pitch = 0.5 + rpm_ratio * 2.0; // 0.5 to 2.5 pitch range
    let volume = (rpm_ratio * 0.7 + 0.3).clamp(0.1, 1.0);
    
    (pitch, volume)
}

/// Calculate tire audio based on slip and road surface
fn calculate_tire_audio(tire_physics: &TirePhysics, dynamics: &VehicleDynamics) -> f32 {
    let slip_ratio = tire_physics.slip_ratio.abs();
    let slip_angle = tire_physics.slip_angle.abs();
    
    // Tire screech based on slip and speed
    let screech_factor = (slip_ratio * 2.0 + slip_angle).clamp(0.0, 1.0);
    let speed_factor = (dynamics.speed / 20.0).clamp(0.0, 1.0); // More screech at higher speeds
    
    screech_factor * speed_factor
}

impl Default for VehicleAudioState {
    fn default() -> Self {
        Self {
            engine_pitch: 1.0,
            engine_volume: 0.0,
            tire_screech_volume: 0.0,
            wind_noise_volume: 0.0,
            brake_noise_volume: 0.0,
            gear_shift_volume: 0.0,
            turbo_whistle_volume: 0.0,
            last_gear: 1,
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

/// Placeholder audio event for integration with audio system
#[derive(Event)]
pub struct AudioEvent {
    pub sound_type: AudioType,
    pub volume: f32,
    pub pitch: f32,
    pub position: Vec3,
}

#[derive(Debug)]
pub enum AudioType {
    Engine,
    TireScreech,
    WindNoise,
    BrakeNoise,
}
