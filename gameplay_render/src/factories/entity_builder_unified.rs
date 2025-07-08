use bevy::prelude::*;
use game_core::components::*;
use crate::factories::entity_factory_unified::UnifiedEntityFactory;
use crate::factories::generic_bundle::*;

/// Unified Entity Builder - Fluent API for the unified factory system
/// Provides type-safe, chainable entity construction with validation
pub struct EntityBuilder<'a> {
    factory: &'a UnifiedEntityFactory,
    commands: &'a mut Commands<'a, 'a>,
    meshes: &'a mut ResMut<'a, Assets<Mesh>>,
    materials: &'a mut ResMut<'a, Assets<StandardMaterial>>,
}

impl<'a> EntityBuilder<'a> {
    /// Creates a new entity builder with access to the unified factory system.
    ///
    /// This function initializes the builder with references to all required
    /// resources for entity creation, including the factory configuration,
    /// ECS commands, and asset storage systems.
    ///
    /// # Arguments
    /// * `factory` - Reference to the unified entity factory with configuration
    /// * `commands` - Mutable reference to Bevy's ECS command system
    /// * `meshes` - Mutable reference to mesh asset storage
    /// * `materials` - Mutable reference to material asset storage
    ///
    /// # Returns
    /// A new [`EntityBuilder`] instance ready for fluent entity construction
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::entity_builder_unified::EntityBuilder;
    /// use gameplay_render::factories::entity_factory_unified::UnifiedEntityFactory;
    ///
    /// fn spawn_entities(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    ///     factory: Res<UnifiedEntityFactory>,
    /// ) {
    ///     let builder = EntityBuilder::new(&factory, &mut commands, &mut meshes, &mut materials);
    ///     // Use builder to create entities with fluent API
    /// }
    /// ```
    pub fn new(
        factory: &'a UnifiedEntityFactory,
        commands: &'a mut Commands<'a, 'a>,
        meshes: &'a mut ResMut<'a, Assets<Mesh>>,
        materials: &'a mut ResMut<'a, Assets<StandardMaterial>>,
    ) -> Self {
        Self {
            factory,
            commands,
            meshes,
            materials,
        }
    }
    
    /// Start building a vehicle
    pub fn vehicle(self, vehicle_type: VehicleType) -> VehicleBuilder<'a> {
        VehicleBuilder::new(self, vehicle_type)
    }
    
    /// Start building a realistic vehicle
    pub fn realistic_vehicle(self, vehicle_type: RealisticVehicleType) -> RealisticVehicleBuilder<'a> {
        RealisticVehicleBuilder::new(self, vehicle_type)
    }
    
    /// Start building an NPC
    pub fn npc(self) -> NPCBuilder<'a> {
        NPCBuilder::new(self)
    }
    
    /// Start building a building
    pub fn building(self, building_type: BuildingType) -> BuildingBuilder<'a> {
        BuildingBuilder::new(self, building_type)
    }
    
    /// Start building terrain
    pub fn terrain(self) -> TerrainBuilder<'a> {
        TerrainBuilder::new(self)
    }
    
    /// Start building a water body
    pub fn water(self) -> WaterBuilder<'a> {
        WaterBuilder::new(self)
    }
    
    /// Start building a tree
    pub fn tree(self) -> TreeBuilder<'a> {
        TreeBuilder::new(self)
    }
    
    /// Start building a particle effect
    pub fn particle_effect(self, effect_type: ParticleEffectType) -> ParticleBuilder<'a> {
        ParticleBuilder::new(self, effect_type)
    }
    
    /// Start building a road
    pub fn road(self) -> RoadBuilder<'a> {
        RoadBuilder::new(self)
    }
    
    /// Create batch builder for efficient bulk operations
    pub fn batch(self) -> BatchBuilder<'a> {
        BatchBuilder::new(self)
    }
}

