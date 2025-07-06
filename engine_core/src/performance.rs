use std::collections::HashMap;

/// Core performance tracking without Bevy dependencies
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    pub categories: HashMap<PerformanceCategory, CategoryMetrics>,
    pub system_timings: HashMap<String, SystemTiming>,
    pub cache_stats: CacheStats,
    pub entity_counters: EntityCounters,
    pub alerts: Vec<PerformanceAlert>,
    pub current_time: f32,
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

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Default, Clone)]
pub struct SystemTiming {
    pub last_execution_time: f32,
    pub average_time: f32,
    pub peak_time: f32,
    pub execution_count: u64,
    pub total_time: f64,
}

#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total_entries: usize,
    pub memory_usage: usize,
}

#[derive(Debug, Default, Clone)]
pub struct EntityCounters {
    pub total_entities: usize,
    pub active_entities: usize,
    pub culled_entities: usize,
    pub spawned_this_frame: usize,
    pub despawned_this_frame: usize,
}

#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub category: PerformanceCategory,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self {
            categories: HashMap::new(),
            system_timings: HashMap::new(),
            cache_stats: CacheStats::default(),
            entity_counters: EntityCounters::default(),
            alerts: Vec::new(),
            current_time: 0.0,
            enabled: true,
        }
    }
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update_time(&mut self, current_time: f32) {
        self.current_time = current_time;
    }
    
    pub fn record_category_time(&mut self, category: PerformanceCategory, time_ms: f32) {
        if !self.enabled {
            return;
        }
        
        let metrics = self.categories.entry(category).or_default();
        metrics.execution_time_ms = time_ms;
        metrics.frame_count += 1;
        metrics.total_execution_time += time_ms as f64;
        
        if time_ms > metrics.peak_execution_time {
            metrics.peak_execution_time = time_ms;
        }
        
        // Calculate rolling average
        metrics.avg_execution_time = (metrics.total_execution_time / metrics.frame_count as f64) as f32;
    }
    
    pub fn record_system_time(&mut self, system_name: &str, time_ms: f32) {
        if !self.enabled {
            return;
        }
        
        let timing = self.system_timings.entry(system_name.to_string()).or_default();
        timing.last_execution_time = time_ms;
        timing.execution_count += 1;
        timing.total_time += time_ms as f64;
        
        if time_ms > timing.peak_time {
            timing.peak_time = time_ms;
        }
        
        timing.average_time = (timing.total_time / timing.execution_count as f64) as f32;
    }
    
    pub fn update_entity_counts(&mut self, total: usize, active: usize, culled: usize) {
        self.entity_counters.total_entities = total;
        self.entity_counters.active_entities = active;
        self.entity_counters.culled_entities = culled;
    }
    
    pub fn record_cache_hit(&mut self) {
        self.cache_stats.hits += 1;
    }
    
    pub fn record_cache_miss(&mut self) {
        self.cache_stats.misses += 1;
    }
    
    pub fn update_cache_stats(&mut self, entries: usize, memory: usize) {
        self.cache_stats.total_entries = entries;
        self.cache_stats.memory_usage = memory;
    }
    
    pub fn add_alert(&mut self, category: PerformanceCategory, message: String, severity: AlertSeverity) {
        let alert = PerformanceAlert {
            category,
            message,
            severity,
            timestamp: self.current_time,
        };
        
        self.alerts.push(alert);
        
        // Keep only recent alerts
        self.alerts.retain(|alert| self.current_time - alert.timestamp < 60.0);
    }
    
    pub fn get_category_metrics(&self, category: PerformanceCategory) -> Option<&CategoryMetrics> {
        self.categories.get(&category)
    }
    
    pub fn get_system_timing(&self, system_name: &str) -> Option<&SystemTiming> {
        self.system_timings.get(system_name)
    }
    
    pub fn get_cache_hit_rate(&self) -> f32 {
        let total = self.cache_stats.hits + self.cache_stats.misses;
        if total == 0 {
            0.0
        } else {
            self.cache_stats.hits as f32 / total as f32
        }
    }
    
    pub fn clear_frame_stats(&mut self) {
        self.entity_counters.spawned_this_frame = 0;
        self.entity_counters.despawned_this_frame = 0;
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}
