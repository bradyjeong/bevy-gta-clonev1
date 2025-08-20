use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Unified performance tracking system - centralizes all performance metrics
#[derive(Resource)]
pub struct UnifiedPerformanceTracker {
    pub categories: HashMap<PerformanceCategory, CategoryMetrics>,
    pub frame_analyzer: FrameTimeAnalyzer,
    pub memory_tracker: MemoryTracker,
    pub system_timings: HashMap<String, SystemTiming>,
    pub bottleneck_detector: BottleneckDetector,
    pub cache_stats: CacheStats,
    pub entity_counters: EntityCounters,
    pub alerts: Vec<PerformanceAlert>,
    pub last_report: Instant,
    pub report_interval: Duration,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerformanceCategory {
    Physics,
    Rendering,
    Culling,
    Input,
    Audio,
    Spawning,
    LOD,
    Batching,
    Transform,
    UI,
    Network,
    System,
}

#[derive(Default)]
pub struct CategoryMetrics {
    pub execution_time_ms: f32,
    pub entity_count: usize,
    pub operations_per_frame: usize,
    pub memory_usage_bytes: usize,
    pub peak_execution_time: f32,
    pub avg_execution_time: f32,
    pub frame_count: u64,
    pub total_execution_time: f64,
}

pub struct FrameTimeAnalyzer {
    pub frame_times: VecDeque<f32>,
    pub fps_history: VecDeque<f32>,
    pub target_fps: f32,
    pub avg_frame_time: f32,
    pub min_frame_time: f32,
    pub max_frame_time: f32,
    pub frame_time_variance: f32,
    pub spike_count: usize,
    pub consistency_score: f32,
    pub last_spike_time: Instant,
    pub frame_spike_threshold: f32,
    pub consecutive_slow_frames: usize,
}

impl Default for FrameTimeAnalyzer {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            fps_history: VecDeque::with_capacity(120),
            target_fps: 60.0,
            avg_frame_time: 0.0,
            min_frame_time: 0.0,
            max_frame_time: 0.0,
            frame_time_variance: 0.0,
            spike_count: 0,
            consistency_score: 0.0,
            last_spike_time: Instant::now(),
            frame_spike_threshold: 20.0, // 20ms spike threshold
            consecutive_slow_frames: 0,
        }
    }
}

#[derive(Default)]
pub struct MemoryTracker {
    pub entity_memory: usize,
    pub component_memory: usize,
    pub system_memory: usize,
    pub asset_memory: usize,
    pub total_allocated: usize,
    pub peak_memory: usize,
    pub memory_pressure: f32,
    pub gc_collections: usize,
}

#[derive(Default)]
pub struct SystemTiming {
    pub last_execution_time: f32,
    pub avg_execution_time: f32,
    pub peak_execution_time: f32,
    pub execution_count: u64,
    pub total_time: f64,
    pub is_bottleneck: bool,
}

#[derive(Default)]
pub struct BottleneckDetector {
    pub critical_systems: Vec<String>,
    pub bottleneck_threshold_ms: f32,
    pub frame_spike_threshold: f32,
    pub consecutive_slow_frames: usize,
    pub bottleneck_history: VecDeque<BottleneckEvent>,
}

#[derive(Clone)]
pub struct BottleneckEvent {
    pub system_name: String,
    pub execution_time: f32,
    pub timestamp: Instant,
    pub severity: AlertSeverity,
}

#[derive(Default)]
pub struct CacheStats {
    pub distance_cache_hits: usize,
    pub distance_cache_misses: usize,
    pub distance_cache_size: usize,
    pub asset_cache_hits: usize,
    pub asset_cache_misses: usize,
    pub lod_cache_hits: usize,
    pub lod_cache_misses: usize,
    pub total_cache_memory: usize,
}

