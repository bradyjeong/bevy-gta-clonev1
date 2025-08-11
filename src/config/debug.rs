use serde::{Deserialize, Serialize};

/// Debug configuration values
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebugConfig {
    pub overlays: OverlayConfig,
    pub logging: LoggingConfig,
    pub instrumentation: InstrumentationConfig,
    pub cheats: CheatConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub show_fps: bool,
    pub show_entity_count: bool,
    pub show_cache_stats: bool,
    pub show_physics_debug: bool,
    pub show_collision_boxes: bool,
    pub show_lod_levels: bool,
    pub show_culling_info: bool,
    pub show_event_flow: bool,
    pub overlay_opacity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String, // "error", "warn", "info", "debug", "trace"
    pub log_events: bool,
    pub log_performance: bool,
    pub log_physics: bool,
    pub log_ai: bool,
    pub log_rendering: bool,
    pub log_input: bool,
    pub performance_threshold_ms: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstrumentationConfig {
    pub enable_profiling: bool,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub sample_rate: f32,
    pub metrics_interval: f32,
    pub profile_output_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheatConfig {
    pub god_mode: bool,
    pub infinite_fuel: bool,
    pub no_clip: bool,
    pub spawn_anything: bool,
    pub time_control: bool,
    pub weather_control: bool,
    pub teleport: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            overlays: OverlayConfig {
                show_fps: false,
                show_entity_count: false,
                show_cache_stats: false,
                show_physics_debug: false,
                show_collision_boxes: false,
                show_lod_levels: false,
                show_culling_info: false,
                show_event_flow: false,
                overlay_opacity: 0.8,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                log_events: false,
                log_performance: false,
                log_physics: false,
                log_ai: false,
                log_rendering: false,
                log_input: false,
                performance_threshold_ms: 16.0, // Log if frame takes > 16ms
            },
            instrumentation: InstrumentationConfig {
                enable_profiling: false,
                enable_metrics: false,
                enable_tracing: false,
                sample_rate: 0.1,
                metrics_interval: 1.0,
                profile_output_path: "profiles/".to_string(),
            },
            cheats: CheatConfig {
                god_mode: false,
                infinite_fuel: false,
                no_clip: false,
                spawn_anything: false,
                time_control: false,
                weather_control: false,
                teleport: false,
            },
        }
    }
}
