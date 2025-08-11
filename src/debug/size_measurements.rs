//! Comprehensive size measurements for all components and resources
//! 
//! This module contains the actual size measurements and static assertions
//! for all types in the codebase.

use crate::components::*;
use crate::resources::*;
use crate::services::*;
use crate::world::*;
use crate::debug::size_audit::{SizeAuditReport, TypeClassification};

// ============================================================================
// HOT-PATH COMPONENT SIZE ASSERTIONS
// Components accessed every frame must be ≤64 bytes for cache efficiency
// ============================================================================

// NOTE: ControlState is 52 bytes - acceptable for hot-path
const _: () = assert!(
    std::mem::size_of::<control_state::ControlState>() <= 64,
    "ControlState exceeds 64 bytes for hot-path component"
);

// NOTE: HumanMovement is 36 bytes - optimal
const _: () = assert!(
    std::mem::size_of::<humans::HumanMovement>() <= 64,
    "HumanMovement exceeds 64 bytes for hot-path component"
);

// NOTE: HumanAnimation is 36 bytes - optimal
const _: () = assert!(
    std::mem::size_of::<humans::HumanAnimation>() <= 64,
    "HumanAnimation exceeds 64 bytes for hot-path component"
);

// NOTE: SharedMovementTracker is 28 bytes - optimal
const _: () = assert!(
    std::mem::size_of::<vehicles::SharedMovementTracker>() <= 64,
    "SharedMovementTracker exceeds 64 bytes for hot-path component"
);

// NOTE: VegetationLOD is 16 bytes - optimal
const _: () = assert!(
    std::mem::size_of::<vegetation::VegetationLOD>() <= 64,
    "VegetationLOD exceeds 64 bytes for hot-path component"
);

// ============================================================================
// HOT-PATH RESOURCE SIZE ASSERTIONS
// Resources accessed every frame must be ≤64 bytes for cache efficiency
// ============================================================================

// ChunkTracker already has its own assertion in the module
// PlacementGrid is 24 bytes - optimal
const _: () = assert!(
    std::mem::size_of::<placement_grid::PlacementGrid>() <= 64,
    "PlacementGrid exceeds 64 bytes for hot-path resource"
);

// GroundDetectionService is ~16 bytes - optimal
const _: () = assert!(
    std::mem::size_of::<ground_detection::GroundDetectionService>() <= 64,
    "GroundDetectionService exceeds 64 bytes for hot-path resource"
);

// ============================================================================
// COMPREHENSIVE SIZE AUDIT FUNCTION
// ============================================================================

