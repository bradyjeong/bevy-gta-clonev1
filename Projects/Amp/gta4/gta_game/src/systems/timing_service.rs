use bevy::prelude::*;
use std::collections::HashMap;

/// Unified timing service that manages throttling intervals across all systems
/// This replaces individual Local<Timer> patterns with a centralized approach
#[derive(Resource)]
pub struct TimingService {
    /// Global time tracking
    pub current_time: f32,
    pub delta_time: f32,
    
    /// Performance throttling intervals
    pub vehicle_lod_interval: f32,        // 0.1s - Vehicle LOD checks
    pub npc_lod_interval: f32,            // 0.1s - NPC LOD checks
    pub weather_debug_interval: f32,      // 5.0s - Weather debug logging  
    pub audio_cleanup_interval: f32,      // 1.0s - Audio entity cleanup
    pub effect_update_interval: f32,      // 0.05s - Effect state updates
    
    /// Last update times for throttled systems
    last_vehicle_lod_check: f32,
    last_npc_lod_check: f32,
    last_weather_debug: f32,
    last_audio_cleanup: f32,
    last_effect_update: f32,
    
    /// Per-entity timing tracking (replaces component-based timers)
    entity_timers: HashMap<Entity, EntityTimer>,
}

#[derive(Debug, Clone)]
pub struct EntityTimer {
    pub last_update: f32,
    pub interval: f32,
    pub timer_type: EntityTimerType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityTimerType {
    VehicleLOD,
    NPCLOD,
    FootstepAudio,
    WeatherEffect,
    Custom(String),
}

impl Default for TimingService {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            delta_time: 0.0,
            
            // Optimal intervals based on system analysis
            vehicle_lod_interval: 0.1,      // Vehicle LOD (performance critical)
            npc_lod_interval: 0.1,          // NPC LOD (performance critical)
            weather_debug_interval: 5.0,    // Weather debug (low priority)
            audio_cleanup_interval: 1.0,    // Audio cleanup (moderate)
            effect_update_interval: 0.05,   // Effects (visual smoothness)
            
            last_vehicle_lod_check: 0.0,
            last_npc_lod_check: 0.0,
            last_weather_debug: 0.0,
            last_audio_cleanup: 0.0,
            last_effect_update: 0.0,
            
            entity_timers: HashMap::new(),
        }
    }
}

impl TimingService {
    /// Update global timing (call once per frame)
    pub fn update(&mut self, time: &Time) {
        self.current_time = time.elapsed_secs();
        self.delta_time = time.delta_secs();
    }
    
    /// Check if a global system should run based on its interval
    pub fn should_run_system(&mut self, system_type: SystemType) -> bool {
        let (interval, last_check) = match system_type {
            SystemType::VehicleLOD => (self.vehicle_lod_interval, &mut self.last_vehicle_lod_check),
            SystemType::NPCLOD => (self.npc_lod_interval, &mut self.last_npc_lod_check),
            SystemType::WeatherDebug => (self.weather_debug_interval, &mut self.last_weather_debug),
            SystemType::AudioCleanup => (self.audio_cleanup_interval, &mut self.last_audio_cleanup),
            SystemType::EffectUpdate => (self.effect_update_interval, &mut self.last_effect_update),
        };
        
        if self.current_time - *last_check >= interval {
            *last_check = self.current_time;
            true
        } else {
            false
        }
    }
    
    /// Register an entity for timing tracking
    pub fn register_entity(&mut self, entity: Entity, timer_type: EntityTimerType, interval: f32) {
        self.entity_timers.insert(entity, EntityTimer {
            last_update: self.current_time,
            interval,
            timer_type,
        });
    }
    
    /// Check if an entity should update based on its individual timer
    pub fn should_update_entity(&mut self, entity: Entity) -> bool {
        if let Some(timer) = self.entity_timers.get_mut(&entity) {
            if self.current_time - timer.last_update >= timer.interval {
                timer.last_update = self.current_time;
                return true;
            }
        }
        false
    }
    
    /// Remove entity timer (call when entity is despawned)
    pub fn unregister_entity(&mut self, entity: Entity) {
        self.entity_timers.remove(&entity);
    }
    
    /// Get timing statistics for debugging
    pub fn get_stats(&self) -> TimingStats {
        TimingStats {
            tracked_entities: self.entity_timers.len(),
            current_time: self.current_time,
            delta_time: self.delta_time,
        }
    }
    
    /// Clean up stale entity timers (entities that no longer exist)
    pub fn cleanup_stale_timers(&mut self, valid_entities: &[Entity]) {
        let valid_set: std::collections::HashSet<_> = valid_entities.iter().collect();
        self.entity_timers.retain(|entity, _| valid_set.contains(entity));
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SystemType {
    VehicleLOD,
    NPCLOD,
    WeatherDebug,
    AudioCleanup,
    EffectUpdate,
}

#[derive(Debug)]
pub struct TimingStats {
    pub tracked_entities: usize,
    pub current_time: f32,
    pub delta_time: f32,
}

/// System that updates the timing service each frame
pub fn update_timing_service(
    mut timing_service: ResMut<TimingService>,
    time: Res<Time>,
) {
    timing_service.update(&time);
}

/// System that periodically cleans up stale entity timers
pub fn cleanup_timing_service(
    mut timing_service: ResMut<TimingService>,
    entity_query: Query<Entity>, // All entities
) {
    // Only clean up every 10 seconds to avoid overhead
    if timing_service.should_run_system(SystemType::AudioCleanup) {
        let valid_entities: Vec<Entity> = entity_query.iter().collect();
        timing_service.cleanup_stale_timers(&valid_entities);
        
        if cfg!(feature = "debug-timing") {
            let stats = timing_service.get_stats();
            println!("⏱️ TIMING SERVICE: {} tracked entities", stats.tracked_entities);
        }
    }
}

/// Marker component for entities that use the timing service
#[derive(Component)]
pub struct ManagedTiming {
    pub timer_type: EntityTimerType,
}

impl ManagedTiming {
    pub fn new(timer_type: EntityTimerType) -> Self {
        Self { timer_type }
    }
}