#[derive(Default)]
pub struct EntityCounters {
    pub total_entities: usize,
    pub active_entities: usize,
    pub culled_entities: usize,
    pub spawned_this_frame: usize,
    pub despawned_this_frame: usize,
    pub entities_by_type: HashMap<String, usize>,
    pub lod_distribution: HashMap<String, usize>,
}

#[derive(Clone)]
pub struct PerformanceAlert {
    pub category: PerformanceCategory,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: Instant,
    pub value: f32,
    pub threshold: f32,
}

#[derive(Clone, Debug)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl Default for UnifiedPerformanceTracker {
    fn default() -> Self {
        let mut categories = HashMap::new();
        for category in [
            PerformanceCategory::Physics,
            PerformanceCategory::Rendering,
            PerformanceCategory::Culling,
            PerformanceCategory::Input,
            PerformanceCategory::Audio,
            PerformanceCategory::Spawning,
            PerformanceCategory::LOD,
            PerformanceCategory::Batching,
            PerformanceCategory::Transform,
            PerformanceCategory::UI,
            PerformanceCategory::Network,
            PerformanceCategory::System,
        ] {
            categories.insert(category, CategoryMetrics::default());
        }

        Self {
            categories,
            frame_analyzer: FrameTimeAnalyzer::default(),
            memory_tracker: MemoryTracker::default(),
            system_timings: HashMap::new(),
            bottleneck_detector: BottleneckDetector {
                bottleneck_threshold_ms: 5.0, // 5ms threshold
                frame_spike_threshold: 20.0, // 20ms spike threshold
                bottleneck_history: VecDeque::with_capacity(100),
                ..default()
            },
            cache_stats: CacheStats::default(),
            entity_counters: EntityCounters::default(),
            alerts: Vec::new(),
            last_report: Instant::now(),
            report_interval: Duration::from_secs(10),
            enabled: true,
        }
    }
}

impl UnifiedPerformanceTracker {
    /// Record execution time for a specific category
    pub fn record_category_time(&mut self, category: PerformanceCategory, time_ms: f32) {
        if !self.enabled { return; }
        
        if let Some(metrics) = self.categories.get_mut(&category) {
            metrics.execution_time_ms = time_ms;
            metrics.frame_count += 1;
            metrics.total_execution_time += time_ms as f64;
            metrics.avg_execution_time = (metrics.total_execution_time / metrics.frame_count as f64) as f32;
            
            if time_ms > metrics.peak_execution_time {
                metrics.peak_execution_time = time_ms;
            }
            
            // Check for performance alerts
            if time_ms > 10.0 {
                self.add_alert(PerformanceAlert {
                    category,
                    severity: AlertSeverity::Warning,
                    message: format!("{:?} system slow: {:.2}ms", category, time_ms),
                    timestamp: Instant::now(),
                    value: time_ms,
                    threshold: 10.0,
                });
            }
        }
    }

    /// Record system execution time
    pub fn record_system_time(&mut self, system_name: &str, time_ms: f32) {
        if !self.enabled { return; }
        
        let timing = self.system_timings.entry(system_name.to_string()).or_insert(SystemTiming::default());
        timing.last_execution_time = time_ms;
        timing.execution_count += 1;
        timing.total_time += time_ms as f64;
        timing.avg_execution_time = (timing.total_time / timing.execution_count as f64) as f32;
        
        if time_ms > timing.peak_execution_time {
            timing.peak_execution_time = time_ms;
        }
        
        // Check if system is a bottleneck
        if time_ms > self.bottleneck_detector.bottleneck_threshold_ms {
            timing.is_bottleneck = true;
            self.bottleneck_detector.bottleneck_history.push_back(BottleneckEvent {
                system_name: system_name.to_string(),
                execution_time: time_ms,
                timestamp: Instant::now(),
                severity: if time_ms > 15.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
            });
            
            if self.bottleneck_detector.bottleneck_history.len() > 100 {
                self.bottleneck_detector.bottleneck_history.pop_front();
            }
        } else {
            timing.is_bottleneck = false;
        }
    }

