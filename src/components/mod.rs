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

pub mod effects;
pub mod lod;
pub mod player;
pub mod vehicles;
pub mod water;
pub mod water_new;
pub mod unified_water;
pub mod world;

pub mod control_state;
pub mod dirty_flags;

pub mod instanced_vegetation;
pub mod safety;
pub mod unified_vehicle;

// Core entity components
pub use player::{
    ActiveEntity, BodyPart, HumanAnimation, HumanMovement, InCar, Player, PlayerBody,
    PlayerBodyMesh, PlayerHead, PlayerLeftArm, PlayerLeftLeg, PlayerRightArm, PlayerRightLeg,
    PlayerTorso,
};

pub use vehicles::{
    AircraftFlight, Car, F16, Helicopter, MainRotor, SimpleCarSpecs, SimpleF16Specs,
    SimpleHelicopterSpecs, TailRotor, VehicleHealth, VehicleLOD, VehicleRendering, VehicleState,
    VehicleType,
};

pub use world::{
    BoundaryEffects, Buildable, Building, BuildingType, ContentType, CullingSettings,
    DynamicContent, DynamicTerrain, IntersectionEntity, Landmark, MainCamera, MovementController,
    NPC, NPC_LOD_CULL_DISTANCE, NPCAppearance, NPCBehaviorComponent, NPCBehaviorType, NPCBodyPart,
    NPCGender, NPCHead, NPCLOD, NPCLeftArm, NPCLeftLeg, NPCRendering, NPCRightArm, NPCRightLeg,
    NPCState, NPCTorso, NPCType, PerformanceCritical, PerformanceStats, RoadEntity, WorldBounds,
};

pub use water::{Boat, Lake, WaterBody, WaterWave, Yacht};
pub use water_new::{WaterRegion, WaterBodyId, GlobalOcean, WaterRegionAsset, TideConfig, WaveParams};

// Visual and rendering components
pub use effects::{
    ControlsDisplay, ControlsText, ExhaustFlame, FlameEffect, JetFlame, VehicleBeacon, WaypointText,
};

pub use lod::{VegetationBillboard, VegetationDetailLevel, VegetationLOD, VegetationMeshLOD};

pub use instanced_vegetation::{
    InstanceData, InstancedBush, InstancedLeafCluster, InstancedPalmFrond, InstancedTreeTrunk,
    InstancedVegetationBundle, VegetationBatchable, VegetationInstancingConfig, VegetationType,
};

// Control and optimization components
pub use control_state::{AIControlled, ControlState, PlayerControlled, VehicleControlType};
pub use dirty_flags::{
    DirtyFlagsMetrics, DirtyLOD, DirtyVegetationInstancing, DirtyVisibility, FrameCounter,
};
pub use unified_vehicle::UnifiedVehicleSpecs;
