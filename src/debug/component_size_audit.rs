//! Component and Resource Size Audit Tool
//! 
//! P2 implementation: Audit Component & Resource Sizes from architectural_shift.md
//! 
//! This module provides compile-time and runtime size auditing for all components
//! and resources to ensure they follow AGENT.md performance guidelines:
//! - Hot-path components: ≤64 bytes (cache line size)
//! - Resources accessed frequently: ≤64 bytes
//! - Components: ≤10 fields guideline for maintainability

use bevy::prelude::*;
use std::mem::{size_of, align_of};

/// Size audit results for components and resources
#[derive(Resource, Default)]
pub struct SizeAuditResults {
    pub oversized_components: Vec<ComponentSizeInfo>,
    pub oversized_resources: Vec<ResourceSizeInfo>,
    pub immutable_candidates: Vec<String>,
    pub split_candidates: Vec<SplitCandidate>,
}

#[derive(Debug, Clone)]
pub struct ComponentSizeInfo {
    pub name: String,
    pub size: usize,
    pub alignment: usize,
    pub is_hot_path: bool,
    pub field_count_estimate: usize,
}

#[derive(Debug, Clone)]
pub struct ResourceSizeInfo {
    pub name: String,
    pub size: usize,
    pub alignment: usize,
    pub is_frequently_accessed: bool,
}

#[derive(Debug, Clone)]
pub struct SplitCandidate {
    pub name: String,
    pub size: usize,
    pub suggested_splits: Vec<String>,
    pub reasoning: String,
}

impl SizeAuditResults {
    pub fn audit_all_sizes() -> Self {
        let mut results = Self::default();
        
        // Audit core components
        results.audit_component::<crate::components::control_state::ControlState>("ControlState", true);
        results.audit_component::<crate::components::vehicles::SuperCarSpecs>("SuperCarSpecs", false);
        results.audit_component::<crate::components::vehicles::SuperCarSuspension>("SuperCarSuspension", false);
        results.audit_component::<crate::components::vehicles::TurboSystem>("TurboSystem", false);
        results.audit_component::<crate::components::vehicles::EngineState>("EngineState", false);
        results.audit_component::<crate::components::vehicles::Transmission>("Transmission", false);
        results.audit_component::<crate::components::vehicles::DrivingModes>("DrivingModes", false);
        results.audit_component::<crate::components::vehicles::AerodynamicsSystem>("AerodynamicsSystem", false);
        results.audit_component::<crate::components::vehicles::PerformanceMetrics>("PerformanceMetrics", false);
        results.audit_component::<crate::components::vehicles::ExhaustSystem>("ExhaustSystem", false);
        results.audit_component::<crate::components::vehicles::AircraftFlight>("AircraftFlight", true);
        results.audit_component::<crate::components::vehicles::F16Specs>("F16Specs", false);
        results.audit_component::<crate::components::vehicles::VehicleState>("VehicleState", true);
        
        // Audit world components
        results.audit_component::<crate::components::world::NPCBehaviorComponent>("NPCBehaviorComponent", false);
        results.audit_component::<crate::components::world::MovementController>("MovementController", true);
        results.audit_component::<crate::components::world::NPCState>("NPCState", true);
        results.audit_component::<crate::components::world::NPCAppearance>("NPCAppearance", false);
        results.audit_component::<crate::components::world::NPCRendering>("NPCRendering", false);
        results.audit_component::<crate::components::world::NPCBodyPart>("NPCBodyPart", false);
        results.audit_component::<crate::components::world::Building>("Building", false);
        results.audit_component::<crate::components::world::Cullable>("Cullable", true);
        
        // Audit optimized NPC components
        results.audit_component::<crate::components::npc_optimized::NPCCore>("NPCCore", true);
        results.audit_component::<crate::components::npc_optimized::NPCConfig>("NPCConfig", false);
        results.audit_component::<crate::components::npc_optimized::NPCVisualConfig>("NPCVisualConfig", false);
        
        // Audit world object components
        results.audit_component::<crate::components::world_object::WorldObject>("WorldObject", false);
        
        // Audit key resources
        results.audit_resource::<crate::components::world::CullingSettings>("CullingSettings", false);
        results.audit_resource::<crate::components::world::PerformanceStats>("PerformanceStats", true);
        results.audit_resource::<crate::components::world::MeshCache>("MeshCache", false);
        results.audit_resource::<crate::components::world::EntityLimits>("EntityLimits", false);
        results.audit_resource::<crate::world::placement_grid::PlacementGrid>("PlacementGrid", true);
        results.audit_resource::<crate::world::chunk_tracker::ChunkTracker>("ChunkTracker", true);
        results.audit_resource::<crate::world::chunk_tracker::ChunkTables>("ChunkTables", false);
        
        // Identify immutable component candidates
        results.identify_immutable_candidates();
        
        // Identify components that should be split
        results.identify_split_candidates();
        
        results
    }
    
    fn audit_component<T: Component>(&mut self, name: &str, is_hot_path: bool) {
        let size = size_of::<T>();
        let alignment = align_of::<T>();
        
        // Estimate field count based on size (rough heuristic)
        let field_count_estimate = (size / 4).max(1); // Assume average 4 bytes per field
        
        let info = ComponentSizeInfo {
            name: name.to_string(),
            size,
            alignment,
            is_hot_path,
            field_count_estimate,
        };
        
        // Flag as oversized if it violates guidelines
        if (is_hot_path && size > 64) || field_count_estimate > 10 {
            self.oversized_components.push(info);
        }
    }
    
