//! P2 Component & Resource Size Audit Implementation
//! 
//! Implements architectural_shift.md requirements:
//! 1. Audit Component & Resource Sizes (>64 bytes or >10 fields)
//! 2. Apply #[component(immutable)] where appropriate
//! 3. Split big components that change at different rates
//! 4. Performance validation

use std::mem::size_of;
use bevy::prelude::*;

/// Complete size audit results
#[derive(Debug)]
pub struct ComponentSizeAuditReport {
    pub oversized_hot_components: Vec<ComponentSizeInfo>,
    pub oversized_warm_components: Vec<ComponentSizeInfo>,
    pub immutable_candidates: Vec<ImmutableCandidate>,
    pub split_candidates: Vec<SplitCandidate>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

#[derive(Debug)]
pub struct ComponentSizeInfo {
    pub name: &'static str,
    pub size: usize,
    pub field_count: usize,
    pub file_location: &'static str,
    pub access_pattern: AccessPattern,
    pub recommendations: Vec<String>,
}

#[derive(Debug)]
pub struct ImmutableCandidate {
    pub name: &'static str,
    pub reason: &'static str,
    pub file_location: &'static str,
}

#[derive(Debug)]
pub struct SplitCandidate {
    pub name: &'static str,
    pub size: usize,
    pub field_count: usize,
    pub hot_fields: Vec<&'static str>,
    pub cold_fields: Vec<&'static str>,
    pub recommended_split: String,
}

#[derive(Debug)]
pub struct OptimizationOpportunity {
    pub component: &'static str,
    pub opportunity_type: OptimizationType,
    pub description: String,
    pub potential_savings: usize,
}

#[derive(Debug)]
pub enum AccessPattern {
    HotPath,    // Every frame (<64 bytes required)
    Warm,       // Frequent access (<128 bytes recommended)
    Cold,       // Infrequent access (size not critical)
}

#[derive(Debug)]
pub enum OptimizationType {
    BitPacking,     // Pack booleans into flags
    FieldReduction, // Remove redundant fields
    ComponentSplit, // Split hot/cold data
    ImmutableMarker, // Add immutable annotation
    TypeOptimization, // Use smaller types (f64->f32)
}

/// Run comprehensive P2 size audit
pub fn run_p2_size_audit() -> ComponentSizeAuditReport {
    let mut report = ComponentSizeAuditReport {
        oversized_hot_components: Vec::new(),
        oversized_warm_components: Vec::new(),
        immutable_candidates: Vec::new(),
        split_candidates: Vec::new(),
        optimization_opportunities: Vec::new(),
    };

    // ========================================================================
    // HOT-PATH COMPONENT AUDIT (≤64 bytes)
    // ========================================================================
    
    audit_hot_component::<crate::components::control_state::ControlState>(
        &mut report, "ControlState", "components/control_state.rs", 13
    );
    
    audit_hot_component::<crate::components::NPCCore>(
        &mut report, "NPCCore", "components/world.rs", 7
    );

    // ========================================================================
    // WARM-PATH COMPONENT AUDIT (≤128 bytes recommended)
    // ========================================================================
    
    audit_warm_component::<crate::components::vehicles::SuperCarSpecs>(
        &mut report, "SuperCarSpecs", "components/vehicles.rs", 7
    );
    
    audit_warm_component::<crate::components::vehicles::F16Specs>(
        &mut report, "F16Specs", "components/vehicles.rs", 19
    );

    audit_warm_component::<crate::components::vehicles::VehicleState>(
        &mut report, "VehicleState", "components/vehicles.rs", 8
    );

    // ========================================================================
    // RESOURCE AUDIT
    // ========================================================================
    
    audit_hot_resource::<crate::world::chunk_tracker::ChunkTracker>(
        &mut report, "ChunkTracker", "world/chunk_tracker.rs"
    );
    
    audit_hot_resource::<crate::world::placement_grid::PlacementGrid>(
        &mut report, "PlacementGrid", "world/placement_grid.rs"
    );

    // ========================================================================
    // IMMUTABLE CANDIDATE IDENTIFICATION
    // ========================================================================
    
    identify_immutable_candidates(&mut report);

    // ========================================================================
    // COMPONENT SPLITTING ANALYSIS
    // ========================================================================
    
    analyze_component_splitting(&mut report);

    // ========================================================================
    // OPTIMIZATION OPPORTUNITIES
    // ========================================================================
    
    identify_optimization_opportunities(&mut report);

    report
}

fn audit_hot_component<T>(
    report: &mut ComponentSizeAuditReport,
    name: &'static str,
    file_location: &'static str,
    field_count: usize,
) {
    let size = size_of::<T>();
    let mut recommendations = Vec::new();
    
    if size > 64 {
        recommendations.push(format!("CRITICAL: {} bytes exceeds 64-byte cache line", size));
        recommendations.push("Consider component splitting (hot/cold data)".to_string());
        recommendations.push("Use bit packing for boolean fields".to_string());
        recommendations.push("Consider f32 instead of f64 where appropriate".to_string());
        
        report.oversized_hot_components.push(ComponentSizeInfo {
            name,
            size,
            field_count,
            file_location,
            access_pattern: AccessPattern::HotPath,
            recommendations,
        });
    }
    
    if field_count > 10 {
        recommendations.push(format!("Field count {} exceeds 10-field guideline", field_count));
    }
}

fn audit_warm_component<T>(
    report: &mut ComponentSizeAuditReport,
    name: &'static str,
    file_location: &'static str,
    field_count: usize,
) {
    let size = size_of::<T>();
    let mut recommendations = Vec::new();
    
    if size > 128 {
        recommendations.push(format!("WARNING: {} bytes is large for warm-path", size));
        recommendations.push("Consider splitting state vs config data".to_string());
        
        report.oversized_warm_components.push(ComponentSizeInfo {
            name,
            size,
            field_count,
            file_location,
            access_pattern: AccessPattern::Warm,
            recommendations,
        });
    }
}

fn audit_hot_resource<T>(
    report: &mut ComponentSizeAuditReport,
    name: &'static str,
    file_location: &'static str,
) {
    let size = size_of::<T>();
    
    if size > 64 {
        let mut recommendations = Vec::new();
        recommendations.push(format!("Resource {} ({} bytes) exceeds hot-path limit", name, size));
        recommendations.push("Consider caching frequently accessed data".to_string());
        
        report.oversized_hot_components.push(ComponentSizeInfo {
            name,
            size,
            field_count: 0, // Resources don't have direct field counting
            file_location,
            access_pattern: AccessPattern::HotPath,
            recommendations,
        });
    }
}

fn identify_immutable_candidates(report: &mut ComponentSizeAuditReport) {
    // Components that should be marked immutable (never change after spawn)
    let candidates = vec![
        ImmutableCandidate {
            name: "SuperCarSpecs",
            reason: "Performance specifications are static configuration",
            file_location: "components/vehicles.rs",
        },
        ImmutableCandidate {
            name: "F16Specs", 
            reason: "Aircraft specifications are static configuration",
            file_location: "components/vehicles.rs",
        },
        ImmutableCandidate {
            name: "Building",
            reason: "Building properties don't change after spawn",
            file_location: "components/world.rs",
        },
        ImmutableCandidate {
            name: "NPCVisuals",
            reason: "NPC appearance is set at spawn and never changes",
            file_location: "components/world.rs",
        },
        ImmutableCandidate {
            name: "NPCConfig",
            reason: "NPC configuration data is static",
            file_location: "components/npc_optimized.rs",
        },
        ImmutableCandidate {
            name: "NPCVisualConfig",
            reason: "Visual configuration is immutable",
            file_location: "components/npc_optimized.rs",
        },
    ];
    
    report.immutable_candidates.extend(candidates);
}

fn analyze_component_splitting(report: &mut ComponentSizeAuditReport) {
    // Analyze components that could benefit from hot/cold splitting
    
    // ControlState analysis - if it becomes oversized
    if size_of::<crate::components::control_state::ControlState>() > 48 {
        report.split_candidates.push(SplitCandidate {
            name: "ControlState",
            size: size_of::<crate::components::control_state::ControlState>(),
            field_count: 13,
            hot_fields: vec!["throttle", "brake", "steering", "pitch", "roll", "yaw"],
            cold_fields: vec!["button_states", "interaction_input", "exit_input"],
            recommended_split: "Split into ControlInputs (hot) and ControlButtons (warm)".to_string(),
        });
    }
    
    // Vehicle component analysis
    if size_of::<crate::components::vehicles::VehicleState>() > 64 {
        report.split_candidates.push(SplitCandidate {
            name: "VehicleState",
            size: size_of::<crate::components::vehicles::VehicleState>(),
            field_count: 8,
            hot_fields: vec!["current_lod", "last_lod_check"],
            cold_fields: vec!["vehicle_type", "color", "max_speed", "acceleration"],
            recommended_split: "Split into VehicleLOD (hot) and VehicleConfig (cold)".to_string(),
        });
    }
}

fn identify_optimization_opportunities(report: &mut ComponentSizeAuditReport) {
    // Control state bit packing opportunity
    report.optimization_opportunities.push(OptimizationOpportunity {
        component: "ControlState",
        opportunity_type: OptimizationType::BitPacking,
        description: "Pack boolean buttons into bitfield flags".to_string(),
        potential_savings: 8, // Multiple bool -> single u8
    });
    
    // F16Specs precision optimization
    report.optimization_opportunities.push(OptimizationOpportunity {
        component: "F16Specs",
        opportunity_type: OptimizationType::TypeOptimization,
        description: "Use f32 instead of f64 for physics calculations".to_string(),
        potential_savings: size_of::<f64>() - size_of::<f32>(), // 4 bytes per f64->f32
    });
    
    // Immutable marker opportunities
    for candidate in &report.immutable_candidates {
        report.optimization_opportunities.push(OptimizationOpportunity {
            component: candidate.name,
            opportunity_type: OptimizationType::ImmutableMarker,
            description: format!("Add #[component(immutable)] - {}", candidate.reason),
            potential_savings: 0, // Performance gain, not size
        });
    }
}

/// Generate comprehensive optimization report
pub fn generate_p2_report(report: &ComponentSizeAuditReport) -> String {
    let mut output = String::new();
    
    output.push_str("=== P2 COMPONENT & RESOURCE SIZE AUDIT REPORT ===\n\n");
    
    // Hot-path violations
    if !report.oversized_hot_components.is_empty() {
        output.push_str("🔴 CRITICAL: Hot-path components exceeding 64 bytes:\n");
        for comp in &report.oversized_hot_components {
            output.push_str(&format!(
                "  • {} ({} bytes, {} fields) - {}\n",
                comp.name, comp.size, comp.field_count, comp.file_location
            ));
            for rec in &comp.recommendations {
                output.push_str(&format!("    - {}\n", rec));
            }
        }
        output.push('\n');
    }
    
    // Warm-path warnings  
    if !report.oversized_warm_components.is_empty() {
        output.push_str("🟡 WARNING: Warm-path components exceeding 128 bytes:\n");
        for comp in &report.oversized_warm_components {
            output.push_str(&format!(
                "  • {} ({} bytes, {} fields) - {}\n",
                comp.name, comp.size, comp.field_count, comp.file_location
            ));
        }
        output.push('\n');
    }
    
    // Immutable candidates
    output.push_str("📌 IMMUTABLE MARKER CANDIDATES:\n");
    for candidate in &report.immutable_candidates {
        output.push_str(&format!(
            "  • {} - {} ({})\n",
            candidate.name, candidate.reason, candidate.file_location
        ));
    }
    output.push('\n');
    
    // Component splitting recommendations
    if !report.split_candidates.is_empty() {
        output.push_str("✂️ COMPONENT SPLITTING RECOMMENDATIONS:\n");
        for split in &report.split_candidates {
            output.push_str(&format!(
                "  • {} ({} bytes, {} fields)\n",
                split.name, split.size, split.field_count
            ));
            output.push_str(&format!("    Hot: {:?}\n", split.hot_fields));
            output.push_str(&format!("    Cold: {:?}\n", split.cold_fields));
            output.push_str(&format!("    Recommendation: {}\n", split.recommended_split));
        }
        output.push('\n');
    }
    
    // Optimization opportunities
    output.push_str("⚡ OPTIMIZATION OPPORTUNITIES:\n");
    for opt in &report.optimization_opportunities {
        let savings_str = if opt.potential_savings > 0 {
            format!(" (saves {} bytes)", opt.potential_savings)
        } else {
            String::new()
        };
        output.push_str(&format!(
            "  • {}: {}{}\n",
            opt.component, opt.description, savings_str
        ));
    }
    
    output
}

/// Static assertions for P2 compliance
pub mod static_assertions {
    use super::*;
    
