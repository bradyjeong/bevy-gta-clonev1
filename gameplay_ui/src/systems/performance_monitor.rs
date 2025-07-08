use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Critical
    Critical,
    /// Emergency
    Emergency,
}

/// Performance categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerformanceCategory {
    /// Physics
    Physics,
    /// Rendering
    Rendering,
    /// Culling
    Culling,
    /// Input
    Input,
    /// Audio
    Audio,
    /// Spawning
    Spawning,
    /// LOD
    LOD,
    /// Batching
    Batching,
    /// Transform
    Transform,
    /// UI
    UI,
    /// Network
    Network,
    /// System
    System,
    /// Memory
    Memory,
    /// GPU
    GPU,
}

/// Performance alert
pub struct PerformanceAlert {
    /// Category
    pub category: PerformanceCategory,
    /// Severity
    pub severity: AlertSeverity,
    /// Message
    pub message: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Value
    pub value: f32,
    /// Threshold
    pub threshold: f32,
}

/// Category metrics
#[derive(Default)]
pub struct CategoryMetrics {
    /// Execution time in milliseconds
    pub execution_time_ms: f32,
    /// Entity count
    pub entity_count: usize,
    /// Operations per frame
    pub operations_per_frame: usize,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Peak execution time
    pub peak_execution_time: f32,
    /// Average execution time
    pub avg_execution_time: f32,
    /// Frame count
    pub frame_count: u64,
    /// Total execution time
    pub total_execution_time: f64,
}

/// Frame time analyzer
pub struct FrameTimeAnalyzer {
    /// Frame times history
    pub frame_times: VecDeque<f32>,
    /// FPS history
    pub fps_history: VecDeque<f32>,
    /// Target FPS
    pub target_fps: f32,
    /// Average frame time
    pub avg_frame_time: f32,
    /// Minimum frame time
    pub min_frame_time: f32,
    /// Maximum frame time
    pub max_frame_time: f32,
    /// Frame time variance
    pub frame_time_variance: f32,
    /// Spike count
    pub spike_count: usize,
    /// Consistency score
    pub consistency_score: f32,
    /// Last spike time
    pub last_spike_time: Instant,
    /// Frame spike threshold
    pub frame_spike_threshold: f32,
    /// Consecutive slow frames
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
            frame_spike_threshold: 20.0,
            consecutive_slow_frames: 0,
        }
    }
}

/// Memory tracker
#[derive(Default)]
pub struct MemoryTracker {
    /// Entity memory usage
    pub entity_memory: usize,
    /// Component memory usage
    pub component_memory: usize,
    /// System memory usage
    pub system_memory: usize,
    /// Asset memory usage
    pub asset_memory: usize,
    /// Total allocated memory
    pub total_allocated: usize,
    /// Peak memory usage
    pub peak_memory: usize,
    /// Memory pressure
    pub memory_pressure: f32,
    /// GC collections
    pub gc_collections: usize,
}

/// System timing
#[derive(Default)]
pub struct SystemTiming {
    /// Last execution time
    pub last_execution_time: f32,
    /// Execution count
    pub execution_count: u64,
    /// Total time
    pub total_time: f64,
    /// Is bottleneck
    pub is_bottleneck: bool,
}

/// Bottleneck detector
#[derive(Default)]
pub struct BottleneckDetector {
    /// Critical systems
    pub critical_systems: Vec<String>,
    /// Bottleneck threshold in milliseconds
    pub bottleneck_threshold_ms: f32,
    /// Bottleneck history
    pub bottleneck_history: VecDeque<BottleneckEvent>,
}

/// Bottleneck event
#[derive(Clone)]
pub struct BottleneckEvent {
    /// System name
    pub system_name: String,
    /// Execution time
    pub execution_time: f32,
    /// Timestamp
    pub timestamp: Instant,
    /// Severity
    pub severity: AlertSeverity,
}

