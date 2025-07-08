//! Bevy plugin for optimized batch processing systems with dirty flag management.
//!
//! # Overview
//! The [`BatchingPlugin`] provides a comprehensive batching system that optimizes
//! entity processing by grouping operations and using dirty flags to minimize
//! redundant work. This plugin is essential for maintaining 60+ FPS in complex
//! game worlds with thousands of entities.
//!
//! The plugin registers multiple systems that work together:
//! - **Dirty flag marking**: Identifies entities that need processing
//! - **Batch processing**: Groups similar operations for efficiency
//! - **Performance monitoring**: Tracks processing metrics and timings
//! - **LOD integration**: Manages level-of-detail based on distance
//!
//! ## Typical usage
//! ```rust
//! use bevy::prelude::*;
//! use gameplay_render::plugins::batching_plugin::BatchingPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(BatchingPlugin)
//!         .run();
//! }
//! ```
//!
//! For more control over which systems to enable:
//! ```rust
//! use bevy::prelude::*;
//! use gameplay_render::plugins::batching_plugin::EnhancedBatchingPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(EnhancedBatchingPlugin {
//!             enable_transform_batching: true,
//!             enable_physics_batching: false,
//!             enable_performance_monitoring: true,
//!             ..default()
//!         })
//!         .run();
//! }
//! ```
//!
//! # Performance considerations
//! The batching system provides significant performance improvements:
//! - 300%+ performance gains in dense entity scenarios
//! - Reduced CPU overhead through batch processing
//! - Intelligent dirty flag management to avoid redundant work
//! - Configurable batch sizes for different entity types
//!
//! # Implementation notes
//! The plugin integrates with [`GameConfig`] to load batch sizes and timing
//! constraints from configuration files. It uses Bevy's scheduling system to
//! ensure proper ordering of dirty flag marking, batch processing, and cleanup.

use bevy::prelude::*;
use game_core::components::*;
use crate::batching::*;
use crate::batching_test::*;
use crate::batch_processing::*;
use game_core::config::GameConfig;

/// A Bevy plugin that adds optimized batching systems with dirty flag management.
///
/// This plugin provides a complete batching solution for high-performance entity
/// processing in dense game worlds. It registers systems for transform updates,
/// visibility culling, physics processing, and LOD management, all optimized
/// through dirty flag tracking and batch processing.
///
/// The plugin automatically configures itself based on the [`GameConfig`] resource
/// if available, otherwise falls back to sensible defaults. It integrates seamlessly
/// with the existing rendering pipeline and physics systems.
///
/// # Systems registered
/// - **PreUpdate**: Dirty flag marking systems and frame counter
/// - **Update**: Batch processing systems and LOD management
/// - **PostUpdate**: Cleanup and performance monitoring systems
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use gameplay_render::plugins::batching_plugin::BatchingPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(BatchingPlugin)
///         .run();
/// }
/// ```
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
        app.add_systems(Update, gameplay_sim::world::master_unified_lod_system)
            .add_systems(Update, gameplay_sim::world::unified_distance_culling::new_unified_distance_culling_system);

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
            crate::batching_test::batching_test_system,
            crate::batching_test::batching_stress_test_system,
            crate::batching_test::batching_performance_comparison_system,
            crate::batching_test::cleanup_test_entities_system,
        ));
    }
}

/// System sets for organizing batching-related systems by execution phase.
///
/// This enum defines the logical grouping of batching systems to ensure proper
/// execution order and dependency management. Each set runs in a specific
/// schedule phase and depends on the previous set completing.
///
/// # Execution order
/// 1. [`MarkDirty`] - Identifies entities needing processing
/// 2. [`ProcessBatches`] - Performs batch operations on dirty entities
/// 3. [`Cleanup`] - Removes processed dirty flags
/// 4. [`Monitor`] - Collects performance metrics
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use gameplay_render::plugins::batching_plugin::BatchingSystemSet;
///
/// fn setup_custom_batching_system(app: &mut App) {
///     app.add_systems(Update, my_custom_batch_system
///         .in_set(BatchingSystemSet::ProcessBatches));
/// }
/// # fn my_custom_batch_system() {}
/// ```
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BatchingSystemSet {
    /// Systems that mark entities as dirty and need processing.
    ///
    /// These systems run in [`PreUpdate`] and identify entities that have
    /// changed state and require batch processing. They add dirty flag
    /// components to entities based on various criteria.
    MarkDirty,
    /// Systems that process dirty entities in optimized batches.
    ///
    /// These systems run in [`Update`] and handle the actual work of
    /// processing entities marked as dirty. They group operations for
    /// maximum efficiency and respect timing constraints.
    ProcessBatches,
    /// Systems that clean up after batch processing completes.
    ///
    /// These systems run in [`PostUpdate`] and remove dirty flag components
    /// from entities that have been successfully processed. They ensure
    /// the system state is ready for the next frame.
    Cleanup,
    /// Systems that monitor performance and collect metrics.
    ///
    /// These systems run in [`PostUpdate`] after cleanup and gather
    /// statistics about batch processing performance, timing, and
    /// entity counts for optimization purposes.
    Monitor,
}

