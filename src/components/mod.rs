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
pub mod map;
pub mod movement_tracker;
pub mod navigation_lights;
pub mod player;
pub mod propeller;
pub mod rotor_wash;
pub mod underwater_settings;
pub mod unified_water;
pub mod vehicles;
pub mod water;
pub mod water_material;
pub mod world;

pub mod control_state;
pub mod dirty_flags;
pub mod input_smoother;
pub mod swimming_events;
pub mod yacht_exit;

pub mod debug;
pub mod unified_vehicle;

// Core entity components
pub use player::{
    ActiveEntity, BodyPart, HumanAnimation, HumanMovement, InCar, Player, PlayerBody,
    PlayerBodyMesh, PlayerHead, PlayerLeftArm, PlayerLeftLeg, PlayerRightArm, PlayerRightLeg,
    PlayerTorso,
};

pub use navigation_lights::{LandingLight, NavigationLight, NavigationLightType};
pub use propeller::PropellerHub;
pub use rotor_wash::RotorWash;

pub use vehicles::{
    AircraftFlight, Car, CarWheelsConfig, F16, Grounded, HeliState, Helicopter, HelicopterRuntime,
    HelicopterVisualBody, MainRotor, RotorBlurDisk, SimpleCarSpecs, SimpleCarSpecsHandle,
    SimpleF16Specs, SimpleF16SpecsHandle, SimpleHelicopterSpecs, SimpleHelicopterSpecsHandle,
    TailRotor, VehicleHealth, VehicleLOD, VehicleRendering, VehicleState, VehicleType, VisualRig,
    VisualRigRoot, WheelMesh, WheelPos, WheelSteerPivot, WheelsRoot,
};

pub use world::{
    BoundaryEffects, Buildable, Building, BuildingType, ContentType, CullingSettings,
    DynamicContent, DynamicTerrain, IntersectionEntity, Landmark, MainCamera, MaterialCache,
    MovementController, NPC, NPCAppearance, NPCBehaviorComponent, NPCBehaviorType, NPCBodyPart,
    NPCGender, NPCHead, NPCLOD, NPCLeftArm, NPCLeftFoot, NPCLeftLeg, NPCRendering, NPCRightArm,
    NPCRightFoot, NPCRightLeg, NPCState, NPCTorso, NPCType, PerformanceCritical, PerformanceStats,
    RoadEntity, WorldBounds,
};

pub use unified_water::{
    CurrentWaterRegion, TideConfig, UnifiedWaterAsset, UnifiedWaterBody, WaterSurface, WaveParams,
};
pub use water::{Boat, WaterBody, WaterWave, Yacht};
pub use water_material::WaterMaterial;

// Visual and rendering components
pub use effects::{
    ControlsDisplay, ControlsText, FlameEffect, JetFlame, VehicleBeacon, WaypointText,
};

// Control and optimization components
pub use control_state::{
    AIControlled, ControlState, PendingPhysicsEnable, PlayerControlled, VehicleControlType,
};
pub use debug::MissingSpecsWarned;
pub use dirty_flags::{DirtyFlagsMetrics, DirtyLOD, DirtyVisibility};
pub use input_smoother::InputSmoother;
pub use map::{MapCamera, MapConfig, MinimapUI, PlayerMapIcon};
pub use movement_tracker::MovementTracker;
pub use swimming_events::SwimmingEvent;
pub use underwater_settings::UnderwaterSettings;
pub use unified_vehicle::UnifiedVehicleSpecs;
pub use yacht_exit::{
    DeckWalkAnchor, DeckWalkable, DeckWalker, DockedOnYacht, DockingCooldown, Enterable, ExitPoint,
    ExitPointKind, Helipad, LandedOnYacht,
};
