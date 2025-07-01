use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use super::Service;

/// Simple service container without complex trait objects
#[derive(Resource, Default)]
pub struct SimpleServiceContainer {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl SimpleServiceContainer {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register<T: Service + 'static>(&mut self, service: T) {
        let type_id = TypeId::of::<T>();
        let wrapped_service = Arc::new(RwLock::new(service));
        self.services.insert(type_id, Box::new(wrapped_service));
    }

    pub fn get<T: Service + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        let type_id = TypeId::of::<T>();
        self.services.get(&type_id)?.downcast_ref::<Arc<RwLock<T>>>().cloned()
    }

    pub fn require<T: Service + 'static>(&self) -> Arc<RwLock<T>> {
        self.get::<T>().expect(&format!(
            "Required service not found: {}",
            std::any::type_name::<T>()
        ))
    }
}

#[derive(SystemParam)]
pub struct SimpleServices<'w> {
    container: Res<'w, SimpleServiceContainer>,
}

impl<'w> SimpleServices<'w> {
    pub fn get<T: Service + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        self.container.get::<T>()
    }
    
    pub fn require<T: Service + 'static>(&self) -> Arc<RwLock<T>> {
        self.container.require::<T>()
    }
}
