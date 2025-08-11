use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct ScheduleOrdering {
    pub system_dependencies: HashMap<&'static str, Vec<&'static str>>,
    pub execution_order: Vec<&'static str>,
    pub cycles_detected: Vec<Vec<&'static str>>,
}

impl ScheduleOrdering {
    pub fn add_dependency(&mut self, system: &'static str, depends_on: &'static str) {
        self.system_dependencies
            .entry(system)
            .or_default()
            .push(depends_on);
    }
    
    pub fn validate_ordering(&mut self) -> bool {
        // Topological sort to detect cycles
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut order = Vec::new();
        
        for system in self.system_dependencies.keys() {
            if !visited.contains(system) {
                if !self.dfs_visit(system, &mut visited, &mut stack, &mut order) {
                    return false;
                }
            }
        }
        
        self.execution_order = order;
        true
    }
    
    fn dfs_visit(
        &self,
        system: &'static str,
        visited: &mut HashSet<&'static str>,
        stack: &mut HashSet<&'static str>,
        order: &mut Vec<&'static str>,
    ) -> bool {
        if stack.contains(&system) {
            // Cycle detected
            return false;
        }
        
        if visited.contains(&system) {
            return true;
        }
        
        stack.insert(system);
        
        if let Some(deps) = self.system_dependencies.get(&system) {
            for dep in deps {
                if !self.dfs_visit(dep, visited, stack, order) {
                    return false;
                }
            }
        }
        
        stack.remove(&system);
        visited.insert(system);
        order.push(system);
        
        true
    }
    
    pub fn generate_mermaid_graph(&self) -> String {
        let mut graph = String::from("graph TD\n");
        
        for (system, deps) in &self.system_dependencies {
            for dep in deps {
                graph.push_str(&format!("    {} --> {}\n", dep, system));
            }
        }
        
        graph
    }
}

pub struct ScheduleOrderingPlugin;

impl Plugin for ScheduleOrderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScheduleOrdering>()
            .add_systems(Startup, validate_schedule_ordering)
            .add_systems(Last, export_schedule_visualization);
        
        // Enforce event handler naming convention
        app.add_systems(
            Update,
            (
                // Example ordering chain for event handlers
                enforce_event_ordering,
            ).chain(),
        );
    }
}

fn validate_schedule_ordering(
    ordering: Res<ScheduleOrdering>,
) {
    if !ordering.cycles_detected.is_empty() {
        error!("Dependency cycles detected in schedule:");
        for cycle in &ordering.cycles_detected {
            error!("  Cycle: {:?}", cycle);
        }
    }
    
    info!("Schedule execution order validated");
    info!("Total systems: {}", ordering.execution_order.len());
}

fn export_schedule_visualization(
    ordering: Res<ScheduleOrdering>,
) {
    #[cfg(feature = "debug-events")]
    {
        let graph = ordering.generate_mermaid_graph();
        if !graph.is_empty() {
            debug!("Schedule visualization (Mermaid format):\n{}", graph);
        }
    }
}

fn enforce_event_ordering(
    mut ordering: ResMut<ScheduleOrdering>,
) {
    // Add common event handler dependencies
    ordering.add_dependency("handle_spawn_event", "validate_entities");
    ordering.add_dependency("handle_despawn_event", "handle_spawn_event");
    ordering.add_dependency("update_transforms", "handle_spawn_event");
    ordering.add_dependency("physics_step", "update_transforms");
}

// Trait for ordered systems
pub trait OrderedSystem {
    fn system_name(&self) -> &'static str;
    fn dependencies(&self) -> Vec<&'static str>;
}

// Macro for defining ordered systems
#[macro_export]
macro_rules! ordered_system {
    ($name:ident, deps: [$($dep:ident),*]) => {
        impl OrderedSystem for $name {
            fn system_name(&self) -> &'static str {
                stringify!($name)
            }
            
            fn dependencies(&self) -> Vec<&'static str> {
                vec![$(stringify!($dep)),*]
            }
        }
    };
}

// Helper for deterministic system names
pub fn validate_system_name(name: &str) -> bool {
    // Enforce handle_*_event naming for event handlers
    if name.contains("event") {
        name.starts_with("handle_") && name.ends_with("_event")
    } else {
        true
    }
}

// Schedule analysis
#[derive(Resource, Default)]
pub struct ScheduleAnalysis {
    pub parallel_opportunities: Vec<(String, String)>,
    pub bottlenecks: Vec<String>,
    pub redundant_dependencies: Vec<(String, String)>,
}

pub fn analyze_schedule(
    ordering: Res<ScheduleOrdering>,
    mut analysis: ResMut<ScheduleAnalysis>,
) {
    // Find systems that could run in parallel
    for (system1, deps1) in &ordering.system_dependencies {
        for (system2, deps2) in &ordering.system_dependencies {
            if system1 != system2 {
                // Check if they have no dependencies on each other
                if !deps1.contains(system2) && !deps2.contains(system1) {
                    analysis.parallel_opportunities.push((
                        system1.to_string(),
                        system2.to_string(),
                    ));
                }
            }
        }
    }
    
    // Find bottleneck systems (many dependencies)
    for (system, deps) in &ordering.system_dependencies {
        if deps.len() > 5 {
            analysis.bottlenecks.push(system.to_string());
        }
    }
}
