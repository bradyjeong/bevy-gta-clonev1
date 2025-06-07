use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::config::*;
use crate::components::*;
use crate::factories::generic_bundle::*;
use crate::factories::unified_entity_factory::*;

/// CRITICAL: Fluent Builder Pattern - Chainable entity creation API
/// Provides intuitive, type-safe entity construction with validation

/// Main entity builder with fluent API
pub struct EntityBuilder<'a> {
    factory: &'a UnifiedEntityFactory,
    commands: &'a mut Commands<'a, 'a>,
    meshes: &'a mut ResMut<'a, Assets<Mesh>>,
    materials: &'a mut ResMut<'a, Assets<StandardMaterial>>,
}

impl<'a> EntityBuilder<'a> {
    /// Create new entity builder
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
}

/// Vehicle builder with fluent API
pub struct VehicleBuilder<'a> {
    builder: EntityBuilder<'a>,
    vehicle_type: VehicleType,
    position: Option<Vec3>,
    color: Option<Color>,
    max_speed_override: Option<f32>,
    mass_override: Option<f32>,
    custom_components: Vec<Box<dyn Fn(&mut Commands, Entity)>>,
}

impl<'a> VehicleBuilder<'a> {
    fn new(builder: EntityBuilder<'a>, vehicle_type: VehicleType) -> Self {
        Self {
            builder,
            vehicle_type,
            position: None,
            color: None,
            max_speed_override: None,
            mass_override: None,
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
    
    /// Override maximum speed
    pub fn with_max_speed(mut self, max_speed: f32) -> Self {
        self.max_speed_override = Some(max_speed);
        self
    }
    
    /// Override vehicle mass
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass_override = Some(mass);
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
            }
        });
        
        // Validate position
        let validated_position = self.builder.factory.validate_position(position)?;
        
        // Create the vehicle entity
        let entity = self.builder.factory.spawn_vehicle(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            self.vehicle_type,
            validated_position,
            color,
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
    behavior: Option<NPCBehavior>,
    custom_components: Vec<Box<dyn Fn(&mut Commands, Entity)>>,
}

impl<'a> NPCBuilder<'a> {
    fn new(builder: EntityBuilder<'a>) -> Self {
        Self {
            builder,
            position: None,
            height: None,
            build: None,
            appearance: None,
            behavior: None,
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
    
    /// Set NPC behavior
    pub fn with_behavior(mut self, behavior: NPCBehavior) -> Self {
        self.behavior = Some(behavior);
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
        
        // Validate position
        let validated_position = self.builder.factory.validate_position(position)?;
        
        // Create the NPC entity
        let entity = self.builder.factory.spawn_npc(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            validated_position,
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
    custom_components: Vec<Box<dyn Fn(&mut Commands, Entity)>>,
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
        
        // Validate position
        let validated_position = self.builder.factory.validate_position(position)?;
        
        // Create the building entity
        let entity = self.builder.factory.spawn_building(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            validated_position,
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
    custom_components: Vec<Box<dyn Fn(&mut Commands, Entity)>>,
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
        
        // Validate position
        let validated_position = self.builder.factory.validate_position(position)?;
        
        // Create the terrain entity
        let entity = self.builder.factory.spawn_terrain(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            validated_position,
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
    custom_components: Vec<Box<dyn Fn(&mut Commands, Entity)>>,
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
        
        // Validate position
        let validated_position = self.builder.factory.validate_position(position)?;
        
        // Create the water entity
        let entity = self.builder.factory.spawn_water_body(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            validated_position,
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
    custom_components: Vec<Box<dyn Fn(&mut Commands, Entity)>>,
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
        
        // Validate position
        let validated_position = self.builder.factory.validate_position(position)?;
        
        // Create the tree entity
        let entity = self.builder.factory.spawn_tree(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            validated_position,
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
    custom_components: Vec<Box<dyn Fn(&mut Commands, Entity)>>,
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
        
        // Validate position
        let validated_position = self.builder.factory.validate_position(position)?;
        
        // Create the particle effect entity
        let entity = self.builder.factory.spawn_particle_effect(
            self.builder.commands,
            self.builder.meshes,
            self.builder.materials,
            validated_position,
            self.effect_type,
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

impl<'a> EntityBuilder<'a> {
    /// Create batch builder for efficient bulk operations
    pub fn batch(self) -> BatchBuilder<'a> {
        BatchBuilder::new(self)
    }
}
