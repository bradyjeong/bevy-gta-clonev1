//! Rendering plugins that organize and execute rendering systems in optimal order.
//!
//! This module provides a collection of specialized [`Plugin`] implementations
//! that manage the execution order and coordination of rendering systems. Each
//! plugin represents a logical grouping of related functionality with carefully
//! orchestrated system dependencies and execution schedules.
//!
//! # Overview
//!
//! The plugins module serves as the central coordination point for all rendering
//! operations in the game engine. It ensures systems execute in the correct order,
//! manages resource dependencies, and provides configuration options for different
//! rendering features.
//!
//! ## Plugin architecture
//!
//! Each plugin in this module follows a consistent pattern:
//!
//! - **Resource initialization**: Sets up necessary [`Resource`] types and
//!   default configurations
//! - **System registration**: Adds systems to appropriate [`Schedule`] phases
//!   ([`PreUpdate`], [`Update`], [`PostUpdate`])
//! - **Dependency management**: Ensures systems run in the correct order using
//!   system sets and chains
//! - **Configuration support**: Integrates with [`GameConfig`] for runtime
//!   parameter adjustment
//!
//! ## Execution order
//!
//! The plugins coordinate system execution across three main phases:
//!
//! 1. **PreUpdate**: Entity marking and dirty flag systems
//! 2. **Update**: Core processing, batching, and rendering systems
//! 3. **PostUpdate**: Cleanup, monitoring, and validation systems
//!
//! This ensures optimal performance and prevents race conditions between
//! interdependent systems.
//!
//! # Available plugins
//!
//! ## [`BatchingPlugin`]
//!
//! The core batching plugin that manages parallel processing of large entity
//! sets. It provides:
//!
//! - Dirty flag marking for efficient change detection
//! - Batch processing systems for transforms, visibility, physics, and LOD
//! - Performance monitoring and optimization
//! - Integration with unified LOD and culling systems
//!
//! # Usage example
//!
//! ```rust
//! use bevy::prelude::*;
//! use gameplay_render::plugins::BatchingPlugin;
//! use game_core::config::GameConfig;
//!
//! fn setup_rendering(app: &mut App) {
//!     app
//!         .add_plugins(BatchingPlugin)
//!         .init_resource::<GameConfig>();
//! }
//!
//! // The plugin automatically configures itself based on GameConfig
//! fn configure_batching(mut config: ResMut<GameConfig>) {
//!     config.batching.transform_batch_size = 128;
//!     config.batching.max_processing_time_ms = 8.0;
//! }
//! ```
//!
//! # Performance characteristics
//!
//! The plugin system is designed for optimal performance in large-scale scenes:
//!
//! - **Batch processing**: Groups operations to minimize per-entity overhead
//! - **Parallel execution**: Uses Bevy's parallel system execution where possible
//! - **Memory efficiency**: Minimizes allocations through resource reuse
//! - **Cache locality**: Processes entities in patterns that maximize CPU cache hits
//!
//! # Integration with core systems
//!
//! The plugins integrate seamlessly with other crate modules:
//!
//! - [`crate::systems`] provides the individual system implementations
//! - [`crate::batching`] supplies the batching infrastructure
//! - [`crate::world`] coordinates entity creation and management
//! - [`gameplay_sim`] provides the source data for rendering operations
//!
//! # Development and debugging
//!
//! In debug builds, additional systems are enabled for development support:
//!
//! - Performance profiling and metrics collection
//! - System execution timing and bottleneck detection
//! - Entity count monitoring and batch size optimization
//! - Visual debugging tools for LOD and culling systems

pub mod batching_plugin;

pub use batching_plugin::*;
