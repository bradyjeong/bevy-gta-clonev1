use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

/// Create a deterministic test world with fixed seed
#[must_use]
pub fn spawn_test_world(seed: u64) -> World {
    let mut world = World::new();
    
    // Initialize RNG with fixed seed for deterministic tests
    let rng = StdRng::seed_from_u64(seed);
    world.insert_resource(TestRng(rng));
    
    // Add basic resources
    world.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
    world.insert_resource(Time::<bevy::time::Real>::default());
    world.insert_resource(Time::<Virtual>::default());
    
    world
}

/// Test RNG resource for deterministic testing
#[derive(Resource)]
pub struct TestRng(pub StdRng);

#[allow(missing_docs)]
impl TestRng {
    pub fn next_f32(&mut self) -> f32 {
        self.0.r#gen()
    }
    
    pub fn next_u32(&mut self) -> u32 {
        self.0.r#gen()
    }
    
    pub fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
        R: rand::distributions::uniform::SampleRange<T>,
    {
        self.0.gen_range(range)
    }
}

/// Type alias for complex component function type
type ComponentFn = Box<dyn Fn(&mut EntityWorldMut)>;

/// Helper to create test entities with common components
pub struct EntityBuilder {
    components: Vec<ComponentFn>,
}

#[allow(missing_docs)]
impl EntityBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { components: Vec::new() }
    }
    
    #[must_use]
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.components.push(Box::new(move |entity| {
            entity.insert(transform);
        }));
        self
    }
    
    #[must_use]
    pub fn with_rigid_body(mut self, rb_type: RigidBody) -> Self {
        self.components.push(Box::new(move |entity| {
            entity.insert(rb_type);
        }));
        self
    }
    
    #[must_use]
    pub fn with_collider(mut self, collider: Collider) -> Self {
        self.components.push(Box::new(move |entity| {
            entity.insert(collider.clone());
        }));
        self
    }
    
    #[must_use]
    pub fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.components.push(Box::new(move |entity| {
            entity.insert(velocity);
        }));
        self
    }
    
    pub fn spawn(self, world: &mut World) -> Entity {
        let entity = world.spawn_empty().id();
        {
            let mut entity_mut = world.entity_mut(entity);
            for component_fn in self.components {
                component_fn(&mut entity_mut);
            }
        }
        entity
    }
}

impl Default for EntityBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create test scenarios with multiple entities
pub struct ScenarioBuilder {
    entities: Vec<(String, EntityBuilder)>,
}

#[allow(missing_docs)]
impl ScenarioBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { entities: Vec::new() }
    }
    
    #[must_use]
    pub fn add_entity(mut self, name: impl Into<String>, builder: EntityBuilder) -> Self {
        self.entities.push((name.into(), builder));
        self
    }
    
    pub fn spawn_all(self, world: &mut World) -> HashMap<String, Entity> {
        let mut entity_map = HashMap::new();
        
        for (name, builder) in self.entities {
            let entity = builder.spawn(world);
            entity_map.insert(name, entity);
        }
        
        entity_map
    }
}

impl Default for ScenarioBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for test world validation
/// 
/// # Errors
/// 
/// Returns an error if the world state is invalid:
/// - Too many entities (>10,000)
/// - NaN values in transforms
/// - Invalid physics velocities
pub fn validate_world_state(world: &mut World) -> Result<(), String> {
    // Check for orphaned entities
    let orphaned_count = world.entities().len();
    if orphaned_count > 10000 {
        return Err(format!("Too many entities in world: {orphaned_count}"));
    }
    
    // Check for NaN values in transforms
    for (entity, transform) in world.query::<(Entity, &Transform)>().iter(world) {
        if !transform.translation.is_finite() {
            return Err(format!("Entity {entity:?} has invalid translation: {:?}", transform.translation));
        }
        if !transform.rotation.is_finite() {
            return Err(format!("Entity {entity:?} has invalid rotation: {:?}", transform.rotation));
        }
        if !transform.scale.is_finite() {
            return Err(format!("Entity {entity:?} has invalid scale: {:?}", transform.scale));
        }
    }
    
    // Check for invalid physics bodies
    for (entity, velocity) in world.query::<(Entity, &Velocity)>().iter(world) {
        if !velocity.linvel.is_finite() {
            return Err(format!("Entity {entity:?} has invalid linear velocity: {:?}", velocity.linvel));
        }
        if !velocity.angvel.is_finite() {
            return Err(format!("Entity {entity:?} has invalid angular velocity: {:?}", velocity.angvel));
        }
    }
    
    Ok(())
}

/// Create a simple test vehicle for testing
pub fn create_test_vehicle(world: &mut World, position: Vec3) -> Entity {
    EntityBuilder::new()
        .with_transform(Transform::from_translation(position))
        .with_rigid_body(RigidBody::Dynamic)
        .with_collider(Collider::cuboid(2.0, 1.0, 4.0))
        .with_velocity(Velocity::zero())
        .spawn(world)
}

/// Create a test building for testing
pub fn create_test_building(world: &mut World, position: Vec3, size: Vec3) -> Entity {
    EntityBuilder::new()
        .with_transform(Transform::from_translation(position))
        .with_rigid_body(RigidBody::Fixed)
        .with_collider(Collider::cuboid(size.x, size.y, size.z))
        .spawn(world)
}

/// Create a test ground plane
pub fn create_test_ground(world: &mut World) -> Entity {
    EntityBuilder::new()
        .with_transform(Transform::from_translation(Vec3::ZERO))
        .with_rigid_body(RigidBody::Fixed)
        .with_collider(Collider::cuboid(1000.0, 0.1, 1000.0))
        .spawn(world)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spawn_test_world() {
        let world = spawn_test_world(42);
        assert!(world.contains_resource::<TestRng>());
    }
    
    #[test]
    fn test_entity_builder() {
        let mut world = spawn_test_world(42);
        let entity = EntityBuilder::new()
            .with_transform(Transform::from_xyz(1.0, 2.0, 3.0))
            .spawn(&mut world);
        
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    }
    
    #[test]
    fn test_scenario_builder() {
        let mut world = spawn_test_world(42);
        let entities = ScenarioBuilder::new()
            .add_entity("vehicle", EntityBuilder::new().with_transform(Transform::from_xyz(0.0, 0.0, 0.0)))
            .add_entity("building", EntityBuilder::new().with_transform(Transform::from_xyz(10.0, 0.0, 0.0)))
            .spawn_all(&mut world);
        
        assert_eq!(entities.len(), 2);
        assert!(entities.contains_key("vehicle"));
        assert!(entities.contains_key("building"));
    }
    
    #[test]
    fn test_validate_world_state() {
        let mut world = spawn_test_world(42);
        assert!(validate_world_state(&mut world).is_ok());
    }
}
