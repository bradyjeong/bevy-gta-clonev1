use std::collections::HashMap;

/// Unified timing service that manages throttling intervals across all systems
/// This replaces individual `Local<Timer>` patterns with a centralized approach
#[derive(Debug, Clone)]
pub struct TimingService {
    /// Global time tracking
    pub current_time: f32,
    /// Frame delta time
    pub delta_time: f32,
    
    /// Performance throttling intervals
    pub vehicle_lod_interval: f32,        // 0.1s - Vehicle LOD checks
    pub npc_lod_interval: f32,            // 0.1s - NPC LOD checks
      
    pub audio_cleanup_interval: f32,      // 1.0s - Audio entity cleanup
    pub effect_update_interval: f32,      // 0.05s - Effect state updates
    
    /// Last update times for throttled systems
    last_vehicle_lod_check: f32,
    last_npc_lod_check: f32,
    
    last_audio_cleanup: f32,
    last_effect_update: f32,
    
    /// Per-entity timing tracking (replaces component-based timers)
    entity_timers: HashMap<EntityId, EntityTimer>,
}

/// Simple entity identifier for timing purposes (not a Bevy entity)
pub type EntityId = u64;

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
    
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemType {
    VehicleLOD,
    NPCLOD,
    AudioUpdate,
    EffectUpdate,
    
    Custom(String),
}

impl Default for TimingService {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            delta_time: 0.0,
            
            vehicle_lod_interval: 0.1,
            npc_lod_interval: 0.1,
            
            audio_cleanup_interval: 1.0,
            effect_update_interval: 0.05,
            
            last_vehicle_lod_check: 0.0,
            last_npc_lod_check: 0.0,
            
            last_audio_cleanup: 0.0,
            last_effect_update: 0.0,
            
            entity_timers: HashMap::new(),
        }
    }
}

impl TimingService {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update_time(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
        self.current_time += delta_time;
    }
    
    pub fn should_run_system(&mut self, system_type: &SystemType) -> bool {
        match system_type {
            SystemType::VehicleLOD => {
                if self.current_time - self.last_vehicle_lod_check >= self.vehicle_lod_interval {
                    self.last_vehicle_lod_check = self.current_time;
                    true
                } else {
                    false
                }
            },
            SystemType::NPCLOD => {
                if self.current_time - self.last_npc_lod_check >= self.npc_lod_interval {
                    self.last_npc_lod_check = self.current_time;
                    true
                } else {
                    false
                }
            },
            SystemType::AudioUpdate => {
                if self.current_time - self.last_audio_cleanup >= self.audio_cleanup_interval {
                    self.last_audio_cleanup = self.current_time;
                    true
                } else {
                    false
                }
            },
            SystemType::EffectUpdate => {
                if self.current_time - self.last_effect_update >= self.effect_update_interval {
                    self.last_effect_update = self.current_time;
                    true
                } else {
                    false
                }
            },
            SystemType::Custom(_) => true, // Always allow custom systems
        }
    }
    
    pub fn register_entity(&mut self, entity_id: EntityId, timer_type: EntityTimerType, interval: f32) {
        let timer = EntityTimer {
            last_update: self.current_time,
            interval,
            timer_type,
        };
        self.entity_timers.insert(entity_id, timer);
    }
    
    pub fn should_update_entity(&mut self, entity_id: EntityId) -> bool {
        if let Some(timer) = self.entity_timers.get_mut(&entity_id) {
            if self.current_time - timer.last_update >= timer.interval {
                timer.last_update = self.current_time;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    
    pub fn unregister_entity(&mut self, entity_id: EntityId) {
        self.entity_timers.remove(&entity_id);
    }
    
    #[must_use]
    pub fn get_entity_timers(&self) -> &HashMap<EntityId, EntityTimer> {
        &self.entity_timers
    }
    
    pub fn cleanup_old_timers(&mut self, max_age: f32) {
        self.entity_timers.retain(|_, timer| {
            self.current_time - timer.last_update < max_age
        });
    }
}
