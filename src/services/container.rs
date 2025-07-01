use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};

use super::traits::*;

/// Service container for managing shared services
#[derive(Resource)]
pub struct ServiceContainer {
    services: HashMap<TypeId, Arc<RwLock<dyn Any + Send + Sync>>>,
    service_names: HashMap<TypeId, &'static str>,
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceContainer {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            service_names: HashMap::new(),
        }
    }
    
    /// Register a service implementation
    pub fn register<T: Service>(&mut self, service: T) {
        let type_id = TypeId::of::<T>();
        let service_name = service.service_name();
        let boxed_service: Arc<RwLock<dyn Any + Send + Sync>> = Arc::new(RwLock::new(service));
        
        self.services.insert(type_id, boxed_service);
        self.service_names.insert(type_id, service_name);
        
        info!("🔧 SERVICE REGISTERED: {}", service_name);
    }
    
    /// Get a service implementation (read-only)
    pub fn get<T: Service + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        let type_id = TypeId::of::<T>();
        self.services.get(&type_id).and_then(|service| {
            // Unsafe but necessary for simple downcast
            let ptr = Arc::as_ptr(service) as *const RwLock<T>;
            Some(unsafe { Arc::from_raw(ptr) })
        })
    }
    
    /// Check if a service is registered
    pub fn has_service<T: Service + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.services.contains_key(&type_id)
    }
    
    /// Get all registered service names
    pub fn get_service_names(&self) -> Vec<&'static str> {
        self.service_names.values().copied().collect()
    }
    
    /// Initialize all services (call during startup)
    pub fn initialize_services(&self) {
        info!("🚀 INITIALIZING SERVICES: {} registered", self.services.len());
        for service_name in self.get_service_names() {
            info!("   ✓ {}", service_name);
        }
    }
}

/// Dependency injection helper for systems
pub struct ServiceDependencies<'a> {
    container: &'a ServiceContainer,
}

impl<'a> ServiceDependencies<'a> {
    pub fn new(container: &'a ServiceContainer) -> Self {
        Self { container }
    }
    
    /// Get a service dependency
    pub fn get<T: Service + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        self.container.get::<T>()
    }
    
    /// Require a service dependency (panics if not found)
    pub fn require<T: Service + 'static>(&self) -> Arc<RwLock<T>> {
        self.get::<T>().expect(&format!(
            "Required service not found: {}",
            std::any::type_name::<T>()
        ))
    }
}

/// System parameter for injecting services
#[derive(SystemParam)]
pub struct Services<'w> {
    container: Res<'w, ServiceContainer>,
}

impl<'w> Services<'w> {
    pub fn get<T: Service + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        self.container.get::<T>()
    }
    
    pub fn require<T: Service + 'static>(&self) -> Arc<RwLock<T>> {
        self.container.get::<T>().expect(&format!(
            "Required service not found: {}",
            std::any::type_name::<T>()
        ))
    }
    
    pub fn dependencies(&self) -> ServiceDependencies {
        ServiceDependencies::new(&self.container)
    }
}

/// Macro for easy service injection in systems
#[macro_export]
macro_rules! inject_service {
    ($services:expr, $service_type:ty) => {
        $services.require::<$service_type>()
    };
}

/// Macro for optional service injection
#[macro_export]
macro_rules! inject_service_optional {
    ($services:expr, $service_type:ty) => {
        $services.get::<$service_type>()
    };
}