/// Cache statistics
#[derive(Default)]
pub struct CacheStats {
    /// Distance cache hits
    pub distance_cache_hits: usize,
    /// Distance cache misses
    pub distance_cache_misses: usize,
    /// Distance cache size
    pub distance_cache_size: usize,
    /// Asset cache hits
    pub asset_cache_hits: usize,
    /// Asset cache misses
    pub asset_cache_misses: usize,
    /// LOD cache hits
    pub lod_cache_hits: usize,
    /// LOD cache misses
    pub lod_cache_misses: usize,
}

/// Entity counters
#[derive(Default)]
pub struct EntityCounters {
    /// Total entities
    pub total_entities: usize,
    /// Active entities
    pub active_entities: usize,
    /// Culled entities
    pub culled_entities: usize,
    /// Spawned this frame
    pub spawned_this_frame: usize,
    /// Destroyed this frame
    pub destroyed_this_frame: usize,
    /// Dirty entities
    pub dirty_entities: usize,
}

/// Performance summary
pub struct PerformanceSummary {
    /// Average FPS
    pub avg_fps: f32,
    /// Average frame time
    pub avg_frame_time: f32,
    /// Consistency score
    pub consistency_score: f32,
    /// Total entities
    pub total_entities: usize,
    /// Active entities
    pub active_entities: usize,
    /// Culled entities
    pub culled_entities: usize,
    /// Memory usage in GB
    pub memory_usage_gb: f32,
    /// Cache hit ratio
    pub cache_hit_ratio: f32,
}

/// Unified performance tracking system
#[derive(Resource)]
pub struct UnifiedPerformanceTracker {
    /// Categories
    pub categories: HashMap<PerformanceCategory, CategoryMetrics>,
    /// Frame analyzer
    pub frame_analyzer: FrameTimeAnalyzer,
    /// Memory tracker
    pub memory_tracker: MemoryTracker,
    /// System timings
    pub system_timings: HashMap<String, SystemTiming>,
    /// Bottleneck detector
    pub bottleneck_detector: BottleneckDetector,
    /// Cache stats
    pub cache_stats: CacheStats,
    /// Entity counters
    pub entity_counters: EntityCounters,
    /// Alerts
    pub alerts: Vec<PerformanceAlert>,
    /// Last report timestamp
    pub last_report: Instant,
    /// Report interval
    pub report_interval: Duration,
    /// Enabled
    pub enabled: bool,
}

impl Default for UnifiedPerformanceTracker {
    fn default() -> Self {
        let mut categories = HashMap::new();
        categories.insert(PerformanceCategory::Physics, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Rendering, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Culling, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Input, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Audio, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Spawning, CategoryMetrics::default());
        categories.insert(PerformanceCategory::LOD, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Batching, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Transform, CategoryMetrics::default());
        categories.insert(PerformanceCategory::UI, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Network, CategoryMetrics::default());
        categories.insert(PerformanceCategory::System, CategoryMetrics::default());
        categories.insert(PerformanceCategory::Memory, CategoryMetrics::default());
        categories.insert(PerformanceCategory::GPU, CategoryMetrics::default());
        
        Self {
            categories,
            frame_analyzer: FrameTimeAnalyzer::default(),
            memory_tracker: MemoryTracker::default(),
            system_timings: HashMap::new(),
            bottleneck_detector: BottleneckDetector::default(),
            cache_stats: CacheStats::default(),
            entity_counters: EntityCounters::default(),
            alerts: Vec::new(),
            last_report: Instant::now(),
            report_interval: Duration::from_secs(5),
            enabled: true,
        }
    }
}

impl UnifiedPerformanceTracker {
    /// Record category execution time
    pub fn record_category_time(&mut self, category: PerformanceCategory, time_ms: f32) {
        if let Some(metrics) = self.categories.get_mut(&category) {
            metrics.execution_time_ms = time_ms;
            metrics.frame_count += 1;
            metrics.total_execution_time += time_ms as f64;
            metrics.avg_execution_time = (metrics.total_execution_time / metrics.frame_count as f64) as f32;
            
            if time_ms > metrics.peak_execution_time {
                metrics.peak_execution_time = time_ms;
            }
        }
    }
    
