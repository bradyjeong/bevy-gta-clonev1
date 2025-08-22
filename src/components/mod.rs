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
//! - `entity_types`: World entity classification for LOD and spawning
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


pub mod dirty_flags;
pub mod instanced_vegetation;
pub mod control_state;
pub mod safety;
pub mod unified_vehicle;
pub mod entity_types;

pub use player::*;
pub use vehicles::*;
pub use world::*;
pub use effects::*;
pub use water::*;
pub use lod::*;


pub use dirty_flags::*;
pub use instanced_vegetation::*;
pub use control_state::*;
// pub use safety::*; // Disabled - conflicts with world::WorldBounds in finite world
pub use unified_vehicle::*;
pub use entity_types::*;
