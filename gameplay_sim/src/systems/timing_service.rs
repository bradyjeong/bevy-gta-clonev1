//! ───────────────────────────────────────────────
//! System:   Timing Service
//! Purpose:  Manages timing intervals for systems and entities
//! Schedule: Update
//! Reads:    Time
//! Writes:   Timing state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashMap;
use game_core::prelude::*;

#[derive(Resource, Default)]
pub struct TimingService {
    current_time: f32,
    delta_time: f32,
    system_intervals: HashMap<SystemType, f32>,
    system_last_run: HashMap<SystemType, f32>,
    entity_timers: HashMap<Entity, EntityTimer>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SystemType {
    DynamicContent,
    RoadGeneration,
    Performance,
    Culling,
    LOD,
}

#[derive(Clone)]
pub struct EntityTimer {
    pub last_update: f32,
    pub interval: f32,
    pub timer_type: EntityTimerType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityTimerType {
    Movement,
    LOD,
    Culling,
    Animation,
}

impl Default for TimingService {
    fn default() -> Self {
        let mut service = Self {
            current_time: 0.0,
            delta_time: 0.0,
            system_intervals: HashMap::new(),
            system_last_run: HashMap::new(),
            entity_timers: HashMap::new(),
        };
        
        // Set default intervals
        service.system_intervals.insert(SystemType::DynamicContent, 2.0);
        service.system_intervals.insert(SystemType::RoadGeneration, 0.5);
        service.system_intervals.insert(SystemType::Performance, 5.0);
        service.system_intervals.insert(SystemType::Culling, 0.5);
        service.system_intervals.insert(SystemType::LOD, 0.2);
        
        service
    }
}

impl TimingService {
    /// Update global timing (call once per frame)
    pub fn update(&mut self, time: &Time) {
        self.current_time = time.elapsed_secs();
        self.delta_time = time.delta_secs();
    }
    
    pub fn should_run_system(&mut self, system_type: SystemType) -> bool {
        let interval = self.system_intervals.get(&system_type).copied().unwrap_or(1.0);
        let last_run = self.system_last_run.get(&system_type).copied().unwrap_or(0.0);
        
        if self.current_time - last_run >= interval {
            self.system_last_run.insert(system_type, self.current_time);
            true
        } else {
            false
        }
    }
    
    pub fn register_entity(&mut self, entity: Entity, timer_type: EntityTimerType, interval: f32) {
        let timer = EntityTimer {
            last_update: self.current_time,
            interval,
            timer_type,
        };
        self.entity_timers.insert(entity, timer);
    }
    
    pub fn should_update_entity(&mut self, entity: Entity) -> bool {
        if let Some(timer) = self.entity_timers.get_mut(&entity) {
            if self.current_time - timer.last_update >= timer.interval {
                timer.last_update = self.current_time;
                true
            } else {
                false
            }
        } else {
            true // Update if no timer registered
        }
    }
    
    pub fn unregister_entity(&mut self, entity: Entity) {
        self.entity_timers.remove(&entity);
    }
    
    pub fn set_system_interval(&mut self, system_type: SystemType, interval: f32) {
        self.system_intervals.insert(system_type, interval);
    }
    
    pub fn get_current_time(&self) -> f32 {
        self.current_time
    }
    
    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }
    
    pub fn cleanup_old_timers(&mut self, max_age: f32) {
        let current_time = self.current_time;
        self.entity_timers.retain(|_, timer| {
            current_time - timer.last_update < max_age
        });
    }
}

impl EntityTimer {
    pub fn new(timer_type: EntityTimerType, interval: f32) -> Self {
        Self {
            last_update: 0.0,
            interval,
            timer_type,
        }
    }
}

pub fn timing_service_update_system(
    mut timing_service: ResMut<TimingService>,
    time: Res<Time>,
) {
    timing_service.update(&time);
}

pub fn timing_service_cleanup_system(
    mut timing_service: ResMut<TimingService>,
) {
    // Clean up entity timers older than 60 seconds
    timing_service.cleanup_old_timers(60.0);
}