    /// Record system timing
    pub fn record_system_time(&mut self, system_name: &str, time_ms: f32) {
        let timing = self.system_timings.entry(system_name.to_string()).or_default();
        timing.last_execution_time = time_ms;
        timing.execution_count += 1;
        timing.total_time += time_ms as f64;
        timing.is_bottleneck = time_ms > self.bottleneck_detector.bottleneck_threshold_ms;
    }
    
    /// Update frame timing
    pub fn update_frame_timing(&mut self, frame_time_ms: f32, fps: f32) {
        let analyzer = &mut self.frame_analyzer;
        
        analyzer.frame_times.push_back(frame_time_ms);
        analyzer.fps_history.push_back(fps);
        
        if analyzer.frame_times.len() > 120 {
            analyzer.frame_times.pop_front();
        }
        if analyzer.fps_history.len() > 120 {
            analyzer.fps_history.pop_front();
        }
        
        // Calculate statistics
        analyzer.avg_frame_time = analyzer.frame_times.iter().sum::<f32>() / analyzer.frame_times.len() as f32;
        analyzer.min_frame_time = analyzer.frame_times.iter().cloned().fold(f32::INFINITY, f32::min);
        analyzer.max_frame_time = analyzer.frame_times.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        
        // Check for spikes
        if frame_time_ms > analyzer.frame_spike_threshold {
            analyzer.spike_count += 1;
            analyzer.last_spike_time = Instant::now();
        }
        
        // Calculate consistency score
        let variance = analyzer.frame_times.iter()
            .map(|&x| (x - analyzer.avg_frame_time).powi(2))
            .sum::<f32>() / analyzer.frame_times.len() as f32;
        analyzer.frame_time_variance = variance.sqrt();
        analyzer.consistency_score = 1.0 / (1.0 + analyzer.frame_time_variance * 0.1);
    }
    
    /// Update entity counts
    pub fn update_entity_counts(&mut self, total: usize, active: usize, culled: usize) {
        self.entity_counters.total_entities = total;
        self.entity_counters.active_entities = active;
        self.entity_counters.culled_entities = culled;
    }
    
    /// Update cache stats
    pub fn update_cache_stats(&mut self, hits: usize, misses: usize, cache_type: &str) {
        match cache_type {
            "distance" => {
                self.cache_stats.distance_cache_hits = hits;
                self.cache_stats.distance_cache_misses = misses;
            }
            "asset" => {
                self.cache_stats.asset_cache_hits = hits;
                self.cache_stats.asset_cache_misses = misses;
            }
            "lod" => {
                self.cache_stats.lod_cache_hits = hits;
                self.cache_stats.lod_cache_misses = misses;
            }
            _ => {}
        }
    }
    
    /// Update memory usage
    pub fn update_memory_usage(&mut self, entity_memory: usize, system_memory: usize, total: usize) {
        self.memory_tracker.entity_memory = entity_memory;
        self.memory_tracker.system_memory = system_memory;
        self.memory_tracker.total_allocated = total;
        
        if total > self.memory_tracker.peak_memory {
            self.memory_tracker.peak_memory = total;
        }
        
        // Calculate memory pressure (simplified)
        self.memory_tracker.memory_pressure = total as f32 / (2.0 * 1024.0 * 1024.0 * 1024.0); // Assume 2GB limit
    }
    
    /// Get cache hit ratio
    pub fn get_cache_hit_ratio(&self, cache_type: &str) -> f32 {
        let (hits, misses) = match cache_type {
            "distance" => (self.cache_stats.distance_cache_hits, self.cache_stats.distance_cache_misses),
            "asset" => (self.cache_stats.asset_cache_hits, self.cache_stats.asset_cache_misses),
            "lod" => (self.cache_stats.lod_cache_hits, self.cache_stats.lod_cache_misses),
            _ => return 0.0,
        };
        
        let total = hits + misses;
        if total > 0 {
            hits as f32 / total as f32
        } else {
            0.0
        }
    }
    
