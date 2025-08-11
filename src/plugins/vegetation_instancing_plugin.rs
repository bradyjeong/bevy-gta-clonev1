use bevy::prelude::*;
use crate::systems::rendering::vegetation_instancing::{
    collect_vegetation_instances_system,
    update_vegetation_instancing_system,
    mark_vegetation_instancing_dirty_system,
    animate_vegetation_instances_system,
};

#[cfg(feature = "debug-ui")]
use crate::systems::rendering::vegetation_instancing::vegetation_instancing_metrics_system;
use crate::systems::vegetation_instancing_integration::{
    integrate_vegetation_with_instancing_system,
};
use crate::system_sets::GameSystemSets;

/// Plugin responsible for vegetation instancing and rendering optimization
pub struct VegetationInstancingPlugin;

impl Plugin for VegetationInstancingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                // Core instancing pipeline
                collect_vegetation_instances_system
                    .in_set(GameSystemSets::ServiceUpdates),
                mark_vegetation_instancing_dirty_system
                    .in_set(GameSystemSets::ServiceUpdates),
                update_vegetation_instancing_system
                    .in_set(GameSystemSets::ServiceUpdates)
                    .after(collect_vegetation_instances_system),
                
                // Integration and animation
                integrate_vegetation_with_instancing_system
                    .in_set(GameSystemSets::ServiceUpdates)
                    .after(update_vegetation_instancing_system),
                animate_vegetation_instances_system
                    .in_set(GameSystemSets::ServiceUpdates),
                
                // Performance monitoring
                #[cfg(feature = "debug-ui")]
                vegetation_instancing_metrics_system
                    .in_set(GameSystemSets::ServiceUpdates),
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vegetation_instancing_plugin_instantiation() {
        // Test that plugin can be instantiated
        let plugin = VegetationInstancingPlugin;
        let mut app = App::new();
        
        // Test that adding plugin doesn't panic during registration
        app.add_plugins(MinimalPlugins);
        
        // Just test the build method doesn't panic
        plugin.build(&mut app);
    }
}