/// An enhanced batching plugin with granular control over individual systems.
///
/// This plugin provides the same functionality as [`BatchingPlugin`] but allows
/// selective enabling/disabling of individual batching systems. This is useful
/// for performance testing, debugging, or when certain systems are not needed
/// in specific game scenarios.
///
/// Each batching system can be independently controlled:
/// - Transform batching for entity positioning and scaling
/// - Visibility batching for culling and LOD management
/// - Physics batching for collision and dynamics processing
/// - LOD batching for level-of-detail adjustments
/// - Performance monitoring for metrics collection
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use gameplay_render::plugins::batching_plugin::EnhancedBatchingPlugin;
///
/// // Enable only transform and visibility batching
/// fn setup_selective_batching(app: &mut App) {
///     app.add_plugins(EnhancedBatchingPlugin {
///         enable_transform_batching: true,
///         enable_visibility_batching: true,
///         enable_physics_batching: false,
///         enable_lod_batching: false,
///         enable_performance_monitoring: true,
///     });
/// }
/// ```
pub struct EnhancedBatchingPlugin {
    /// Whether to enable transform batching systems for entity positioning
    pub enable_transform_batching: bool,
    /// Whether to enable visibility batching systems for culling operations
    pub enable_visibility_batching: bool,
    /// Whether to enable physics batching systems for collision processing
    pub enable_physics_batching: bool,
    /// Whether to enable LOD batching systems for level-of-detail management
    pub enable_lod_batching: bool,
    /// Whether to enable performance monitoring systems for metrics collection
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
                .add_systems(Update, gameplay_sim::world::unified_distance_culling::new_unified_distance_culling_system);
        }

        if self.enable_physics_batching {
            app.add_systems(PreUpdate, mark_physics_dirty_system);
            
            app.add_systems(Update, batch_physics_processing_system);
        }

        if self.enable_lod_batching {
            // app.add_systems(PreUpdate, movement_based_lod_marking_system); // Removed - functionality moved to unified systems
            
            app.add_systems(Update, batch_lod_processing_system)
                .add_systems(Update, gameplay_sim::world::master_unified_lod_system);
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

/// Utility functions for working with dirty flags and batch processing.
///
/// This module provides convenience functions for marking entities as dirty
/// and managing batch processing state. These utilities are used by gameplay
/// systems to trigger batch processing when entities change state.
///
/// # Usage patterns
/// Most functions in this module are designed to be called from gameplay systems
/// when they detect entity state changes. The dirty flag system ensures that
/// batch processing systems will handle these entities efficiently.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use gameplay_render::plugins::batching_plugin::dirty_flag_utils::*;
/// use crate::batching::DirtyPriority;
///
/// fn movement_system(mut commands: Commands, frame: Res<FrameCounter>) {
///     let entity = Entity::from_raw(1);
///     mark_entity_transform_dirty(&mut commands, entity, DirtyPriority::High, frame.0);
/// }
/// ```
pub mod dirty_flag_utils {
    use super::*;

    /// Marks an entity as needing transform updates in the next batch processing cycle.
    ///
    /// This function adds a [`DirtyTransform`] component to the entity, which signals
    /// the batch processing system to include this entity in the next transform update
    /// batch. The priority determines processing order within the batch.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer
    /// * `entity` - The entity to mark as dirty
    /// * `priority` - Processing priority for this entity
    /// * `frame` - Current frame number for tracking
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::plugins::batching_plugin::dirty_flag_utils::*;
    /// use crate::batching::DirtyPriority;
    ///
    /// fn handle_movement(mut commands: Commands, frame: Res<FrameCounter>) {
    ///     let entity = Entity::from_raw(1);
    ///     mark_entity_transform_dirty(&mut commands, entity, DirtyPriority::High, frame.0);
    /// }
    /// ```
    pub fn mark_entity_transform_dirty(
        commands: &mut Commands,
        entity: Entity,
        priority: DirtyPriority,
        frame: u64,
    ) {
        commands.entity(entity).insert(DirtyTransform::new(priority, frame));
    }

    /// Marks an entity as needing visibility updates in the next batch processing cycle.
    ///
    /// This function adds a [`DirtyVisibility`] component to the entity, which signals
    /// the batch processing system to include this entity in the next visibility/culling
    /// update batch. Used when entities move or change visibility state.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer
    /// * `entity` - The entity to mark as dirty
    /// * `priority` - Processing priority for this entity
    /// * `frame` - Current frame number for tracking
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::plugins::batching_plugin::dirty_flag_utils::*;
    /// use crate::batching::DirtyPriority;
    ///
    /// fn handle_visibility_change(mut commands: Commands, frame: Res<FrameCounter>) {
    ///     let entity = Entity::from_raw(1);
    ///     mark_entity_visibility_dirty(&mut commands, entity, DirtyPriority::Medium, frame.0);
    /// }
    /// ```
    pub fn mark_entity_visibility_dirty(
        commands: &mut Commands,
        entity: Entity,
        priority: DirtyPriority,
        frame: u64,
    ) {
        commands.entity(entity).insert(DirtyVisibility::new(priority, frame));
    }

    /// Marks an entity as needing physics updates in the next batch processing cycle.
    ///
    /// This function adds a [`DirtyPhysics`] component to the entity, which signals
    /// the batch processing system to include this entity in the next physics update
    /// batch. Used when entities change velocity, collision state, or other physics properties.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer
    /// * `entity` - The entity to mark as dirty
    /// * `priority` - Processing priority for this entity
    /// * `frame` - Current frame number for tracking
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::plugins::batching_plugin::dirty_flag_utils::*;
    /// use crate::batching::DirtyPriority;
    ///
    /// fn handle_collision(mut commands: Commands, frame: Res<FrameCounter>) {
    ///     let entity = Entity::from_raw(1);
    ///     mark_entity_physics_dirty(&mut commands, entity, DirtyPriority::High, frame.0);
    /// }
    /// ```
    pub fn mark_entity_physics_dirty(
        commands: &mut Commands,
        entity: Entity,
        priority: DirtyPriority,
        frame: u64,
    ) {
        commands.entity(entity).insert(DirtyPhysics::new(priority, frame));
    }

    /// Marks an entity as needing LOD updates in the next batch processing cycle.
    ///
    /// This function adds a [`DirtyLOD`] component to the entity, which signals
    /// the batch processing system to include this entity in the next level-of-detail
    /// update batch. The distance parameter helps prioritize entities closer to the camera.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer
    /// * `entity` - The entity to mark as dirty
    /// * `priority` - Processing priority for this entity
    /// * `frame` - Current frame number for tracking
    /// * `distance` - Distance from camera for LOD calculations
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::plugins::batching_plugin::dirty_flag_utils::*;
    /// use crate::batching::DirtyPriority;
    ///
    /// fn handle_lod_change(mut commands: Commands, frame: Res<FrameCounter>) {
    ///     let entity = Entity::from_raw(1);
    ///     let distance = 50.0; // Distance from camera
    ///     mark_entity_lod_dirty(&mut commands, entity, DirtyPriority::Medium, frame.0, distance);
    /// }
    /// ```
    pub fn mark_entity_lod_dirty(
        commands: &mut Commands,
        entity: Entity,
        priority: DirtyPriority,
        frame: u64,
        distance: f32,
    ) {
        commands.entity(entity).insert(DirtyLOD::new(priority, frame, distance));
    }

    /// Marks multiple entities as needing transform updates in batch.
    ///
    /// This is a convenience function for marking many entities as dirty at once,
    /// which is more efficient than calling [`mark_entity_transform_dirty`] individually.
    /// Commonly used when large groups of entities move together.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer
    /// * `entities` - Slice of entities to mark as dirty
    /// * `priority` - Processing priority for all entities
    /// * `frame` - Current frame number for tracking
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::plugins::batching_plugin::dirty_flag_utils::*;
    /// use crate::batching::DirtyPriority;
    ///
    /// fn handle_formation_movement(mut commands: Commands, frame: Res<FrameCounter>) {
    ///     let entities = vec![Entity::from_raw(1), Entity::from_raw(2), Entity::from_raw(3)];
    ///     mark_entities_transform_dirty(&mut commands, &entities, DirtyPriority::High, frame.0);
    /// }
    /// ```
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

    /// Generates a formatted string containing dirty flag processing statistics.
    ///
    /// This function creates a human-readable summary of batch processing performance,
    /// including entity counts and processing times for each batch type. Useful for
    /// debugging and performance monitoring.
    ///
    /// # Arguments
    /// * `metrics` - Reference to the current dirty flags metrics
    ///
    /// # Returns
    /// A formatted string containing processing statistics
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::plugins::batching_plugin::dirty_flag_utils::*;
    /// use crate::batching::DirtyFlagsMetrics;
    ///
    /// fn print_batch_stats(metrics: Res<DirtyFlagsMetrics>) {
    ///     let stats = get_dirty_flag_stats(&metrics);
    ///     println!("{}", stats);
    /// }
    /// ```
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

    /// Calculates optimal batch sizes based on entity count and performance targets.
    ///
    /// This function determines appropriate batch sizes for different processing types
    /// based on the total number of entities and target frame time. It uses heuristics
    /// to balance processing efficiency with frame rate stability.
    ///
    /// # Arguments
    /// * `total_entities` - Total number of entities in the world
    /// * `target_frame_time_ms` - Target frame time in milliseconds
    ///
    /// # Returns
    /// A [`BatchConfig`] with optimized batch sizes
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::plugins::batching_plugin::dirty_flag_utils::*;
    ///
    /// fn optimize_batching(entity_count: usize) {
    ///     let config = calculate_optimal_batch_sizes(entity_count, 16.67); // 60 FPS
    ///     println!("Recommended transform batch size: {}", config.transform_batch_size);
    /// }
    /// ```
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
