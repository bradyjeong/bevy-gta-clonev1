use bevy::prelude::*;

use crate::services::{initialize_service_locator, register_core_services};

/// Plugin for service-based systems and dependency injection
pub struct ServicePlugin;

impl Plugin for ServicePlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize services first
            .add_systems(PreStartup, (
                initialize_service_locator,
                register_core_services,
            ).chain());
            
            // Add service-based example systems (temporarily disabled)
            // .add_systems(Update, (
            //     service_based_entity_creation_system,
            //     service_config_update_system,
            //     service_asset_cleanup_system,
            //     service_based_factory_system,
            // ));
            
        info!("🔧 SERVICE PLUGIN: Registered service-based systems");
    }
}