    /// Update frame timing analysis
    pub fn update_frame_timing(&mut self, frame_time_ms: f32, fps: f32) {
        if !self.enabled { return; }
        
        let analyzer = &mut self.frame_analyzer;
        
        // Add to history
        analyzer.frame_times.push_back(frame_time_ms);
        analyzer.fps_history.push_back(fps);
        
        // Maintain history size
        if analyzer.frame_times.len() > 120 {
            analyzer.frame_times.pop_front();
        }
        if analyzer.fps_history.len() > 120 {
            analyzer.fps_history.pop_front();
        }
        
        // Calculate statistics
        if !analyzer.frame_times.is_empty() {
            analyzer.avg_frame_time = analyzer.frame_times.iter().sum::<f32>() / analyzer.frame_times.len() as f32;
            analyzer.min_frame_time = analyzer.frame_times.iter().cloned().fold(f32::INFINITY, f32::min);
            analyzer.max_frame_time = analyzer.frame_times.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            
            // Calculate variance for consistency score
            let variance = analyzer.frame_times.iter()
                .map(|&x| (x - analyzer.avg_frame_time).powi(2))
                .sum::<f32>() / analyzer.frame_times.len() as f32;
            analyzer.frame_time_variance = variance.sqrt();
            analyzer.consistency_score = 1.0 / (1.0 + analyzer.frame_time_variance * 0.1);
        }
        
        // Collect alerts to add after analyzer updates
        let mut alerts_to_add = Vec::new();
        
        // Detect frame spikes
        let frame_spike_threshold = analyzer.frame_spike_threshold;
        if frame_time_ms > frame_spike_threshold {
            analyzer.spike_count += 1;
            analyzer.last_spike_time = Instant::now();
            
            alerts_to_add.push(PerformanceAlert {
                category: PerformanceCategory::System,
                severity: AlertSeverity::Warning,
                message: format!("Frame spike detected: {:.2}ms", frame_time_ms),
                timestamp: Instant::now(),
                value: frame_time_ms,
                threshold: frame_spike_threshold,
            });
        }
        
        // Check for sustained low FPS
        let target_fps = analyzer.target_fps;
        if fps < target_fps * 0.8 {
            analyzer.consecutive_slow_frames += 1;
            if analyzer.consecutive_slow_frames > 30 { // 0.5 seconds of slow frames
                alerts_to_add.push(PerformanceAlert {
                    category: PerformanceCategory::System,
                    severity: AlertSeverity::Critical,
                    message: format!("Sustained low FPS: {:.1} (target: {:.1})", fps, target_fps),
                    timestamp: Instant::now(),
                    value: fps,
                    threshold: target_fps * 0.8,
                });
                analyzer.consecutive_slow_frames = 0; // Reset to avoid spam
            }
        } else {
            analyzer.consecutive_slow_frames = 0;
        }
        
        // Add collected alerts
        for alert in alerts_to_add {
            self.add_alert(alert);
        }
    }

    /// Update memory tracking
    pub fn update_memory_usage(&mut self, entity_memory: usize, system_memory: usize, total_memory: usize) {
        if !self.enabled { return; }
        
        self.memory_tracker.entity_memory = entity_memory;
        self.memory_tracker.system_memory = system_memory;
        self.memory_tracker.total_allocated = total_memory;
        
        if total_memory > self.memory_tracker.peak_memory {
            self.memory_tracker.peak_memory = total_memory;
        }
        
        // Calculate memory pressure (simplified)
        let memory_pressure = total_memory as f32 / (1024.0 * 1024.0 * 1024.0); // GB
        self.memory_tracker.memory_pressure = memory_pressure;
        
        // Alert on high memory usage
        if memory_pressure > 2.0 {
            self.add_alert(PerformanceAlert {
                category: PerformanceCategory::System,
                severity: AlertSeverity::Warning,
                message: format!("High memory usage: {:.1} GB", memory_pressure),
                timestamp: Instant::now(),
                value: memory_pressure,
                threshold: 2.0,
            });
        }
    }