/// Run a complete size audit of all components and resources
pub fn run_comprehensive_audit() -> SizeAuditReport {
    let mut report = SizeAuditReport::new();
    
    // ========================================================================
    // MARKER COMPONENTS (0 bytes)
    // ========================================================================
    
    use TypeClassification::*;
    
    // Player markers
    report.add_component::<Player>("Player", Marker, "components/mod.rs", "Player entity marker");
    report.add_component::<ActiveEntity>("ActiveEntity", Marker, "components/mod.rs", "Active entity marker");
    
    // Vehicle markers
    report.add_component::<vehicles::Car>("Car", Marker, "components/vehicles.rs", "Car marker");
    report.add_component::<vehicles::Helicopter>("Helicopter", Marker, "components/vehicles.rs", "Helicopter marker");
    report.add_component::<aircraft::F16>("F16", Marker, "components/aircraft.rs", "F16 marker");
    report.add_component::<vehicles::Yacht>("Yacht", Marker, "components/vehicles.rs", "Yacht marker");
    
    // Body part markers
    report.add_component::<humans::PlayerHead>("PlayerHead", Marker, "components/humans.rs", "Head marker");
    report.add_component::<humans::PlayerTorso>("PlayerTorso", Marker, "components/humans.rs", "Torso marker");
    report.add_component::<humans::PlayerLeftArm>("PlayerLeftArm", Marker, "components/humans.rs", "Left arm marker");
    report.add_component::<humans::PlayerRightArm>("PlayerRightArm", Marker, "components/humans.rs", "Right arm marker");
    
    // ========================================================================
    // HOT-PATH COMPONENTS (accessed every frame)
    // ========================================================================
    
    report.add_component::<control_state::ControlState>(
        "ControlState", HotPath, "components/control_state.rs",
        "Primary input state (52 bytes) - consider bit packing"
    );
    
    report.add_component::<humans::HumanMovement>(
        "HumanMovement", HotPath, "components/humans.rs",
        "Movement state (36 bytes) - optimal size"
    );
    
    report.add_component::<humans::HumanAnimation>(
        "HumanAnimation", HotPath, "components/humans.rs",
        "Animation state (36 bytes) - optimal size"
    );
    
    report.add_component::<vehicles::SharedMovementTracker>(
        "SharedMovementTracker", HotPath, "components/vehicles.rs",
        "Movement tracking (28 bytes) - optimal size"
    );
    
    report.add_component::<vegetation::VegetationLOD>(
        "VegetationLOD", HotPath, "components/vegetation.rs",
        "LOD state (16 bytes) - optimal size"
    );
    
    // ========================================================================
    // WARM PATH COMPONENTS (frequently accessed)
    // ========================================================================
    
    report.add_component::<vehicles::VehicleState>(
        "VehicleState", Warm, "components/vehicles.rs",
        "Vehicle state (~32 bytes) - consider splitting LOD"
    );
    
    report.add_component::<vehicles::SuperCarSpecs>(
        "SuperCarSpecs", Warm, "components/vehicles.rs",
        "Vehicle specs (28 bytes) - optimal"
    );
    
    report.add_component::<vehicles::EngineState>(
        "EngineState", Warm, "components/vehicles.rs",
        "Engine parameters (36 bytes) - optimal"
    );
    
    report.add_component::<aircraft::AircraftFlight>(
        "AircraftFlight", Warm, "components/aircraft.rs",
        "Flight controls (32 bytes) - optimal"
    );
    
    report.add_component::<aircraft::F16Specs>(
        "F16Specs", Warm, "components/aircraft.rs",
        "Aircraft specs (64 bytes) - borderline, consider optimization"
    );
    
    report.add_component::<npc::NPCState>(
        "NPCState", Warm, "components/npc.rs",
        "NPC state (~120 bytes) - OVERSIZED, needs splitting"
    );
    
    report.add_component::<npc::NPCAppearance>(
        "NPCAppearance", Warm, "components/npc.rs",
        "NPC visuals (52 bytes) - acceptable"
    );
    
    // ========================================================================
    // CACHE/CONFIG COMPONENTS (infrequent access)
    // ========================================================================
    
    report.add_component::<culling::UnifiedCullable>(
        "UnifiedCullable", Cache, "components/culling.rs",
        "Culling config (~200+ bytes) - large but acceptable for cache"
    );
    
    report.add_component::<instanced_vegetation::InstancedPalmFrond>(
        "InstancedPalmFrond", Cache, "components/instanced_vegetation.rs",
        "Vec<InstanceData> - can be very large (MB+)"
    );
    
    report.add_component::<instanced_vegetation::InstancedLeafCluster>(
        "InstancedLeafCluster", Cache, "components/instanced_vegetation.rs",
        "Vec<InstanceData> - can be very large (MB+)"
    );
    
    report.add_component::<instanced_vegetation::VegetationBatchable>(
        "VegetationBatchable", Cache, "components/instanced_vegetation.rs",
        "Batching data (48 bytes) - acceptable"
    );
    
    report.add_component::<vehicles::SuperCarSuspension>(
        "SuperCarSuspension", Cache, "components/vehicles.rs",
        "Suspension config (28 bytes) - optimal"
    );
    
    report.add_component::<vehicles::TurboSystem>(
        "TurboSystem", Cache, "components/vehicles.rs",
        "Turbo state (32 bytes) - optimal"
    );
    
    report.add_component::<vehicles::Transmission>(
        "Transmission", Cache, "components/vehicles.rs",
        "Gear ratios (28 bytes + Vec) - acceptable"
    );
    
    report.add_component::<humans::PlayerBody>(
        "PlayerBody", Cache, "components/humans.rs",
        "Body config (84 bytes) - acceptable for cache"
    );
    
    // ========================================================================
    // HOT-PATH RESOURCES
    // ========================================================================
    
    report.add_resource::<chunk_tracker::ChunkTracker>(
        "ChunkTracker", HotPath, "world/chunk_tracker.rs",
        "Optimized to 64 bytes with static assertion"
    );
    
    report.add_resource::<placement_grid::PlacementGrid>(
        "PlacementGrid", HotPath, "world/placement_grid.rs",
        "Bitfield grid (24 bytes) - optimal"
    );
    
    report.add_resource::<ground_detection::GroundDetectionService>(
        "GroundDetectionService", HotPath, "services/ground_detection.rs",
        "Service state (16 bytes) - optimal"
    );
    
    report.add_resource::<global_rng::GlobalRng>(
        "GlobalRng", HotPath, "resources/global_rng.rs",
        "RNG state (≤32 bytes) - optimal"
    );
    
    // ========================================================================
    // WARM PATH RESOURCES
    // ========================================================================
    
    #[cfg(feature = "debug-ui")]
    report.add_resource::<crate::observers::content_observers::ObserverMetrics>(
        "ObserverMetrics", Warm, "observers/content_observers.rs",
        "Metrics tracking (~32 bytes) - optimal"
    );
    
    // ========================================================================
    // CACHE RESOURCES
    // ========================================================================
    
    report.add_resource::<distance_cache::DistanceCache>(
        "DistanceCache", Cache, "services/distance_cache.rs",
        "HashMap cache - large but necessary"
    );
    
    report.add_resource::<timing_service::TimingService>(
        "TimingService", Cache, "services/timing_service.rs",
        "Timer management - HashMap based"
    );
    
    report.add_resource::<chunk_tracker::ChunkTables>(
        "ChunkTables", Cache, "world/chunk_tracker.rs",
        "Dynamic chunk data - unbounded HashMaps"
    );
    
    #[cfg(feature = "debug-ui")]
    report.add_resource::<crate::debug::event_audit::EventAuditStats>(
        "EventAuditStats", Cache, "debug/event_audit.rs",
        "Event tracking - HashMap based"
    );
    
    report
}

