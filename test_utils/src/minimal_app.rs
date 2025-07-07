use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy_rapier3d::prelude::*;
use std::time::Duration;

/// Lightweight Bevy app for testing with minimal plugins
pub struct MinimalBevyApp {
    /// The Bevy app instance
    pub app: App,
}

impl MinimalBevyApp {
    /// Create a new minimal Bevy app for testing
    pub fn new() -> Self {
        let mut app = App::new();
        
        // Add just minimal plugins needed for testing
        app.add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin::default(),
            ScheduleRunnerPlugin::run_once(),
        ));
        
        Self { app }
    }
    
    /// Create a minimal app with physics
    pub fn with_physics() -> Self {
        let mut minimal_app = Self::new();
        minimal_app.app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        minimal_app
    }
    
    /// Create a minimal app with rendering (for screenshot tests)
    pub fn with_rendering() -> Self {
        let mut minimal_app = Self::new();
        minimal_app.app.add_plugins(DefaultPlugins);
        minimal_app
    }
    
    /// Add a system to the app
    pub fn add_simple_system(mut self, system: impl IntoSystem<(), (), ()> + 'static) -> Self {
        self.app.add_systems(Update, system);
        self
    }
    
    /// Add plugins to the app  
    pub fn add_plugin(mut self, plugin: impl Plugin) -> Self {
        self.app.add_plugins(plugin);
        self
    }
    
    /// Add a resource to the app
    pub fn add_resource<R: Resource>(mut self, resource: R) -> Self {
        self.app.insert_resource(resource);
        self
    }
    
    /// Run the app once
    pub fn run_once(mut self) {
        self.app.update();
    }
    
    /// Run the app for a specific duration
    pub fn run_for(mut self, duration: Duration) {
        let start = std::time::Instant::now();
        while start.elapsed() < duration {
            self.app.update();
        }
    }
    
    /// Get mutable access to the world
    pub fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }
    
    /// Get access to the world
    pub fn world(&self) -> &World {
        self.app.world()
    }
    
    /// Spawn an entity with components
    pub fn spawn_entity(&mut self, bundle: impl Bundle) -> Entity {
        self.app.world_mut().spawn(bundle).id()
    }
    
    /// Get a resource from the world
    pub fn resource<R: Resource>(&self) -> &R {
        self.app.world().resource::<R>()
    }
    
    /// Get a mutable resource from the world
    pub fn resource_mut<R: Resource>(&mut self) -> Mut<R> {
        self.app.world_mut().resource_mut::<R>()
    }
}

impl Default for MinimalBevyApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_minimal_app_creation() {
        let app = MinimalBevyApp::new();
        // Basic test - just verify app creation works
        assert_eq!(app.world().entities().len(), 0);
    }
    
    #[test]
    fn test_minimal_app_with_physics() {
        let app = MinimalBevyApp::with_physics();
        // Basic test - just verify physics app creation works
        assert_eq!(app.world().entities().len(), 0);
    }
    
    #[test]
    fn test_spawn_entity() {
        let mut app = MinimalBevyApp::new();
        let entity = app.spawn_entity(Transform::default());
        assert!(app.world().entities().contains(entity));
    }
}
