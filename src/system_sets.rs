use bevy::prelude::*;

// # System Ordering and Organization
//
// This module defines the system sets that control execution order throughout the game.
// Proper system ordering is critical for maintaining data consistency and avoiding
// race conditions in the ECS architecture.
//
// ## Why Explicit Ordering Matters
//
// Bevy's ECS runs systems in parallel by default, but some operations must happen
// in a specific sequence:
// - Services must initialize before world setup
// - Core world structure must exist before spawning entities
// - Physics updates must complete before rendering
//
// ## System Set Dependencies
//
// ```text
// ServiceInit
//     ↓
// WorldSetup (depends on ServiceInit)
//     ↓
// SecondarySetup (depends on WorldSetup)
//     ↓
// ServiceUpdates (runtime, depends on SecondarySetup)
// ```
//
// ## Adding New Systems
//
// When adding a new system, determine its dependency requirements:
//
// 1. **ServiceInit**: Core services (distance cache, entity limits, timing)
// 2. **WorldSetup**: Terrain, roads, basic world structure
// 3. **SecondarySetup**: NPCs, vehicles, vegetation, decorative elements
// 4. **ServiceUpdates**: Ongoing runtime systems (culling, physics, input)
//
// Use `.in_set()` to assign systems to the appropriate set:
//
// ```rust
// app.add_systems(Update, (
//     my_system.in_set(GameSystemSets::ServiceUpdates),
//     another_system.in_set(GameSystemSets::WorldSetup),
// ));
// ```
//
// ## Inter-Set Communication
//
// Sets communicate exclusively through:
// - **Resources**: Shared state that persists across frames
// - **Events**: One-time messages between systems
// - **Components**: Data attached to entities
//
// Avoid direct system-to-system calls or global variables.

/// System sets for organizing startup and update systems.
///
/// These sets define the execution order of systems throughout the game lifecycle.
/// Each set has clear dependencies and responsibilities to maintain system integrity.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSystemSets {
    /// Service initialization - runs first during startup.
    ///
    /// Initializes core services like:
    /// - Distance caching service
    /// - Entity limit management
    /// - Timing services
    /// - Performance monitoring
    ///
    /// All other sets depend on these services being available.
    ServiceInit,

    /// Core world setup - depends on services being initialized.
    ///
    /// Sets up the fundamental world structure:
    /// - Terrain generation
    /// - Road networks
    /// - Core lighting and camera setup
    /// - Physics world initialization
    ///
    /// Must run after ServiceInit, before SecondarySetup.
    WorldSetup,

    /// Secondary world setup - populates the world with entities.
    ///
    /// Spawns dynamic content that depends on the core world:
    /// - NPCs and pedestrians
    /// - Vehicles and traffic
    /// - Vegetation and decorative elements
    /// - Interactive objects
    ///
    /// Must run after WorldSetup, before ServiceUpdates.
    SecondarySetup,

    /// Runtime service updates - ongoing systems during gameplay.
    ///
    /// Handles continuous game operations:
    /// - Entity culling based on distance
    /// - Physics simulation updates
    /// - Input processing
    /// - UI updates
    /// - Performance monitoring
    ///
    /// Runs every frame after initial setup is complete.
    ServiceUpdates,
}

/// World event flow system sets for explicit ordering of dynamic content events.
///
/// These sets ensure deterministic processing of world events within a single frame:
/// 1. Query systems identify spawn candidates
/// 2. Validation requests are sent for road/terrain checks  
/// 3. Validation responses are processed
/// 4. Valid spawn requests are emitted
/// 5. Spawn requests are executed to create entities
///
/// This guarantees 0-frame latency for the complete validation→spawn pipeline.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum WorldEventFlow {
    /// Dynamic content querying - identifies potential spawn locations
    SpawnQuery,
    
    /// Spawn validation transmission - sends validation requests
    SpawnValidationTx,
    
    /// Road validation processing - validates against road/terrain constraints
    RoadValidation,
    
    /// Spawn validation reception - processes validation results
    SpawnValidationRx,
    
    /// Spawn emission - converts validation results to spawn requests
    SpawnEmit,
    
    /// Spawn execution - creates entities from spawn requests
    SpawnExecute,
}
