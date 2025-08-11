use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Resource, Default)]
pub struct SystemMetrics {
    pub timings: HashMap<&'static str, SystemTiming>,
}

#[derive(Debug, Clone)]
pub struct SystemTiming {
    pub last_duration: Duration,
    pub average_duration: Duration,
    pub max_duration: Duration,
    pub min_duration: Duration,
    pub call_count: u64,
    pub samples: Vec<Duration>,
}

impl Default for SystemTiming {
    fn default() -> Self {
        Self {
            last_duration: Duration::ZERO,
            average_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            call_count: 0,
            samples: Vec::with_capacity(100),
        }
    }
}

impl SystemMetrics {
    pub fn record(&mut self, system_name: &'static str, duration: Duration) {
        let timing = self.timings.entry(system_name).or_default();
        
        timing.last_duration = duration;
        timing.call_count += 1;
        timing.max_duration = timing.max_duration.max(duration);
        timing.min_duration = timing.min_duration.min(duration);
        
        // Keep sliding window of samples
        timing.samples.push(duration);
        if timing.samples.len() > 100 {
            timing.samples.remove(0);
        }
        
        // Calculate average
        if !timing.samples.is_empty() {
            let sum: Duration = timing.samples.iter().sum();
            timing.average_duration = sum / timing.samples.len() as u32;
        }
    }
    
    pub fn get_slow_systems(&self, threshold: Duration) -> Vec<(&'static str, Duration)> {
        self.timings
            .iter()
            .filter(|(_, timing)| timing.average_duration > threshold)
            .map(|(name, timing)| (*name, timing.average_duration))
            .collect()
    }
}

// System profiler resource for collecting metrics
#[derive(Resource, Default)]
pub struct SystemProfiler {
    pending_metrics: Vec<(&'static str, Duration)>,
}

impl SystemProfiler {
    pub fn record(&mut self, name: &'static str, duration: Duration) {
        self.pending_metrics.push((name, duration));
    }
    
    pub fn flush(&mut self, metrics: &mut SystemMetrics) {
        for (name, duration) in self.pending_metrics.drain(..) {
            metrics.record(name, duration);
        }
    }
}

// Macro for profiling systems - requires SystemProfiler resource
#[macro_export]
macro_rules! profiled_system {
    ($name:literal, $profiler:expr, $body:expr) => {
        {
            #[cfg(feature = "debug-events")]
            let _start = std::time::Instant::now();
            
            let result = $body;
            
            #[cfg(feature = "debug-events")]
            {
                let duration = _start.elapsed();
                $profiler.record($name, duration);
            }
            
            result
        }
    };
}



// Flush profiler metrics to resource
pub fn flush_system_metrics(
    mut profiler: ResMut<SystemProfiler>,
    mut metrics: ResMut<SystemMetrics>,
) {
    profiler.flush(&mut metrics);
}

// Helper for identifying hot paths
pub fn analyze_hot_paths(metrics: Res<SystemMetrics>) {
    let threshold = Duration::from_millis(1);
    let slow_systems = metrics.get_slow_systems(threshold);
    
    if !slow_systems.is_empty() {
        debug!("Slow systems (>1ms average):");
        for (name, duration) in slow_systems {
            debug!("  {}: {:?}", name, duration);
        }
    }
}

// Performance budget tracking
#[derive(Resource)]
pub struct PerformanceBudget {
    pub frame_budget: Duration,
    pub system_budgets: HashMap<&'static str, Duration>,
}

impl Default for PerformanceBudget {
    fn default() -> Self {
        Self {
            frame_budget: Duration::from_millis(16), // 60 FPS target
            system_budgets: HashMap::new(),
        }
    }
}

pub fn check_performance_budget(
    metrics: Res<SystemMetrics>,
    budget: Res<PerformanceBudget>,
) {
    for (name, timing) in &metrics.timings {
        if let Some(budget_duration) = budget.system_budgets.get(name) {
            if timing.average_duration > *budget_duration {
                warn!(
                    "System '{}' exceeds budget: {:?} > {:?}",
                    name, timing.average_duration, budget_duration
                );
            }
        }
    }
}
