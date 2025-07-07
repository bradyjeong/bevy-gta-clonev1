use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use engine_core::prelude::*;
use std::time::Instant;

/// Bevy resource wrapper around core performance tracker
#[derive(Resource)]
pub struct UnifiedPerformanceTracker {
    core_tracker: PerformanceTracker,
    last_frame_time: Instant,
    frame_time_buffer: Vec<f32>,
    last_fps: f32,
}

impl Default for UnifiedPerformanceTracker {
    fn default() -> Self {
        Self {
            core_tracker: PerformanceTracker::new(),
            last_frame_time: Instant::now(),
            frame_time_buffer: Vec::with_capacity(60),
            last_fps: 0.0,
        }
    }
}

impl UnifiedPerformanceTracker {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update_frame(&mut self, delta_time: f32) {
        self.core_tracker.update_time(delta_time);
        
        // Track frame time
        let current_time = Instant::now();
        let frame_time = current_time.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;
        
        // Maintain rolling buffer for FPS calculation
        self.frame_time_buffer.push(frame_time);
        if self.frame_time_buffer.len() > 60 {
            self.frame_time_buffer.remove(0);
        }
        
        // Calculate FPS
        if !self.frame_time_buffer.is_empty() {
            let avg_frame_time: f32 = self.frame_time_buffer.iter().sum::<f32>() / self.frame_time_buffer.len() as f32;
            self.last_fps = if avg_frame_time > 0.0 { 1.0 / avg_frame_time } else { 0.0 };
        }
        
        self.core_tracker.clear_frame_stats();
    }
    
    pub fn record_category_time(&mut self, category: PerformanceCategory, time_ms: f32) {
        self.core_tracker.record_category_time(category, time_ms);
    }
    
    pub fn record_system_time(&mut self, system_name: &str, time_ms: f32) {
        self.core_tracker.record_system_time(system_name, time_ms);
    }
    
    pub fn update_entity_counts(&mut self, total: usize, active: usize, culled: usize) {
        self.core_tracker.update_entity_counts(total, active, culled);
    }
    
    pub fn record_cache_hit(&mut self) {
        self.core_tracker.record_cache_hit();
    }
    
    pub fn record_cache_miss(&mut self) {
        self.core_tracker.record_cache_miss();
    }
    
    pub fn update_cache_stats(&mut self, entries: usize, memory: usize) {
        self.core_tracker.update_cache_stats(entries, memory);
    }
    
    pub fn add_alert(&mut self, category: PerformanceCategory, message: String, severity: AlertSeverity) {
        self.core_tracker.add_alert(category, message, severity);
    }
    
    pub fn get_fps(&self) -> f32 {
        self.last_fps
    }
    
    pub fn get_frame_time_ms(&self) -> f32 {
        if !self.frame_time_buffer.is_empty() {
            let avg: f32 = self.frame_time_buffer.iter().sum::<f32>() / self.frame_time_buffer.len() as f32;
            avg * 1000.0
        } else {
            0.0
        }
    }
    
    pub fn get_cache_hit_rate(&self) -> f32 {
        self.core_tracker.get_cache_hit_rate()
    }
    
    pub fn enable(&mut self) {
        self.core_tracker.enable();
    }
    
    pub fn disable(&mut self) {
        self.core_tracker.disable();
    }
    
    // Delegate to core tracker
    pub fn get_category_metrics(&self, category: PerformanceCategory) -> Option<&CategoryMetrics> {
        self.core_tracker.get_category_metrics(category)
    }
    
    pub fn get_system_timing(&self, system_name: &str) -> Option<&SystemTiming> {
        self.core_tracker.get_system_timing(system_name)
    }
}

/// System to update performance tracker each frame
pub fn update_performance_tracker(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    time: Res<Time>,
) {
    tracker.update_frame(time.delta_secs());
}

/// System to monitor Bevy diagnostics
pub fn monitor_bevy_diagnostics(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    diagnostics: Res<DiagnosticsStore>,
) {
    // Monitor frame time from Bevy's diagnostics
    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diagnostic.smoothed() {
            // Track rendering performance
            let frame_time_ms = if fps > 0.0 { 1000.0 / fps } else { 0.0 };
            tracker.record_category_time(PerformanceCategory::Rendering, frame_time_ms as f32);
        }
    }
}

/// Plugin for performance monitoring
pub struct PerformanceServicePlugin;

impl Plugin for PerformanceServicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnifiedPerformanceTracker>()
            .add_systems(Update, (
                update_performance_tracker,
                monitor_bevy_diagnostics,
            ));
    }
}
