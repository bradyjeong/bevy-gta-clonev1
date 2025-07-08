//! World rendering systems and ECS components for a render-ready game world.
//!
//! This module provides the core infrastructure for transforming gameplay simulation
//! data into render-ready ECS entities. It bridges the gap between gameplay logic
//! and visual representation, ensuring consistent entity creation and efficient
//! rendering pipelines.
//!
//! # Overview
//!
//! The world module handles the complete lifecycle of world entities in the rendering
//! pipeline. It coordinates factory-based entity creation, manages rendering-specific
//! components, and ensures proper integration with Bevy's ECS architecture.
//!
//! ## Key responsibilities
//!
//! - **Factory coordination**: Manages unified factory systems for consistent entity
//!   creation across all world object types
//! - **Render preparation**: Transforms simulation data into render-ready components
//!   with proper [`Transform`], [`Visibility`], and material assignments
//! - **ECS integration**: Ensures proper component bundles and entity relationships
//!   for optimal Bevy rendering performance
//! - **Configuration management**: Applies [`GameConfig`] settings to control
//!   rendering quality, LOD behavior, and performance characteristics
//!
//! ## Architecture integration
//!
//! This module integrates with several core systems:
//!
//! - [`gameplay_sim`] provides the source simulation data
//! - [`engine_core`] supplies configuration and shared components
//! - [`game_core`] defines the component types and data structures
//! - Bevy's [`PbrBundle`] and rendering systems handle final visualization
//!
//! # Usage example
//!
//! ```rust
//! use bevy::prelude::*;
//! use gameplay_render::world::unified_factory_setup::unified_factory_setup_system;
//! use game_core::config::GameConfig;
//!
//! fn setup_world_rendering(app: &mut App) {
//!     app.add_systems(Startup, unified_factory_setup_system);
//! }
//!
//! fn spawn_world_entities(
//!     mut commands: Commands,
//!     config: Res<GameConfig>,
//! ) {
//!     // World factory systems automatically handle entity creation
//!     // based on configuration and simulation data
//! }
//! ```
//!
//! # Performance considerations
//!
//! The world module is optimized for large-scale entity creation and management:
//!
//! - Uses batch processing for efficient entity spawning
//! - Implements automatic LOD selection based on camera distance
//! - Integrates with the culling system to hide distant entities
//! - Supports instancing for repetitive world objects like vegetation
//!
//! # Implementation notes
//!
//! World rendering systems run during the [`Update`] schedule and coordinate
//! with the [`crate::plugins::BatchingPlugin`] for optimal performance. Factory
//! systems ensure consistent entity creation patterns across all world object
//! types, from buildings and vehicles to vegetation and environmental effects.

/// Unified factory setup system for initializing world entity creation resources.
pub mod unified_factory_setup;
