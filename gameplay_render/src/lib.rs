//! Gameplay rendering - LOD, culling, effects
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;
pub use game_core;

pub mod prelude;
pub mod systems;

pub use prelude::*;

/// Main plugin for rendering systems
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;
        
        // Initialize resources
        app.init_resource::<game_core::config::performance_config::PerformanceCounters>()
            .init_resource::<game_core::components::dirty_flags::FrameCounter>()
            .init_resource::<game_core::components::instanced_vegetation::VegetationInstancingConfig>();
        
        // Add core rendering systems
        app.add_systems(
            Update,
            (
                // Camera systems
                camera::camera_follow_system,
                
                // LOD systems
                lod::modern_lod_system,
                lod::lod_performance_monitoring_system,
                
                // Audio systems
                audio::realistic_vehicle_audio_system,
                audio::vehicle_audio_culling_system,
                audio::vehicle_audio_performance_system,
                
                // Visual effects
                effects::update_jet_flames,
                effects::update_flame_colors,
                effects::exhaust_effects_system,
                effects::update_waypoint_system,
                effects::update_beacon_visibility,
                
                // Rendering optimization
                rendering::render_optimization_system,
                rendering::collect_vegetation_instances_system,
                rendering::update_vegetation_instancing_system,
                rendering::mark_vegetation_instancing_dirty_system,
                rendering::animate_vegetation_instances_system,
                rendering::vegetation_instancing_metrics_system,
                
                // Vegetation integration
                vegetation_instancing_integration::integrate_vegetation_with_instancing_system,
                vegetation_instancing_integration::spawn_test_vegetation_system,
                
                // Debug systems
                distance_cache_debug::distance_cache_debug_system,
            )
        );
        
        // Add post-update systems
        app.add_systems(
            PostUpdate,
            (
                transform_sync::sync_transforms_system,
                // visibility_fix::fix_missing_inherited_visibility,
                // visibility_fix::fix_parent_visibility,
            )
        );
    }
}