    // Hot-path component assertions (≤64 bytes)
    const _: () = assert!(
        size_of::<crate::components::control_state::ControlState>() <= 64,
        "ControlState exceeds 64-byte cache line for hot-path component"
    );
    
    const _: () = assert!(
        size_of::<crate::components::NPCCore>() <= 64,
        "NPCCore exceeds 64-byte cache line for hot-path component"
    );
    
    // Hot-path resource assertions (≤64 bytes)
    const _: () = assert!(
        size_of::<crate::world::chunk_tracker::ChunkTracker>() <= 64,
        "ChunkTracker exceeds 64-byte limit for hot-path resource"
    );
    
    const _: () = assert!(
        size_of::<crate::world::placement_grid::PlacementGrid>() <= 64,
        "PlacementGrid exceeds 64-byte limit for hot-path resource"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_p2_size_audit() {
        let report = run_p2_size_audit();
        println!("{}", generate_p2_report(&report));
        
        // Validate that we have reasonable audit results
        assert!(!report.immutable_candidates.is_empty(), "Should identify immutable candidates");
        assert!(!report.optimization_opportunities.is_empty(), "Should identify optimization opportunities");
    }
    
    #[test]
    fn test_critical_hot_path_sizes() {
        // Ensure critical hot-path components meet requirements
        assert!(
            size_of::<crate::components::control_state::ControlState>() <= 64,
            "ControlState must be ≤64 bytes for cache efficiency"
        );
        
        assert!(
            size_of::<crate::components::NPCCore>() <= 64,
            "NPCCore must be ≤64 bytes for cache efficiency"
        );
    }
}