/// Vehicle builder with fluent API
pub struct VehicleBuilder<'a> {
    builder: EntityBuilder<'a>,
    vehicle_type: VehicleType,
    position: Option<Vec3>,
    color: Option<Color>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> VehicleBuilder<'a> {
    /// Creates a new vehicle builder with the specified vehicle type.
    ///
    /// This function initializes the builder with the given vehicle type and
    /// default configuration. The builder can then be customized using the
    /// fluent API methods before spawning the final entity.
    ///
    /// # Arguments
    /// * `builder` - The base entity builder with factory and resource access
    /// * `vehicle_type` - The type of vehicle to create
    ///
    /// # Returns
    /// A new [`VehicleBuilder`] instance ready for customization
    fn new(builder: EntityBuilder<'a>, vehicle_type: VehicleType) -> Self {
        Self {
            builder,
            vehicle_type,
            position: None,
            color: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set vehicle position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Set vehicle color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    
    /// Add custom component to vehicle
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the vehicle entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(Vec3::ZERO);
        let color = self.color.unwrap_or_else(|| {
            match self.vehicle_type {
                VehicleType::BasicCar => self.builder.factory.config.vehicles.basic_car.default_color,
                VehicleType::SuperCar => self.builder.factory.config.vehicles.super_car.default_color,
                VehicleType::Helicopter => self.builder.factory.config.vehicles.helicopter.default_color,
                VehicleType::F16 => self.builder.factory.config.vehicles.f16.default_color,
                VehicleType::Car => self.builder.factory.config.vehicles.basic_car.default_color,
            }
        });
        
        // Create the vehicle entity
        let entity = self.builder.factory.spawn_vehicle(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            self.vehicle_type,
            position,
            color,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// Realistic vehicle builder with fluent API
pub struct RealisticVehicleBuilder<'a> {
    builder: EntityBuilder<'a>,
    vehicle_type: RealisticVehicleType,
    position: Option<Vec3>,
    rotation: Option<Quat>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> RealisticVehicleBuilder<'a> {
    fn new(builder: EntityBuilder<'a>, vehicle_type: RealisticVehicleType) -> Self {
        Self {
            builder,
            vehicle_type,
            position: None,
            rotation: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set vehicle position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Set vehicle rotation
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = Some(rotation);
        self
    }
    
    /// Add custom component to vehicle
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the realistic vehicle entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(Vec3::ZERO);
        let rotation = self.rotation.unwrap_or(Quat::IDENTITY);
        
        // Create the realistic vehicle entity
        let entity = self.builder.factory.spawn_realistic_vehicle(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            self.vehicle_type,
            position,
            rotation,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// NPC builder with fluent API
pub struct NPCBuilder<'a> {
    builder: EntityBuilder<'a>,
    position: Option<Vec3>,
    height: Option<f32>,
    build: Option<f32>,
    appearance: Option<NPCAppearance>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> NPCBuilder<'a> {
    fn new(builder: EntityBuilder<'a>) -> Self {
        Self {
            builder,
            position: None,
            height: None,
            build: None,
            appearance: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set NPC position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Set NPC height
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
    
    /// Set NPC build (body width scaling)
    pub fn with_build(mut self, build: f32) -> Self {
        self.build = Some(build);
        self
    }
    
    /// Set NPC appearance
    pub fn with_appearance(mut self, appearance: NPCAppearance) -> Self {
        self.appearance = Some(appearance);
        self
    }
    
    /// Add custom component to NPC
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the NPC entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(Vec3::ZERO);
        let height = self.height.unwrap_or(self.builder.factory.config.npc.default_height);
        let build = self.build.unwrap_or(self.builder.factory.config.npc.default_build);
        let appearance = self.appearance.unwrap_or_else(|| NPCAppearance {
            height,
            build,
            skin_tone: Color::srgb(0.8, 0.7, 0.6),
            hair_color: Color::srgb(0.3, 0.2, 0.1),
            shirt_color: Color::srgb(0.3, 0.3, 0.7),
            pants_color: Color::srgb(0.2, 0.2, 0.4),
            gender: NPCGender::Male,
        });
        
        // Create the NPC entity
        let entity = self.builder.factory.spawn_npc(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            position,
            appearance,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// Building builder with fluent API
pub struct BuildingBuilder<'a> {
    builder: EntityBuilder<'a>,
    building_type: BuildingType,
    position: Option<Vec3>,
    size: Option<Vec3>,
    color: Option<Color>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> BuildingBuilder<'a> {
    fn new(builder: EntityBuilder<'a>, building_type: BuildingType) -> Self {
        Self {
            builder,
            building_type,
            position: None,
            size: None,
            color: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set building position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Set building size
    pub fn with_size(mut self, size: Vec3) -> Self {
        self.size = Some(size);
        self
    }
    
    /// Set building color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    
    /// Add custom component to building
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the building entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(Vec3::ZERO);
        let size = self.size.unwrap_or(Vec3::new(10.0, 15.0, 10.0));
        let color = self.color.unwrap_or(match self.building_type {
            BuildingType::Residential => Color::srgb(0.8, 0.7, 0.6),
            BuildingType::Commercial => Color::srgb(0.6, 0.7, 0.8),
            BuildingType::Industrial => Color::srgb(0.5, 0.5, 0.6),
            BuildingType::Skyscraper => Color::srgb(0.7, 0.7, 0.8),
            BuildingType::Generic => Color::srgb(0.6, 0.6, 0.6),
        });
        
        // Create the building entity
        let entity = self.builder.factory.spawn_building(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            position,
            size,
            self.building_type,
            color,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// Terrain builder with fluent API
pub struct TerrainBuilder<'a> {
    builder: EntityBuilder<'a>,
    position: Option<Vec3>,
    size: Option<Vec2>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> TerrainBuilder<'a> {
    fn new(builder: EntityBuilder<'a>) -> Self {
        Self {
            builder,
            position: None,
            size: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set terrain position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Set terrain size
    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }
    
    /// Add custom component to terrain
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the terrain entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(Vec3::ZERO);
        let size = self.size.unwrap_or(Vec2::new(1000.0, 1000.0));
        
        // Create the terrain entity
        let entity = self.builder.factory.spawn_terrain(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            position,
            size,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// Water builder with fluent API
pub struct WaterBuilder<'a> {
    builder: EntityBuilder<'a>,
    position: Option<Vec3>,
    size: Option<Vec2>,
    depth: Option<f32>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> WaterBuilder<'a> {
    fn new(builder: EntityBuilder<'a>) -> Self {
        Self {
            builder,
            position: None,
            size: None,
            depth: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set water position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Set water size
    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }
    
    /// Set water depth
    pub fn with_depth(mut self, depth: f32) -> Self {
        self.depth = Some(depth);
        self
    }
    
    /// Add custom component to water
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the water entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(self.builder.factory.config.world.lake_position);
        let size = self.size.unwrap_or(Vec2::splat(self.builder.factory.config.world.lake_size));
        let depth = self.depth.unwrap_or(self.builder.factory.config.world.lake_depth);
        
        // Create the water entity
        let entity = self.builder.factory.spawn_water_body(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            position,
            size,
            depth,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// Tree builder with fluent API
pub struct TreeBuilder<'a> {
    builder: EntityBuilder<'a>,
    position: Option<Vec3>,
    height: Option<f32>,
    trunk_radius: Option<f32>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> TreeBuilder<'a> {
    fn new(builder: EntityBuilder<'a>) -> Self {
        Self {
            builder,
            position: None,
            height: None,
            trunk_radius: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set tree position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Set tree height
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
    
    /// Set trunk radius
    pub fn with_trunk_radius(mut self, radius: f32) -> Self {
        self.trunk_radius = Some(radius);
        self
    }
    
    /// Add custom component to tree
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the tree entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(Vec3::ZERO);
        let height = self.height.unwrap_or(8.0);
        let trunk_radius = self.trunk_radius.unwrap_or(0.3);
        
        // Create the tree entity
        let entity = self.builder.factory.spawn_tree(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            position,
            height,
            trunk_radius,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// Particle effect builder with fluent API
pub struct ParticleBuilder<'a> {
    builder: EntityBuilder<'a>,
    effect_type: ParticleEffectType,
    position: Option<Vec3>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> ParticleBuilder<'a> {
    fn new(builder: EntityBuilder<'a>, effect_type: ParticleEffectType) -> Self {
        Self {
            builder,
            effect_type,
            position: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set particle effect position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Add custom component to particle effect
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the particle effect entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(Vec3::ZERO);
        
        // Create the particle effect entity
        let entity = self.builder.factory.spawn_particle_effect(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            position,
            self.effect_type,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// Road builder with fluent API
pub struct RoadBuilder<'a> {
    builder: EntityBuilder<'a>,
    position: Option<Vec3>,
    size: Option<Vec3>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
}

impl<'a> RoadBuilder<'a> {
    fn new(builder: EntityBuilder<'a>) -> Self {
        Self {
            builder,
            position: None,
            size: None,
            custom_components: Vec::new(),
        }
    }
    
    /// Set road position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Set road size
    pub fn with_size(mut self, size: Vec3) -> Self {
        self.size = Some(size);
        self
    }
    
    /// Add custom component to road
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
        self
    }
    
    /// Spawn the road entity
    pub fn spawn(self) -> Result<Entity, BundleError> {
        let position = self.position.unwrap_or(Vec3::ZERO);
        let size = self.size.unwrap_or(Vec3::new(10.0, 0.1, 100.0));
        
        // Create the road entity
        let entity = self.builder.factory.spawn_road_entity(
            self.builder.commands,
            position,
            size,
            self.builder.meshes,
            self.builder.materials,
        )?;
        
        // Apply custom components
        for component_fn in self.custom_components {
            component_fn(self.builder.commands, entity);
        }
        
        Ok(entity)
    }
}

/// Batch builder for creating multiple entities efficiently
pub struct BatchBuilder<'a> {
    builder: EntityBuilder<'a>,
}

impl<'a> BatchBuilder<'a> {
    /// Creates a new batch builder for efficient multi-entity creation.
    ///
    /// This function initializes a batch builder that can create multiple
    /// entities of the same type efficiently using batch processing techniques.
    /// Batch creation reduces individual entity spawn overhead and improves
    /// performance when creating large numbers of entities.
    ///
    /// # Arguments
    /// * `builder` - The base entity builder with factory and resource access
    ///
    /// # Returns
    /// A new [`BatchBuilder`] instance ready for batch entity creation
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::entity_builder_unified::EntityBuilder;
    /// use gameplay_render::factories::entity_factory_unified::UnifiedEntityFactory;
    /// use game_core::components::VehicleType;
    ///
    /// fn spawn_vehicle_fleet(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    ///     factory: Res<UnifiedEntityFactory>,
    /// ) {
    ///     let vehicle_specs = vec![
    ///         (VehicleType::BasicCar, Vec3::new(0.0, 0.0, 0.0), Color::RED),
    ///         (VehicleType::BasicCar, Vec3::new(10.0, 0.0, 0.0), Color::BLUE),
    ///         (VehicleType::BasicCar, Vec3::new(20.0, 0.0, 0.0), Color::GREEN),
    ///     ];
    ///     
    ///     let entities = factory.entity_builder(&mut commands, &mut meshes, &mut materials)
    ///         .batch()
    ///         .spawn_vehicles(vehicle_specs)
    ///         .expect("Failed to spawn vehicle fleet");
    /// }
    /// ```
    pub fn new(builder: EntityBuilder<'a>) -> Self {
        Self { builder }
    }
    
    /// Spawn multiple vehicles in batch
    pub fn spawn_vehicles(
        self,
        vehicle_specs: Vec<(VehicleType, Vec3, Color)>,
    ) -> Result<Vec<Entity>, BundleError> {
        self.builder.factory.spawn_vehicle_batch(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            vehicle_specs,
        )
    }
    
    /// Spawn multiple NPCs in batch
    pub fn spawn_npcs(
        self,
        npc_specs: Vec<(Vec3, NPCAppearance)>,
    ) -> Result<Vec<Entity>, BundleError> {
        self.builder.factory.spawn_npc_batch(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            npc_specs,
        )
    }
    
    /// Spawn multiple buildings in batch
    pub fn spawn_buildings(
        self,
        building_specs: Vec<(Vec3, Vec3, BuildingType, Color)>,
    ) -> Result<Vec<Entity>, BundleError> {
        self.builder.factory.spawn_building_batch(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            building_specs,
        )
    }
}

/// Convenience trait for easy factory access
pub trait EntityBuilderExt {
    /// Creates an entity builder instance using this factory as the configuration source.
    ///
    /// This method provides convenient access to the [`EntityBuilder`] fluent API
    /// directly from a [`UnifiedEntityFactory`] instance. It automatically passes
    /// the factory configuration to the builder for consistent entity creation.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's ECS command system
    /// * `meshes` - Mutable reference to mesh asset storage
    /// * `materials` - Mutable reference to material asset storage
    ///
    /// # Returns
    /// A new [`EntityBuilder`] instance configured with this factory's settings
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::entity_builder_unified::EntityBuilderExt;
    /// use gameplay_render::factories::entity_factory_unified::UnifiedEntityFactory;
    ///
    /// fn spawn_car(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    ///     factory: Res<UnifiedEntityFactory>,
    /// ) {
    ///     let car_entity = factory.entity_builder(&mut commands, &mut meshes, &mut materials)
    ///         .vehicle(VehicleType::BasicCar)
    ///         .at_position(Vec3::new(100.0, 0.0, 200.0))
    ///         .with_color(Color::RED)
    ///         .spawn()
    ///         .expect("Failed to spawn car");
    /// }
    /// ```
    fn entity_builder<'a>(
        &'a self,
        commands: &'a mut Commands<'a, 'a>,
        meshes: &'a mut ResMut<'a, Assets<Mesh>>,
        materials: &'a mut ResMut<'a, Assets<StandardMaterial>>,
    ) -> EntityBuilder<'a>;
}

impl EntityBuilderExt for UnifiedEntityFactory {
    fn entity_builder<'a>(
        &'a self,
        commands: &'a mut Commands<'a, 'a>,
        meshes: &'a mut ResMut<'a, Assets<Mesh>>,
        materials: &'a mut ResMut<'a, Assets<StandardMaterial>>,
    ) -> EntityBuilder<'a> {
        EntityBuilder::new(self, commands, meshes, materials)
    }
}