    fn audit_resource<T: Resource>(&mut self, name: &str, is_frequently_accessed: bool) {
        let size = size_of::<T>();
        let alignment = align_of::<T>();
        
        let info = ResourceSizeInfo {
            name: name.to_string(),
            size,
            alignment,
            is_frequently_accessed,
        };
        
        // Flag as oversized if frequently accessed and > 64 bytes
        if is_frequently_accessed && size > 64 {
            self.oversized_resources.push(info);
        }
    }
    
    fn identify_immutable_candidates(&mut self) {
        // Components that never change after spawn should be marked immutable
        let candidates = vec![
            "F16Specs".to_string(),
            "SuperCarSpecs".to_string(), // Base specs don't change
            "NPCConfig".to_string(),
            "NPCVisualConfig".to_string(),
            "Building".to_string(), // Building properties don't change
            "WorldObject".to_string(), // Object type and radius are static
        ];
        
        self.immutable_candidates = candidates;
    }
    
    fn identify_split_candidates(&mut self) {
        // Identify components that should be split based on access patterns
        
        // NPCState is too large and mixes hot/cold data
        if let Some(npc_state) = self.oversized_components.iter().find(|c| c.name == "NPCState") {
            self.split_candidates.push(SplitCandidate {
                name: "NPCState".to_string(),
                size: npc_state.size,
                suggested_splits: vec![
                    "NPCCore (position, velocity, health) - hot path".to_string(),
                    "NPCVisuals (appearance, colors) - cold path".to_string(),
                    "NPCBehavior (behavior type, targets) - medium frequency".to_string(),
                ],
                reasoning: "Mixes frequently updated data (position, health) with static data (appearance)".to_string(),
            });
        }
        
        // SuperCar components could be further optimized
        let supercar_components = ["SuperCarSpecs", "SuperCarSuspension", "EngineState"];
        for &comp_name in &supercar_components {
            if let Some(comp) = self.oversized_components.iter().find(|c| c.name == comp_name) {
                if comp.size > 32 {
                    self.split_candidates.push(SplitCandidate {
                        name: comp_name.to_string(),
                        size: comp.size,
                        suggested_splits: vec![
                            format!("{}Core - frequently updated values", comp_name),
                            format!("{}Config - static configuration", comp_name),
                        ],
                        reasoning: "Large component with mixed access patterns".to_string(),
                    });
                }
            }
        }
    }
    
    pub fn print_audit_report(&self) {
        info!("=== COMPONENT & RESOURCE SIZE AUDIT REPORT ===");
        
        if !self.oversized_components.is_empty() {
            warn!("Oversized Components (>64 bytes for hot-path, >10 fields estimated):");
            for comp in &self.oversized_components {
                warn!(
                    "  {} - {} bytes ({} estimated fields) - Hot path: {}",
                    comp.name, comp.size, comp.field_count_estimate, comp.is_hot_path
                );
            }
        }
        
        if !self.oversized_resources.is_empty() {
            warn!("Oversized Frequently-Accessed Resources (>64 bytes):");
            for res in &self.oversized_resources {
                warn!(
                    "  {} - {} bytes - Frequently accessed: {}",
                    res.name, res.size, res.is_frequently_accessed
                );
            }
        }
        
        if !self.immutable_candidates.is_empty() {
            info!("Components that should be marked #[component(immutable)]:");
            for candidate in &self.immutable_candidates {
                info!("  {}", candidate);
            }
        }
        
        if !self.split_candidates.is_empty() {
            info!("Components that should be split:");
            for candidate in &self.split_candidates {
                info!("  {} ({} bytes):", candidate.name, candidate.size);
                info!("    Reason: {}", candidate.reasoning);
                for split in &candidate.suggested_splits {
                    info!("    Suggested: {}", split);
                }
            }
        }
        
        info!("=== END SIZE AUDIT REPORT ===");
    }
}

/// System to run size audit and log results
pub fn run_size_audit_system() {
    let audit_results = SizeAuditResults::audit_all_sizes();
    audit_results.print_audit_report();
}

// Compile-time size assertions for critical hot-path components

// Already validated in respective modules:
// - ControlState: ≤64 bytes ✓
// - ChunkTracker: ≤64 bytes ✓  
// - NPCCore: ≤64 bytes ✓

// Additional validations for other hot-path components
const _: () = assert!(
    size_of::<crate::components::world::MovementController>() <= 64,
    "MovementController exceeds 64-byte cache line"
);

const _: () = assert!(
    size_of::<crate::components::world::Cullable>() <= 64,
    "Cullable exceeds 64-byte cache line"
);

const _: () = assert!(
    size_of::<crate::components::vehicles::AircraftFlight>() <= 64,
    "AircraftFlight exceeds 64-byte cache line"
);

const _: () = assert!(
    size_of::<crate::components::vehicles::VehicleState>() <= 64,
    "VehicleState exceeds 64-byte cache line"
);

// Resource size validations for frequently accessed resources
const _: () = assert!(
    size_of::<crate::components::world::PerformanceStats>() <= 64,
    "PerformanceStats exceeds 64-byte cache line"
);
