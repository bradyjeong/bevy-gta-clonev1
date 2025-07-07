use bevy::prelude::*;
use engine_core::prelude::*;
use std::collections::HashMap;

/// Bevy resource wrapper around the core timing service
#[derive(Resource, Debug)]
pub struct BevyTimingService {
    core_service: TimingService,
    /// Map Bevy entities to core entity IDs
    entity_mapping: HashMap<Entity, u64>,
    next_entity_id: u64,
}

/// Bevy component for managed timing
#[derive(Component, Debug, Clone)]
pub struct ManagedTiming {
    pub entity_id: u64,
    pub timer_type: EntityTimerType,
}

impl Default for BevyTimingService {
    fn default() -> Self {
        Self {
            core_service: TimingService::new(),
            entity_mapping: HashMap::new(),
            next_entity_id: 1,
        }
    }
}

impl BevyTimingService {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update timing with current frame delta
    pub fn update_time(&mut self, delta_time: f32) {
        self.core_service.update_time(delta_time);
    }
    
    /// Check if a system should run this frame
    pub fn should_run_system(&mut self, system_type: SystemType) -> bool {
        self.core_service.should_run_system(system_type)
    }
    
    /// Register a Bevy entity for timing management
    pub fn register_entity(&mut self, entity: Entity, timer_type: EntityTimerType, interval: f32) {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        
        self.entity_mapping.insert(entity, entity_id);
        self.core_service.register_entity(entity_id, timer_type, interval);
    }
    
    /// Check if a Bevy entity should update this frame
    pub fn should_update_entity(&mut self, entity: Entity) -> bool {
        if let Some(&entity_id) = self.entity_mapping.get(&entity) {
            self.core_service.should_update_entity(entity_id)
        } else {
            false
        }
    }
    
    /// Unregister a Bevy entity
    pub fn unregister_entity(&mut self, entity: Entity) {
        if let Some(entity_id) = self.entity_mapping.remove(&entity) {
            self.core_service.unregister_entity(entity_id);
        }
    }
    
    /// Clean up old timers
    pub fn cleanup_old_timers(&mut self, max_age: f32) {
        self.core_service.cleanup_old_timers(max_age);
    }
    
    /// Get current time
    pub fn current_time(&self) -> f32 {
        self.core_service.current_time
    }
    
    /// Get delta time
    pub fn delta_time(&self) -> f32 {
        self.core_service.delta_time
    }
}

impl ManagedTiming {
    pub fn new(timer_type: EntityTimerType) -> Self {
        Self {
            entity_id: 0, // Will be set when registered
            timer_type,
        }
    }
}

/// System to update timing service each frame
pub fn update_timing_service(
    mut timing_service: ResMut<BevyTimingService>,
    time: Res<Time>,
) {
    timing_service.update_time(time.delta_secs());
}

/// System to cleanup despawned entities from timing service
pub fn cleanup_timing_entities(
    mut timing_service: ResMut<BevyTimingService>,
    mut removed_entities: RemovedComponents<ManagedTiming>,
) {
    for entity in removed_entities.read() {
        timing_service.unregister_entity(entity);
    }
}

/// Plugin for timing service
pub struct TimingServicePlugin;

impl Plugin for TimingServicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyTimingService>()
            .add_systems(Update, (
                update_timing_service,
                cleanup_timing_entities,
            ));
    }
}