    /// Update cache statistics
    pub fn update_cache_stats(&mut self, hits: usize, misses: usize, cache_type: &str) {
        if !self.enabled { return; }
        
        match cache_type {
            "distance" => {
                self.cache_stats.distance_cache_hits += hits;
                self.cache_stats.distance_cache_misses += misses;
            },
            "asset" => {
                self.cache_stats.asset_cache_hits += hits;
                self.cache_stats.asset_cache_misses += misses;
            },
            "lod" => {
                self.cache_stats.lod_cache_hits += hits;
                self.cache_stats.lod_cache_misses += misses;
            },
            _ => {}
        }
    }

    /// Update entity counters
    pub fn update_entity_counts(&mut self, total: usize, active: usize, culled: usize) {
        if !self.enabled { return; }
        
        let counters = &mut self.entity_counters;
        counters.total_entities = total;
        counters.active_entities = active;
        counters.culled_entities = culled;
    }

    /// Add performance alert
    pub fn add_alert(&mut self, alert: PerformanceAlert) {
        self.alerts.push(alert);
        
        // Keep only recent alerts (last 5 minutes)
        self.alerts.retain(|a| a.timestamp.elapsed() < Duration::from_secs(300));
    }

    /// Get cache hit ratio
    pub fn get_cache_hit_ratio(&self, cache_type: &str) -> f32 {
        let (hits, misses) = match cache_type {
            "distance" => (self.cache_stats.distance_cache_hits, self.cache_stats.distance_cache_misses),
            "asset" => (self.cache_stats.asset_cache_hits, self.cache_stats.asset_cache_misses),
            "lod" => (self.cache_stats.lod_cache_hits, self.cache_stats.lod_cache_misses),
            _ => return 0.0,
        };
        
        if hits + misses == 0 {
            0.0
        } else {
            hits as f32 / (hits + misses) as f32
        }
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            avg_fps: if !self.frame_analyzer.fps_history.is_empty() {
                self.frame_analyzer.fps_history.iter().sum::<f32>() / self.frame_analyzer.fps_history.len() as f32
            } else { 0.0 },
            avg_frame_time: self.frame_analyzer.avg_frame_time,
            consistency_score: self.frame_analyzer.consistency_score,
            total_entities: self.entity_counters.total_entities,
            culled_entities: self.entity_counters.culled_entities,
            memory_usage_gb: self.memory_tracker.memory_pressure,
            active_alerts: self.alerts.len(),
            bottleneck_systems: self.system_timings.iter()
                .filter(|(_, timing)| timing.is_bottleneck)
                .map(|(name, _)| name.clone())
                .collect(),
        }
    }

    /// Generate detailed performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("\n🎯 UNIFIED PERFORMANCE MONITOR REPORT 🎯\n");
        report.push_str("===============================================\n");
        
        // Frame timing
        if !self.frame_analyzer.fps_history.is_empty() {
            let avg_fps = self.frame_analyzer.fps_history.iter().sum::<f32>() / self.frame_analyzer.fps_history.len() as f32;
            report.push_str(&format!("📊 Frame Analysis:\n"));
            report.push_str(&format!("   Current FPS: {:.1} | Target: {:.1}\n", 
                self.frame_analyzer.fps_history.back().unwrap_or(&0.0), self.frame_analyzer.target_fps));
            report.push_str(&format!("   Average FPS: {:.1} | Frame Time: {:.2}ms\n", avg_fps, self.frame_analyzer.avg_frame_time));
            report.push_str(&format!("   Consistency: {:.1}% | Spikes: {}\n", 
                self.frame_analyzer.consistency_score * 100.0, self.frame_analyzer.spike_count));
        }
        
        // System performance
        if !self.system_timings.is_empty() {
            report.push_str("\n⚡ System Performance:\n");
            let mut systems: Vec<_> = self.system_timings.iter().collect();
            systems.sort_by(|a, b| b.1.avg_execution_time.partial_cmp(&a.1.avg_execution_time).unwrap());
            
            for (name, timing) in systems.iter().take(5) {
                let status = if timing.is_bottleneck { "🚨" } else { "✅" };
                report.push_str(&format!("   {} {}: {:.2}ms avg (peak: {:.2}ms)\n", 
                    status, name, timing.avg_execution_time, timing.peak_execution_time));
            }
        }
        
        // Category performance
        report.push_str("\n📈 Category Performance:\n");
        for (category, metrics) in &self.categories {
            if metrics.frame_count > 0 {
                report.push_str(&format!("   {:?}: {:.2}ms avg | {} entities | {:.1} MB\n",
                    category, metrics.avg_execution_time, metrics.entity_count, 
                    metrics.memory_usage_bytes as f32 / 1024.0 / 1024.0));
            }
        }
        
        // Memory usage
        report.push_str(&format!("\n💾 Memory Usage:\n"));
        report.push_str(&format!("   Total: {:.1} GB | Peak: {:.1} GB\n", 
            self.memory_tracker.memory_pressure, self.memory_tracker.peak_memory as f32 / 1024.0 / 1024.0 / 1024.0));
        report.push_str(&format!("   Entities: {:.1} MB | Systems: {:.1} MB\n",
            self.memory_tracker.entity_memory as f32 / 1024.0 / 1024.0,
            self.memory_tracker.system_memory as f32 / 1024.0 / 1024.0));
        
        // Cache performance
        report.push_str("\n🗄️ Cache Performance:\n");
        report.push_str(&format!("   Distance Cache: {:.1}% hit rate\n", self.get_cache_hit_ratio("distance") * 100.0));
        report.push_str(&format!("   Asset Cache: {:.1}% hit rate\n", self.get_cache_hit_ratio("asset") * 100.0));
        report.push_str(&format!("   LOD Cache: {:.1}% hit rate\n", self.get_cache_hit_ratio("lod") * 100.0));
        
        // Entity statistics
        report.push_str(&format!("\n🎮 Entity Statistics:\n"));
        report.push_str(&format!("   Total: {} | Active: {} | Culled: {}\n",
            self.entity_counters.total_entities, self.entity_counters.active_entities, self.entity_counters.culled_entities));
        
        // Active alerts
        if !self.alerts.is_empty() {
            report.push_str("\n⚠️ Active Alerts:\n");
            for alert in self.alerts.iter().rev().take(5) {
                let severity_icon = match alert.severity {
                    AlertSeverity::Info => "ℹ️",
                    AlertSeverity::Warning => "⚠️",
                    AlertSeverity::Critical => "🚨",
                    AlertSeverity::Emergency => "🆘",
                };
                report.push_str(&format!("   {} {}: {}\n", severity_icon, format!("{:?}", alert.category), alert.message));
            }
        }
        
        report.push_str("\n===============================================\n");
        report
    }
}

