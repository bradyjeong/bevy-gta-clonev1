use bevy::prelude::*;
use std::sync::{Arc, RwLock, OnceLock};
use super::traits::*;
use super::container::ServiceContainer;

/// Global service locator for runtime service discovery
/// This provides a fallback when dependency injection is not available
static SERVICE_LOCATOR: OnceLock<Arc<RwLock<ServiceLocator>>> = OnceLock::new();

pub struct ServiceLocator {
    container: ServiceContainer,
}

impl ServiceLocator {
    /// Initialize the global service locator
    pub fn initialize() -> Arc<RwLock<Self>> {
        let locator = Arc::new(RwLock::new(Self {
            container: ServiceContainer::new(),
        }));
        
        SERVICE_LOCATOR.set(locator.clone()).unwrap_or_else(|_| {
            panic!("ServiceLocator already initialized");
        });
        
        info!("🌐 SERVICE LOCATOR: Initialized");
        locator
    }
    
    /// Get the global service locator instance
    pub fn instance() -> Arc<RwLock<Self>> {
        SERVICE_LOCATOR.get()
            .expect("ServiceLocator not initialized")
            .clone()
    }
    
    /// Register a service with the locator
    pub fn register<T: Service>(&mut self, service: T) {
        self.container.register(service);
    }
    
    /// Get a service from the locator
    pub fn get<T: Service + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        self.container.get::<T>()
    }
    
    /// Require a service from the locator (panics if not found)
    pub fn require<T: Service + 'static>(&self) -> Arc<RwLock<T>> {
        self.get::<T>().expect(&format!(
            "Required service not found in locator: {}",
            std::any::type_name::<T>()
        ))
    }
    
    /// Get the underlying container
    pub fn container(&self) -> &ServiceContainer {
        &self.container
    }
    
    /// Get a mutable reference to the underlying container
    pub fn container_mut(&mut self) -> &mut ServiceContainer {
        &mut self.container
    }
}

/// Convenience functions for global service access
pub mod global {
    use super::*;
    
    /// Get a service from the global locator
    pub fn get_service<T: Service + 'static>() -> Option<Arc<RwLock<T>>> {
        ServiceLocator::instance().read().unwrap().get::<T>()
    }
    
    /// Require a service from the global locator
    pub fn require_service<T: Service + 'static>() -> Arc<RwLock<T>> {
        ServiceLocator::instance().read().unwrap().require::<T>()
    }
    
    /// Register a service with the global locator
    pub fn register_service<T: Service>(service: T) {
        ServiceLocator::instance().write().unwrap().register(service);
    }
}

/// System for initializing the service locator
pub fn initialize_service_locator(mut commands: Commands) {
    let locator = ServiceLocator::initialize();
    
    // Convert the locator's container to a Bevy resource
    let container = {
        let _locator_guard = locator.read().unwrap();
        ServiceContainer::new() // We'll populate this separately
    };
    
    commands.insert_resource(container);
    info!("🔧 SERVICE LOCATOR: Added as Bevy resource");
}

/// System for registering core services with both the locator and container
pub fn register_core_services(
    mut container: ResMut<ServiceContainer>,
    config: Res<crate::config::GameConfig>,
) {
    use super::implementations::*;
    
    // Register services in the container
    container.register(DefaultConfigService::new(config.clone()));
    container.register(DefaultTimingService::new());
    container.register(DefaultAudioService::new());
    container.register(DefaultAssetService::new());
    container.register(DefaultPhysicsService::new(config.physics.clone()));
    container.register(DefaultLoggingService::new());
    
    // Also register in the global locator
    {
        let instance = ServiceLocator::instance();
        let mut locator = instance.write().unwrap();
        locator.register(DefaultConfigService::new(config.clone()));
        locator.register(DefaultTimingService::new());
        locator.register(DefaultAudioService::new());
        locator.register(DefaultAssetService::new());
        locator.register(DefaultPhysicsService::new(config.physics.clone()));
        locator.register(DefaultLoggingService::new());
    }
    
    container.initialize_services();
    info!("✅ CORE SERVICES: Registered and initialized");
}

/// Mock services for testing
#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::collections::HashMap;
    
    pub struct MockConfigService {
        config: crate::config::GameConfig,
    }
    
    impl MockConfigService {
        pub fn new() -> Self {
            Self {
                config: crate::config::GameConfig::default(),
            }
        }
    }
    
    impl Service for MockConfigService {
        fn service_name(&self) -> &'static str {
            "MockConfigService"
        }
    }
    
    impl ConfigService for MockConfigService {
        fn get_physics_config(&self) -> &crate::config::PhysicsConfig {
            &self.config.physics
        }
        
        fn get_world_config(&self) -> &crate::config::WorldConfig {
            &self.config.world
        }
        
        fn get_vehicle_config(&self) -> &crate::config::VehicleConfig {
            &self.config.vehicles
        }
        
        fn get_npc_config(&self) -> &crate::config::NPCConfig {
            &self.config.npc
        }
        
        fn get_performance_config(&self) -> &crate::config::PerformanceConfig {
            &self.config.performance
        }
        
        fn get_audio_config(&self) -> &crate::config::AudioConfig {
            &self.config.audio
        }
        
        
        
        fn get_camera_config(&self) -> &crate::config::CameraConfig {
            &self.config.camera
        }
        
        fn get_ui_config(&self) -> &crate::config::UIConfig {
            &self.config.ui
        }
        
        fn get_batching_config(&self) -> &crate::config::BatchingConfig {
            &self.config.batching
        }
        
        fn update_config(&mut self, config: crate::config::GameConfig) {
            self.config = config;
        }
        
        fn validate_and_clamp(&mut self) {
            self.config.validate_and_clamp();
        }
    }
    
    pub struct MockTimingService {
        current_time: f32,
    }
    
    impl MockTimingService {
        pub fn new() -> Self {
            Self { current_time: 0.0 }
        }
    }
    
    impl Service for MockTimingService {
        fn service_name(&self) -> &'static str {
            "MockTimingService"
        }
    }
    
    impl TimingService for MockTimingService {
        fn current_time(&self) -> f32 { self.current_time }
        fn delta_time(&self) -> f32 { 0.016 }
        fn should_run_system(&mut self, _system_type: SystemType) -> bool { true }
        fn register_entity(&mut self, _entity: Entity, _timer_type: EntityTimerType, _interval: f32) {}
        fn should_update_entity(&mut self, _entity: Entity) -> bool { true }
        fn unregister_entity(&mut self, _entity: Entity) {}
        fn get_stats(&self) -> TimingStats {
            TimingStats {
                tracked_entities: 0,
                current_time: self.current_time,
                delta_time: 0.016,
            }
        }
        fn update_time(&mut self, time: &Time) {
            self.current_time = time.elapsed_secs();
        }
    }
}
