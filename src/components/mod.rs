//! # Component Definitions
//!
//! This module contains all component definitions used throughout the game.
//! Components are pure data structures with no behavior - they only store state.
//!
//! ## Component Design Principles
//!
//! - **Data-Only**: Components contain no methods or logic
//! - **Single Purpose**: Each component represents one specific aspect of an entity
//! - **Composition**: Complex entities combine multiple simple components
//! - **Default Implementation**: All components should derive Default when possible
//!
//! ## Component Categories
//!
//! ### Entity Identity
//! - `player`: Player character markers and state
//! - `vehicles`: Vehicle types and properties
//! - `world`: Terrain and world structure markers
//!
//! ### Visual & Rendering
//! - `effects`: Visual effect data and parameters
//! - `water`: Water simulation properties
//! - `lod`: Level-of-detail rendering data
//! - `instanced_vegetation`: Efficient vegetation rendering
//!
//! ### Optimization
//! - `dirty_flags`: Change tracking for selective updates
//! - `realistic_vehicle`: Detailed vehicle physics parameters
//!
//! ## Component Usage Patterns
//!
//! ```rust
//! // Simple entity creation
//! commands.spawn((
//!     PlayerComponent::default(),
//!     MovementComponent { speed: 5.0 },
//!     Transform::default(),
//! ));
//!
//! // Query for entities with specific components
//! fn player_movement_system(
//!     mut query: Query<(&mut Transform, &PlayerComponent, &MovementComponent)>
//! ) {
//!     // System logic here
//! }
//! ```
//!
//! ## Adding New Components
//!
//! 1. Create the component struct in the appropriate module
//! 2. Derive `Component` and `Default` when possible
//! 3. Keep fields public for ECS access
//! 4. Add documentation for each field
//! 5. Export from this mod.rs file

pub mod player;
pub mod vehicles;
pub mod world;
pub mod effects;
pub mod water;
pub mod lod;

pub mod realistic_vehicle;
pub mod dirty_flags;
pub mod instanced_vegetation;
pub mod control_state;
pub mod world_object;
pub mod dynamic_content;
pub mod npc_optimized;

// Re-export Bevy's ChildOf for hierarchy relationships
pub use bevy::ecs::hierarchy::ChildOf;

pub use player::*;
pub use vehicles::*;
pub use world::{
    NPCBehaviorComponent, MovementController, BuildingType, NPC, NPCType, NPCLOD,
    NPCCore, NPCVisuals, NPCAppearance, NPCGender, NPCBehaviorType, NPCRendering,
    NPCHead, NPCTorso, NPCLeftArm, NPCRightArm, NPCLeftLeg, NPCRightLeg, NPCBodyPart,
    NPC_LOD_FULL_DISTANCE, NPC_LOD_MEDIUM_DISTANCE, NPC_LOD_LOW_DISTANCE, NPC_LOD_CULL_DISTANCE,
    Cullable, RoadEntity, IntersectionEntity, DynamicTerrain, DynamicContent, ContentType,
    PerformanceCritical, Building, Landmark, Buildable, MainCamera, CullingSettings,
    PerformanceStats, MeshCache, EntityLimits
};
pub use effects::*;
pub use water::*;
pub use lod::*;

pub use realistic_vehicle::*;
pub use dirty_flags::*;
pub use instanced_vegetation::*;
pub use control_state::*;
pub use world_object::*;
pub use npc_optimized::*;
