use bevy::prelude::*;
use crate::components::*;
use crate::systems::*;
use crate::config::GameConfig;

/// Plugin that adds optimized batching systems with dirty flags
pub struct BatchingPlugin;

impl Plugin for BatchingPlugin {
    fn build(&self, app: &mut App) {
        // Add resources
        app.insert_resource(FrameCounter::default())
            .insert_resource(DirtyFlagsMetrics::default())
            .insert_resource(BatchState::default());

        // Initialize batch configuration from game config
        if let Some(config) = app.world().get_resource::<GameConfig>() {
            app.insert_resource(BatchConfig {
                transform_batch_size: config.batching.transform_batch_size,
                visibility_batch_size: config.batching.visibility_batch_size,
                physics_batch_size: config.batching.physics_batch_size,
                lod_batch_size: config.batching.lod_batch_size,
                max_processing_time_ms: config.batching.max_processing_time_ms,
                priority_boost_frames: config.batching.priority_boost_frames,
            });
        } else {
            app.insert_resource(BatchConfig::default());
        }

        // Core batching systems - run in PreUpdate to mark dirty flags
        app.add_systems(PreUpdate, frame_counter_system)
            .add_systems(PreUpdate, mark_transform_dirty_system)
            .add_systems(PreUpdate, mark_visibility_dirty_system)
            .add_systems(PreUpdate, mark_physics_dirty_system);
            // .add_systems(PreUpdate, movement_based_lod_marking_system); // Removed - functionality moved to unified systems

        // Batch processing systems - run in Update
        app.add_systems(Update, (
            batch_transform_processing_system,
            batch_physics_processing_system,
            batch_lod_processing_system,
            batch_culling_system,
        ).chain());

        // Optimized LOD systems - run in Update after batch processing
        app.add_systems(Update, master_unified_lod_system)
            .add_systems(Update, new_unified_distance_culling_system);

        // Cleanup and monitoring systems - run in PostUpdate
        app.add_systems(PostUpdate, (
            dirty_flag_cleanup_system,
            dirty_flags_metrics_system,
            // optimized_lod_performance_monitor, // Removed - functionality moved to unified_lod.rs
            // periodic_lod_marking_system, // Removed - functionality moved to unified systems
        ));

        // Add test systems for development/debugging
        #[cfg(debug_assertions)]
        app.add_systems(Update, (
            crate::systems::batching_test_system,
            crate::systems::batching_stress_test_system,
            crate::systems::batching_performance_comparison_system,
            crate::systems::cleanup_test_entities_system,
        ));
    }
}

/// Helper system set for batching-related systems
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BatchingSystemSet {
    /// Systems that mark entities as dirty
    MarkDirty,
    /// Systems that process dirty entities in batches
    ProcessBatches,
    /// Systems that clean up after batch processing
    Cleanup,
    /// Systems that monitor performance
    Monitor,
}

/// Enhanced batching plugin with more configuration options
pub struct EnhancedBatchingPlugin {
    pub enable_transform_batching: bool,
    pub enable_visibility_batching: bool,
    pub enable_physics_batching: bool,
    pub enable_lod_batching: bool,
    pub enable_performance_monitoring: bool,
}

impl Default for EnhancedBatchingPlugin {
    fn default() -> Self {
        Self {
            enable_transform_batching: true,
            enable_visibility_batching: true,
            enable_physics_batching: true,
            enable_lod_batching: true,
            enable_performance_monitoring: true,
        }
    }
}