#[derive(Debug)]
pub struct PerformanceSummary {
    pub avg_fps: f32,
    pub avg_frame_time: f32,
    pub consistency_score: f32,
    pub total_entities: usize,
    pub culled_entities: usize,
    pub memory_usage_gb: f32,
    pub active_alerts: usize,
    pub bottleneck_systems: Vec<String>,
}

/// System to update the unified performance tracker
pub fn unified_performance_monitoring_system(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    diagnostics: Res<DiagnosticsStore>,
    _time: Res<Time>,
) {
    if !tracker.enabled {
        return;
    }
    
    // Update frame timing from diagnostics
    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diagnostic.smoothed() {
            let frame_time_ms = 1000.0 / fps as f32;
            tracker.update_frame_timing(frame_time_ms, fps as f32);
        }
    }
    
    // Generate periodic reports
    if tracker.last_report.elapsed() > tracker.report_interval {
        let report = tracker.generate_report();
        #[cfg(debug_assertions)]
        println!("{}", report);
        tracker.last_report = Instant::now();
    }
}

/// Debug UI component for F3 overlay
#[derive(Component)]
pub struct PerformanceOverlay;

/// System to handle F3 debug key toggle
pub fn performance_debug_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay_query: Query<&mut Visibility, With<PerformanceOverlay>>,
    mut commands: Commands,
    tracker: Res<UnifiedPerformanceTracker>,
) {
    if keys.just_pressed(KeyCode::F3) {
        // Toggle overlay visibility
        if let Ok(mut visibility) = overlay_query.single_mut() {
            *visibility = if *visibility == Visibility::Visible {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        } else {
            // Create overlay if it doesn't exist
            spawn_performance_overlay(&mut commands, &tracker);
        }
    }
}

/// Spawn the performance overlay UI
fn spawn_performance_overlay(commands: &mut Commands, tracker: &UnifiedPerformanceTracker) {
    let summary = tracker.get_performance_summary();
    
    commands.spawn((
        Text::new(format!(
            "Performance Monitor (F3)\n\
            FPS: {:.1} | Frame: {:.2}ms\n\
            Entities: {} (Culled: {})\n\
            Memory: {:.1} GB\n\
            Alerts: {} | Bottlenecks: {}",
            summary.avg_fps,
            summary.avg_frame_time,
            summary.total_entities,
            summary.culled_entities,
            summary.memory_usage_gb,
            summary.active_alerts,
            summary.bottleneck_systems.len()
        )),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // Yellow text
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        PerformanceOverlay,
    ));
}

/// System to update the performance overlay
pub fn update_performance_overlay_system(
    tracker: Res<UnifiedPerformanceTracker>,
    mut overlay_query: Query<&mut Text, With<PerformanceOverlay>>,
) {
    if let Ok(mut text) = overlay_query.single_mut() {
        let summary = tracker.get_performance_summary();
        
        text.0 = format!(
            "Performance Monitor (F3)\n\
            FPS: {:.1} | Frame: {:.2}ms | Consistency: {:.1}%\n\
            Entities: {} (Active: {}, Culled: {})\n\
            Memory: {:.1} GB\n\
            Cache Hit Rates: D:{:.1}% A:{:.1}% L:{:.1}%\n\
            Alerts: {} | Bottlenecks: {}",
            summary.avg_fps,
            summary.avg_frame_time,
            summary.consistency_score * 100.0,
            summary.total_entities,
            tracker.entity_counters.active_entities,
            summary.culled_entities,
            summary.memory_usage_gb,
            tracker.get_cache_hit_ratio("distance") * 100.0,
            tracker.get_cache_hit_ratio("asset") * 100.0,
            tracker.get_cache_hit_ratio("lod") * 100.0,
            summary.active_alerts,
            summary.bottleneck_systems.len()
        );
    }
}

/// Macro for easy performance timing
#[macro_export]
macro_rules! time_system {
    ($tracker:expr, $category:expr, $system_name:expr, $code:block) => {
        {
            let start = std::time::Instant::now();
            let result = $code;
            let elapsed = start.elapsed().as_secs_f32() * 1000.0;
            $tracker.record_category_time($category, elapsed);
            $tracker.record_system_time($system_name, elapsed);
            result
        }
    };
}

/// Plugin for unified performance monitoring
pub struct UnifiedPerformancePlugin;

impl Plugin for UnifiedPerformancePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnifiedPerformanceTracker>()
            .add_systems(Update, (
                unified_performance_monitoring_system,
                performance_debug_input_system,
                update_performance_overlay_system,
            ));
    }
}
