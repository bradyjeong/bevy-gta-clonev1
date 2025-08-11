//! Component and Resource Size Audit Module
//! 
//! Provides automated size measurement, classification, and optimization recommendations
//! for all components and resources in the codebase.

use bevy::prelude::*;
use std::mem::size_of;
use std::collections::HashMap;

/// Classification of a type based on its access pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeClassification {
    /// Accessed every frame, must be cache-optimized (≤64 bytes)
    HotPath,
    /// Accessed frequently but not every frame (≤128 bytes recommended)
    Warm,
    /// Configuration or cache data, size less critical
    Cache,
    /// Zero-sized marker type
    Marker,
}

/// Audit entry for a single type
#[derive(Debug, Clone)]
pub struct SizeAuditEntry {
    pub name: &'static str,
    pub size: usize,
    pub classification: TypeClassification,
    pub priority: u8,  // 0-10, higher = more important to optimize
    pub location: &'static str,
    pub notes: &'static str,
}

/// Size audit report generator
#[derive(Resource)]
pub struct SizeAuditReport {
    pub components: Vec<SizeAuditEntry>,
    pub resources: Vec<SizeAuditEntry>,
}

impl SizeAuditReport {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Add a component to the audit
    pub fn add_component<T: Component>(
        &mut self,
        name: &'static str,
        classification: TypeClassification,
        location: &'static str,
        notes: &'static str,
    ) {
        let size = size_of::<T>();
        let priority = Self::calculate_priority(size, classification);
        
        self.components.push(SizeAuditEntry {
            name,
            size,
            classification,
            priority,
            location,
            notes,
        });
    }

    /// Add a resource to the audit
    pub fn add_resource<T: Resource>(
        &mut self,
        name: &'static str,
        classification: TypeClassification,
        location: &'static str,
        notes: &'static str,
    ) {
        let size = size_of::<T>();
        let priority = Self::calculate_priority(size, classification);
        
        self.resources.push(SizeAuditEntry {
            name,
            size,
            classification,
            priority,
            location,
            notes,
        });
    }

    /// Calculate optimization priority (0-10 scale)
    fn calculate_priority(size: usize, classification: TypeClassification) -> u8 {
        match classification {
            TypeClassification::Marker => 0,  // Already optimal
            TypeClassification::Cache => {
                // Cache types have low priority unless extremely large
                if size > 1024 { 3 } else { 1 }
            }
            TypeClassification::Warm => {
                // Warm path: moderate priority if over 128 bytes
                if size > 256 { 7 }
                else if size > 128 { 5 }
                else if size > 64 { 3 }
                else { 1 }
            }
            TypeClassification::HotPath => {
                // Hot path: high priority if over 64 bytes
                if size > 256 { 10 }
                else if size > 128 { 9 }
                else if size > 64 { 7 }
                else { 0 }  // Already optimal
            }
        }
    }

    /// Generate a formatted report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Component & Resource Size Audit Report\n\n");
        report.push_str("Generated: Size Audit System\n\n");
        
        // Components section
        report.push_str("## Components\n\n");
        report.push_str("| Name | Size | Class | Priority | Location | Notes |\n");
        report.push_str("|------|------|-------|----------|----------|-------|\n");
        
        let mut sorted_components = self.components.clone();
        sorted_components.sort_by_key(|e| (std::cmp::Reverse(e.priority), e.size));
        
        for entry in &sorted_components {
            report.push_str(&format!(
                "| {} | {} | {:?} | {} | {} | {} |\n",
                entry.name, entry.size, entry.classification, entry.priority,
                entry.location, entry.notes
            ));
        }
        
        // Resources section
        report.push_str("\n## Resources\n\n");
        report.push_str("| Name | Size | Class | Priority | Location | Notes |\n");
        report.push_str("|------|------|-------|----------|----------|-------|\n");
        
        let mut sorted_resources = self.resources.clone();
        sorted_resources.sort_by_key(|e| (std::cmp::Reverse(e.priority), e.size));
        
        for entry in &sorted_resources {
            report.push_str(&format!(
                "| {} | {} | {:?} | {} | {} | {} |\n",
                entry.name, entry.size, entry.classification, entry.priority,
                entry.location, entry.notes
            ));
        }
        
        // Summary statistics
        report.push_str("\n## Summary\n\n");
        
        let hot_path_components: Vec<_> = self.components.iter()
            .filter(|e| e.classification == TypeClassification::HotPath)
            .collect();
        
        let oversized_hot_path: Vec<_> = hot_path_components.iter()
            .filter(|e| e.size > 64)
            .collect();
        
        report.push_str(&format!("- Total components: {}\n", self.components.len()));
        report.push_str(&format!("- Total resources: {}\n", self.resources.len()));
        report.push_str(&format!("- Hot-path components: {}\n", hot_path_components.len()));
        report.push_str(&format!("- Oversized hot-path components (>64 bytes): {}\n", oversized_hot_path.len()));
        