/// Apply immutable markers to components that don't change after spawn
pub fn apply_immutable_markers() {
    // Components that should be marked immutable:
    // - Vehicle specs (SuperCarSpecs, F16Specs, etc.)
    // - NPC appearance data
    // - Static configuration components
    // - Marker components (already zero-sized)
    
    // This would require modifying the component definitions
    // to add #[component(immutable)] attribute
}

/// Generate optimization recommendations based on size audit
pub fn generate_optimization_recommendations(report: &SizeAuditReport) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Find oversized hot-path components
    for entry in &report.components {
        if entry.classification == TypeClassification::HotPath && entry.size > 64 {
            recommendations.push(format!(
                "CRITICAL: {} ({} bytes) exceeds 64-byte cache line. Consider:\n  \
                 - Splitting into multiple components\n  \
                 - Using bit packing for booleans\n  \
                 - Reducing field precision (f64 -> f32)",
                entry.name, entry.size
            ));
        }
    }
    
    // Find warm-path components that could be optimized
    for entry in &report.components {
        if entry.classification == TypeClassification::Warm && entry.size > 128 {
            recommendations.push(format!(
                "WARNING: {} ({} bytes) is large for warm-path. Consider:\n  \
                 - Component splitting (state vs config)\n  \
                 - Moving infrequent fields to separate component\n  \
                 - Using entity references instead of embedded data",
                entry.name, entry.size
            ));
        }
    }
    
    // Find components with Vec allocations
    if report.components.iter().any(|e| e.notes.contains("Vec")) {
        recommendations.push(
            "INFO: Components with Vec allocations can cause memory fragmentation. \
             Consider using fixed-size arrays or separate entity hierarchies.".to_string()
        );
    }
    
    recommendations
}
