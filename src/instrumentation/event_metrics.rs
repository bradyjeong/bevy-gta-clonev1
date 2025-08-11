use bevy::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Resource)]
pub struct EventMetrics {
    pub event_counts: HashMap<&'static str, EventStats>,
    pub last_reset: Instant,
}

impl Default for EventMetrics {
    fn default() -> Self {
        Self {
            event_counts: HashMap::new(),
            last_reset: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventStats {
    pub frame_count: u32,
    pub total_count: u64,
    pub rate_per_second: f32,
    pub max_queue_size: usize,
    pub last_update: Instant,
    pub queue_ages: Vec<Duration>,
}

impl Default for EventStats {
    fn default() -> Self {
        Self {
            frame_count: 0,
            total_count: 0,
            rate_per_second: 0.0,
            max_queue_size: 0,
            last_update: Instant::now(),
            queue_ages: Vec::new(),
        }
    }
}

impl EventMetrics {
    pub fn record_event(&mut self, event_name: &'static str, count: usize) {
        let now = Instant::now();
        let stats = self.event_counts.entry(event_name).or_default();
        
        stats.frame_count = count as u32;
        stats.total_count += count as u64;
        
        // Calculate rate
        let delta = now.duration_since(stats.last_update).as_secs_f32();
        if delta > 0.0 {
            stats.rate_per_second = count as f32 / delta;
        }
        
        stats.max_queue_size = stats.max_queue_size.max(count);
        stats.last_update = now;
    }
    
    pub fn record_queue_age(&mut self, event_name: &'static str, age: Duration) {
        let stats = self.event_counts.entry(event_name).or_default();
        stats.queue_ages.push(age);
        
        // Keep only last 100 samples
        if stats.queue_ages.len() > 100 {
            stats.queue_ages.remove(0);
        }
    }
    
    pub fn get_average_queue_age(&self, event_name: &str) -> Option<Duration> {
        self.event_counts.get(event_name).and_then(|stats| {
            if stats.queue_ages.is_empty() {
                None
            } else {
                let sum: Duration = stats.queue_ages.iter().sum();
                Some(sum / stats.queue_ages.len() as u32)
            }
        })
    }
}

// Macro for instrumenting event readers
#[macro_export]
macro_rules! instrument_events {
    ($reader:expr, $event_name:literal, $metrics:expr) => {
        #[cfg(feature = "debug-events")]
        {
            let events: Vec<_> = $reader.read().collect();
            let count = events.len();
            if count > 0 {
                $metrics.record_event($event_name, count);
            }
            events
        }
        #[cfg(not(feature = "debug-events"))]
        {
            $reader.read().collect::<Vec<_>>()
        }
    };
}

pub struct EventMetricsPlugin;

impl Plugin for EventMetricsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventMetrics>()
            .init_resource::<crate::instrumentation::system_profiling::SystemMetrics>()
            .init_resource::<crate::instrumentation::system_profiling::SystemProfiler>()
            .add_systems(Update, (
                update_event_metrics,
                crate::instrumentation::system_profiling::flush_system_metrics,
            ).chain().in_set(InstrumentationSet));
        
        // Add instrumentation set that runs after all game logic
        app.configure_sets(
            Update,
            InstrumentationSet.after(GameLogicSet),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstrumentationSet;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameLogicSet;

fn update_event_metrics(
    mut metrics: ResMut<EventMetrics>,
) {
    // Reset per-frame counts
    let now = Instant::now();
    if now.duration_since(metrics.last_reset).as_secs() >= 1 {
        for stats in metrics.event_counts.values_mut() {
            stats.frame_count = 0;
        }
        metrics.last_reset = now;
    }
}

// High-traffic event instrumentation traits
pub trait InstrumentedEvent: Event {
    const NAME: &'static str;
}

// Example implementations for common events
impl InstrumentedEvent for crate::events::DynamicContentSpawned {
    const NAME: &'static str = "DynamicContentSpawned";
}

impl InstrumentedEvent for crate::events::DynamicContentDespawned {
    const NAME: &'static str = "DynamicContentDespawned";
}

// Helper system for automatic event instrumentation
pub fn instrument_event_system<E: InstrumentedEvent>(
    mut reader: EventReader<E>,
    mut metrics: ResMut<EventMetrics>,
) {
    let events: Vec<_> = reader.read().collect();
    if !events.is_empty() {
        metrics.record_event(E::NAME, events.len());
    }
}