        if !oversized_hot_path.is_empty() {
            report.push_str("\n### Optimization Candidates (Priority 7+)\n\n");
            for entry in oversized_hot_path {
                report.push_str(&format!("- **{}** ({} bytes): {}\n", entry.name, entry.size, entry.notes));
            }
        }
        
        report
    }
}

/// Macro for measuring and documenting component sizes
#[macro_export]
macro_rules! audit_component {
    ($report:expr, $type:ty, $class:expr, $location:expr, $notes:expr) => {
        $report.add_component::<$type>(
            stringify!($type),
            $class,
            $location,
            $notes,
        );
    };
}

/// Macro for measuring and documenting resource sizes
#[macro_export]
macro_rules! audit_resource {
    ($report:expr, $type:ty, $class:expr, $location:expr, $notes:expr) => {
        $report.add_resource::<$type>(
            stringify!($type),
            $class,
            $location,
            $notes,
        );
    };
}

/// Static assertion for hot-path component sizes
#[macro_export]
macro_rules! assert_hot_path_size {
    ($type:ty) => {
        const _: () = assert!(
            std::mem::size_of::<$type>() <= 64,
            concat!(stringify!($type), " exceeds 64 bytes for hot-path component")
        );
    };
}

/// Static assertion for hot-path resource sizes
#[macro_export]
macro_rules! assert_hot_resource_size {
    ($type:ty) => {
        const _: () = assert!(
            std::mem::size_of::<$type>() <= 64,
            concat!(stringify!($type), " exceeds 64 bytes for hot-path resource")
        );
    };
}

/// Plugin for running size audits
pub struct SizeAuditPlugin;

impl Plugin for SizeAuditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, run_size_audit);
    }
}

/// System that runs the size audit and generates the report
pub fn run_size_audit(mut commands: Commands) {
    info!("Running component and resource size audit...");
    
    let mut report = SizeAuditReport::new();
    
    // Audit all components
    use crate::components::*;
    use crate::debug::size_audit::TypeClassification::*;
    
    // Hot-path components (accessed every frame)
    audit_component!(report, control_state::ControlState, HotPath, 
        "components/control_state.rs", "Primary input state - optimize field packing");
    audit_component!(report, humans::HumanMovement, HotPath,
        "components/humans.rs", "Movement state - consider bit packing booleans");
    audit_component!(report, humans::HumanAnimation, HotPath,
        "components/humans.rs", "Animation state - frequently updated");
    audit_component!(report, vehicles::SharedMovementTracker, HotPath,
        "components/vehicles.rs", "Movement tracking - accessed every physics frame");
    
    // Marker components (zero-sized)
    audit_component!(report, Player, Marker,
        "components/mod.rs", "Zero-sized marker");
    audit_component!(report, vehicles::Car, Marker,
        "components/vehicles.rs", "Zero-sized marker");
    audit_component!(report, vehicles::Helicopter, Marker,
        "components/vehicles.rs", "Zero-sized marker");
    
    // Warm path components (accessed frequently)
    audit_component!(report, vehicles::VehicleState, Warm,
        "components/vehicles.rs", "Vehicle state - consider splitting LOD info");
    audit_component!(report, npc::NPCState, Warm,
        "components/npc.rs", "NPC state - large, consider component splitting");
    audit_component!(report, vegetation::VegetationLOD, Warm,
        "components/vegetation.rs", "LOD state for vegetation");
    
    // Cache/config components (infrequent access)
    audit_component!(report, culling::UnifiedCullable, Cache,
        "components/culling.rs", "Large culling config - acceptable for cache");
    audit_component!(report, instanced_vegetation::InstancedPalmFrond, Cache,
        "components/instanced_vegetation.rs", "Vec allocation - can be very large");
    
    // Audit all resources
    use crate::resources::*;
    use crate::services::*;
    use crate::world::*;
    
    // Hot-path resources
    audit_resource!(report, chunk_tracker::ChunkTracker, HotPath,
        "world/chunk_tracker.rs", "Optimized to 64 bytes with static assertion");
    audit_resource!(report, placement_grid::PlacementGrid, HotPath,
        "world/placement_grid.rs", "Bitfield-based spatial grid");
    audit_resource!(report, ground_detection::GroundDetectionService, HotPath,
        "services/ground_detection.rs", "Small service state");
    
    // Cache resources
    audit_resource!(report, distance_cache::DistanceCache, Cache,
        "services/distance_cache.rs", "Large HashMap cache - acceptable");
    audit_resource!(report, timing_service::TimingService, Cache,
        "services/timing_service.rs", "Timer management - infrequent updates");
    
    // Generate and save the report
    let report_text = report.generate_report();
    
    // Write to file
    std::fs::write("size_audit_report.md", &report_text)
        .expect("Failed to write size audit report");
    
    info!("Size audit complete. Report saved to size_audit_report.md");
    
    // Store report as resource for runtime access
    commands.insert_resource(report);
}