    /// Add alert
    pub fn add_alert(&mut self, alert: PerformanceAlert) {
        self.alerts.push(alert);
        
        // Keep only recent alerts
        self.alerts.retain(|a| a.timestamp.elapsed() < Duration::from_secs(60));
    }
    
    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let avg_fps = if !self.frame_analyzer.fps_history.is_empty() {
            self.frame_analyzer.fps_history.iter().sum::<f32>() / self.frame_analyzer.fps_history.len() as f32
        } else {
            0.0
        };
        
        PerformanceSummary {
            avg_fps,
            avg_frame_time: self.frame_analyzer.avg_frame_time,
            consistency_score: self.frame_analyzer.consistency_score,
            total_entities: self.entity_counters.total_entities,
            active_entities: self.entity_counters.active_entities,
            culled_entities: self.entity_counters.culled_entities,
            memory_usage_gb: self.memory_tracker.total_allocated as f32 / (1024.0 * 1024.0 * 1024.0),
            cache_hit_ratio: self.get_cache_hit_ratio("distance"),
        }
    }
}

/// Main performance monitoring system
pub fn unified_performance_monitoring_system(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    diagnostics: Res<DiagnosticsStore>,
    _time: Res<Time>,
) {
    if !tracker.enabled {
        return;
    }
    
    // Update frame timing from Bevy diagnostics
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(frame_time) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
            if let (Some(fps_val), Some(frame_time_val)) = (fps.smoothed(), frame_time.smoothed()) {
                tracker.update_frame_timing(frame_time_val as f32, fps_val as f32);
            }
        }
    }
    
    // Generate periodic reports
    if tracker.last_report.elapsed() > tracker.report_interval {
        let summary = tracker.get_performance_summary();
        
        println!("🚀 Performance Report:");
        println!("   FPS: {:.1} | Frame: {:.2}ms | Consistency: {:.1}%",
                 summary.avg_fps, summary.avg_frame_time, summary.consistency_score * 100.0);
        println!("   Entities: {} (Active: {}, Culled: {})",
                 summary.total_entities, summary.active_entities, summary.culled_entities);
        
        tracker.last_report = Instant::now();
    }
}

/// Debug input system for performance monitoring
pub fn performance_debug_input_system(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F3) {
        tracker.enabled = !tracker.enabled;
        println!("Performance monitoring: {}", if tracker.enabled { "ON" } else { "OFF" });
    }
}

/// Performance overlay component
#[derive(Component)]
pub struct PerformanceOverlay;

/// Setup performance overlay
pub fn setup_performance_overlay(mut commands: Commands) {
    commands.spawn((
        Text::new("Performance Overlay"),
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
    mut overlay_query: Query<&mut Text, With<PerformanceOverlay>>,
    tracker: Res<UnifiedPerformanceTracker>,
) {
    if let Ok(mut text) = overlay_query.single_mut() {
        let summary = tracker.get_performance_summary();
        text.0 = format!(
            "FPS: {:.1} | Frame: {:.2}ms | Consistency: {:.1}%\nEntities: {} (Active: {}, Culled: {})\nCache Hit Rates: D:{:.1}% A:{:.1}% L:{:.1}%",
            summary.avg_fps,
            summary.avg_frame_time,
            summary.consistency_score * 100.0,
            summary.total_entities,
            summary.active_entities,
            summary.culled_entities,
            tracker.get_cache_hit_ratio("distance") * 100.0,
            tracker.get_cache_hit_ratio("asset") * 100.0,
            tracker.get_cache_hit_ratio("lod") * 100.0,
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
            ))
            .add_systems(Startup, setup_performance_overlay);
    }
}
