//! Event system debug instrumentation for F3 overlay
//! 
//! Provides real-time monitoring of event flow and emission rates for debugging
//! event system performance and validating event-driven architecture compliance.

use bevy::prelude::*;
use std::collections::HashMap;

/// Event counters for debug overlay instrumentation
#[derive(Resource)]
pub struct EventCounters {
    /// Per-event type counters for current frame
    pub current_frame: HashMap<String, EventFrameStats>,
    /// Historical data for rate calculations
    pub historical: HashMap<String, EventHistoricalStats>,
    /// Total events processed this frame
    pub total_events_frame: usize,
    /// Frame counter for rate calculations
    pub frame_count: u64,
    /// Last reset time for rate calculations
    pub last_reset: std::time::Instant,
}

impl Default for EventCounters {
    fn default() -> Self {
        Self {
            current_frame: HashMap::new(),
            historical: HashMap::new(),
            total_events_frame: 0,
            frame_count: 0,
            last_reset: std::time::Instant::now(),
        }
    }
}

#[derive(Default, Clone)]
pub struct EventFrameStats {
    pub sent: usize,
    pub received: usize,
    pub processed: usize,
}

#[derive(Clone)]
pub struct EventHistoricalStats {
    pub total_sent: usize,
    pub total_received: usize,
    pub peak_rate_per_second: usize,
    pub avg_rate_per_second: f32,
    pub last_activity: std::time::Instant,
}

impl Default for EventHistoricalStats {
    fn default() -> Self {
        Self {
            total_sent: 0,
            total_received: 0,
            peak_rate_per_second: 0,
            avg_rate_per_second: 0.0,
            last_activity: std::time::Instant::now(),
        }
    }
}

impl EventCounters {
    pub fn new() -> Self {
        Self {
            current_frame: HashMap::new(),
            historical: HashMap::new(),
            total_events_frame: 0,
            frame_count: 0,
            last_reset: std::time::Instant::now(),
        }
    }

    /// Record event emission
    pub fn record_sent(&mut self, event_type: &str) {
        let stats = self.current_frame.entry(event_type.to_string()).or_default();
        stats.sent += 1;
        self.total_events_frame += 1;

        let historical = self.historical.entry(event_type.to_string()).or_default();
        historical.total_sent += 1;
        historical.last_activity = std::time::Instant::now();
    }

    /// Record event consumption
    pub fn record_received(&mut self, event_type: &str) {
        let stats = self.current_frame.entry(event_type.to_string()).or_default();
        stats.received += 1;

        let historical = self.historical.entry(event_type.to_string()).or_default();
        historical.total_received += 1;
    }

    /// Reset frame counters and update rates
    pub fn end_frame(&mut self) {
        let elapsed = self.last_reset.elapsed().as_secs_f32();
        
        if elapsed >= 1.0 {
            // Update rates every second
            for (event_type, frame_stats) in &self.current_frame {
                let historical = self.historical.entry(event_type.clone()).or_default();
                let rate = frame_stats.sent as f32 / elapsed;
                
                if rate > historical.peak_rate_per_second as f32 {
                    historical.peak_rate_per_second = rate as usize;
                }
                
                // Simple moving average
                if historical.avg_rate_per_second == 0.0 {
                    historical.avg_rate_per_second = rate;
                } else {
                    historical.avg_rate_per_second = 
                        (historical.avg_rate_per_second * 0.9) + (rate * 0.1);
                }
            }
            
            self.last_reset = std::time::Instant::now();
        }

        self.current_frame.clear();
        self.total_events_frame = 0;
        self.frame_count += 1;
    }

    /// Get debug text for F3 overlay
    pub fn get_debug_text(&self) -> String {
        let mut text = String::new();
        text.push_str("=== EVENT SYSTEM STATS ===\n");
        text.push_str(&format!("Total Events This Frame: {}\n", self.total_events_frame));
        text.push_str(&format!("Frame Count: {}\n", self.frame_count));
        text.push_str("\n--- Event Rates ---\n");

        let mut event_types: Vec<_> = self.historical.keys().collect();
        event_types.sort();

        for event_type in event_types {
            if let (Some(historical), Some(current)) = 
                (self.historical.get(event_type), self.current_frame.get(event_type)) {
                text.push_str(&format!(
                    "{}: {}/frame (avg: {:.1}/s, peak: {}/s)\n",
                    event_type,
                    current.sent,
                    historical.avg_rate_per_second,
                    historical.peak_rate_per_second
                ));
            } else if let Some(historical) = self.historical.get(event_type) {
                text.push_str(&format!(
                    "{}: 0/frame (avg: {:.1}/s, peak: {}/s)\n",
                    event_type,
                    historical.avg_rate_per_second,
                    historical.peak_rate_per_second
                ));
            }
        }

        text
    }

    /// Check for event flow issues
    pub fn get_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        for (event_type, stats) in &self.current_frame {
            // Warn about unprocessed events
            if stats.sent > 0 && stats.received == 0 {
                warnings.push(format!("Event {} sent but not received", event_type));
            }
            
            // Warn about high event rates
            if stats.sent > 100 {
                warnings.push(format!("High event rate: {} ({}/frame)", event_type, stats.sent));
            }
        }

        // Warn about total event volume
        if self.total_events_frame > 1000 {
            warnings.push(format!("Very high total event volume: {}/frame", self.total_events_frame));
        }

        warnings
    }
}

/// System to reset event counters each frame
pub fn reset_event_counters(mut counters: ResMut<EventCounters>) {
    counters.end_frame();
}

/// Macro for instrumenting EventWriter calls
#[macro_export]
macro_rules! send_instrumented {
    ($writer:expr, $event:expr, $counters:expr) => {
        {
            let event_type = std::any::type_name::<_>();
            $counters.record_sent(event_type);
            $writer.send($event);
        }
    };
}

/// Macro for instrumenting EventReader calls
#[macro_export]
macro_rules! read_instrumented {
    ($reader:expr, $counters:expr, $event_type:ty) => {
        {
            let event_type_name = std::any::type_name::<$event_type>();
            let events: Vec<_> = $reader.read().collect();
            for _ in &events {
                $counters.record_received(event_type_name);
            }
            events
        }
    };
}

// Compile-time verification
const _: () = {
    assert!(std::mem::size_of::<EventFrameStats>() <= 64);
    assert!(std::mem::size_of::<EventHistoricalStats>() <= 128);
};