impl Plugin for EnhancedBatchingPlugin {
    fn build(&self, app: &mut App) {
        // Add resources
        app.insert_resource(FrameCounter::default())
            .insert_resource(DirtyFlagsMetrics::default())
            .insert_resource(BatchState::default());

        // Initialize batch configuration from game config
        if let Some(config) = app.world().get_resource::<GameConfig>() {
            app.insert_resource(BatchConfig {
                transform_batch_size: config.batching.transform_batch_size,
                visibility_batch_size: config.batching.visibility_batch_size,
                physics_batch_size: config.batching.physics_batch_size,
                lod_batch_size: config.batching.lod_batch_size,
                max_processing_time_ms: config.batching.max_processing_time_ms,
                priority_boost_frames: config.batching.priority_boost_frames,
            });
        } else {
            app.insert_resource(BatchConfig::default());
        }

        // Configure system sets
        app.configure_sets(PreUpdate, BatchingSystemSet::MarkDirty)
            .configure_sets(Update, BatchingSystemSet::ProcessBatches)
            .configure_sets(PostUpdate, (
                BatchingSystemSet::Cleanup,
                BatchingSystemSet::Monitor,
            ).chain());

        // Core systems that always run
        app.add_systems(PreUpdate, frame_counter_system);

        // Conditional systems based on configuration
        if self.enable_transform_batching {
            app.add_systems(PreUpdate, mark_transform_dirty_system);
            
            app.add_systems(Update, batch_transform_processing_system);
        }

        if self.enable_visibility_batching {
            app.add_systems(PreUpdate, mark_visibility_dirty_system);
            
            app.add_systems(Update, batch_culling_system)
                .add_systems(Update, new_unified_distance_culling_system);
        }

        if self.enable_physics_batching {
            app.add_systems(PreUpdate, mark_physics_dirty_system);
            
            app.add_systems(Update, batch_physics_processing_system);
        }

        if self.enable_lod_batching {
            // app.add_systems(PreUpdate, movement_based_lod_marking_system); // Removed - functionality moved to unified systems
            
            app.add_systems(Update, batch_lod_processing_system)
                .add_systems(Update, master_unified_lod_system);
                // .add_systems(Update, periodic_lod_marking_system); // Removed - functionality moved to unified systems
        }

        // Cleanup systems
        app.add_systems(PostUpdate, dirty_flag_cleanup_system);

        // Performance monitoring systems
        if self.enable_performance_monitoring {
            app.add_systems(PostUpdate, dirty_flags_metrics_system);
                // .add_systems(PostUpdate, optimized_lod_performance_monitor); // Removed - functionality moved to unified_lod.rs
        }
    }
}

/// Utility functions for working with dirty flags
pub mod dirty_flag_utils {
    use super::*;

    /// Mark an entity as needing transform updates
    pub fn mark_entity_transform_dirty(
        commands: &mut Commands,
        entity: Entity,
        priority: DirtyPriority,
        frame: u64,
    ) {
        commands.entity(entity).insert(DirtyTransform::new(priority, frame));
    }

    /// Mark an entity as needing visibility updates
    pub fn mark_entity_visibility_dirty(
        commands: &mut Commands,
        entity: Entity,
        priority: DirtyPriority,
        frame: u64,
    ) {
        commands.entity(entity).insert(DirtyVisibility::new(priority, frame));
    }

    /// Mark an entity as needing physics updates
    pub fn mark_entity_physics_dirty(
        commands: &mut Commands,
        entity: Entity,
        priority: DirtyPriority,
        frame: u64,
    ) {
        commands.entity(entity).insert(DirtyPhysics::new(priority, frame));
    }

    /// Mark an entity as needing LOD updates
    pub fn mark_entity_lod_dirty(
        commands: &mut Commands,
        entity: Entity,
        priority: DirtyPriority,
        frame: u64,
        distance: f32,
    ) {
        commands.entity(entity).insert(DirtyLOD::new(priority, frame, distance));
    }

    /// Mark multiple entities as needing transform updates
    pub fn mark_entities_transform_dirty(
        commands: &mut Commands,
        entities: &[Entity],
        priority: DirtyPriority,
        frame: u64,
    ) {
        for &entity in entities {
            mark_entity_transform_dirty(commands, entity, priority, frame);
        }
    }

    /// Get processing statistics for dirty flag systems
    pub fn get_dirty_flag_stats(metrics: &DirtyFlagsMetrics) -> String {
        format!(
            "Dirty Flags Stats - Marked: T:{} V:{} P:{} L:{} | Processed: T:{} V:{} P:{} L:{} | Avg Time: T:{:.1}ms V:{:.1}ms P:{:.1}ms L:{:.1}ms",
            metrics.entities_marked_transform,
            metrics.entities_marked_visibility,
            metrics.entities_marked_physics,
            metrics.entities_marked_lod,
            metrics.entities_processed_transform,
            metrics.entities_processed_visibility,
            metrics.entities_processed_physics,
            metrics.entities_processed_lod,
            metrics.processing_time_transform,
            metrics.processing_time_visibility,
            metrics.processing_time_physics,
            metrics.processing_time_lod,
        )
    }

    /// Calculate recommended batch sizes based on entity count
    pub fn calculate_optimal_batch_sizes(
        total_entities: usize,
        target_frame_time_ms: f32,
    ) -> BatchConfig {
        let base_batch_size = (total_entities / 10).max(20).min(200);
        
        BatchConfig {
            transform_batch_size: base_batch_size,
            visibility_batch_size: (base_batch_size * 12 / 10),
            physics_batch_size: (base_batch_size * 8 / 10),
            lod_batch_size: base_batch_size,
            max_processing_time_ms: target_frame_time_ms * 0.5, // Use half the target frame time
            priority_boost_frames: 10,
        }
    }
}
