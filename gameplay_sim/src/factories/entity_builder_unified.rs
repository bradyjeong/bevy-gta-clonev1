use bevy::prelude::*;
use game_core::prelude::*;
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
    /// Start building a realistic vehicle
    pub fn realistic_vehicle(self, vehicle_type: RealisticVehicleType) -> RealisticVehicleBuilder<'a> {
        RealisticVehicleBuilder::new(self, vehicle_type)
    /// Start building an NPC
    pub fn npc(self) -> NPCBuilder<'a> {
        NPCBuilder::new(self)
    /// Start building a building
    pub fn building(self, building_type: BuildingType) -> BuildingBuilder<'a> {
        BuildingBuilder::new(self, building_type)
    /// Start building terrain
    pub fn terrain(self) -> TerrainBuilder<'a> {
        TerrainBuilder::new(self)
    /// Start building a water body
    pub fn water(self) -> WaterBuilder<'a> {
        WaterBuilder::new(self)
    /// Start building a tree
    pub fn tree(self) -> TreeBuilder<'a> {
        TreeBuilder::new(self)
    /// Start building a particle effect
    pub fn particle_effect(self, effect_type: ParticleEffectType) -> ParticleBuilder<'a> {
        ParticleBuilder::new(self, effect_type)
    /// Start building a road
    pub fn road(self) -> RoadBuilder<'a> {
        RoadBuilder::new(self)
    /// Create batch builder for efficient bulk operations
    pub fn batch(self) -> BatchBuilder<'a> {
        BatchBuilder::new(self)
/// Vehicle builder with fluent API
pub struct VehicleBuilder<'a> {
    builder: EntityBuilder<'a>,
    vehicle_type: VehicleType,
    position: Option<Vec3>,
    color: Option<Color>,
    custom_components: Vec<Box<dyn FnOnce(&mut Commands, Entity)>>,
impl<'a> VehicleBuilder<'a> {
    fn new(builder: EntityBuilder<'a>, vehicle_type: VehicleType) -> Self {
            builder,
            vehicle_type,
            position: None,
            color: None,
            custom_components: Vec::new(),
    /// Set vehicle position
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    /// Set vehicle color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
    /// Add custom component to vehicle
    pub fn with_component<T: Component + Clone>(mut self, component: T) -> Self {
        self.custom_components.push(Box::new(move |commands, entity| {
            commands.entity(entity).insert(component.clone());
        }));
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
        Ok(entity)
/// Realistic vehicle builder with fluent API
pub struct RealisticVehicleBuilder<'a> {
    vehicle_type: RealisticVehicleType,
    rotation: Option<Quat>,
impl<'a> RealisticVehicleBuilder<'a> {
    fn new(builder: EntityBuilder<'a>, vehicle_type: RealisticVehicleType) -> Self {
            rotation: None,
    /// Set vehicle rotation
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = Some(rotation);
    /// Spawn the realistic vehicle entity
        let rotation = self.rotation.unwrap_or(Quat::IDENTITY);
        // Create the realistic vehicle entity
        let entity = self.builder.factory.spawn_realistic_vehicle(
            rotation,
/// NPC builder with fluent API
pub struct NPCBuilder<'a> {
    height: Option<f32>,
    build: Option<f32>,
    appearance: Option<NPCAppearance>,
impl<'a> NPCBuilder<'a> {
    fn new(builder: EntityBuilder<'a>) -> Self {
            height: None,
            build: None,
            appearance: None,
    /// Set NPC position
    /// Set NPC height
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
    /// Set NPC build (body width scaling)
    pub fn with_build(mut self, build: f32) -> Self {
        self.build = Some(build);
    /// Set NPC appearance
    pub fn with_appearance(mut self, appearance: NPCAppearance) -> Self {
        self.appearance = Some(appearance);
    /// Add custom component to NPC
    /// Spawn the NPC entity
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
        // Create the NPC entity
        let entity = self.builder.factory.spawn_npc(
            appearance,
/// Building builder with fluent API
pub struct BuildingBuilder<'a> {
    building_type: BuildingType,
    size: Option<Vec3>,
impl<'a> BuildingBuilder<'a> {
    fn new(builder: EntityBuilder<'a>, building_type: BuildingType) -> Self {
            building_type,
            size: None,
    /// Set building position
    /// Set building size
    pub fn with_size(mut self, size: Vec3) -> Self {
        self.size = Some(size);
    /// Set building color
    /// Add custom component to building
    /// Spawn the building entity
        let size = self.size.unwrap_or(Vec3::new(10.0, 15.0, 10.0));
        let color = self.color.unwrap_or(match self.building_type {
            BuildingType::Residential => Color::srgb(0.8, 0.7, 0.6),
            BuildingType::Commercial => Color::srgb(0.6, 0.7, 0.8),
            BuildingType::Industrial => Color::srgb(0.5, 0.5, 0.6),
            BuildingType::Skyscraper => Color::srgb(0.7, 0.7, 0.8),
            BuildingType::Generic => Color::srgb(0.6, 0.6, 0.6),
        // Create the building entity
        let entity = self.builder.factory.spawn_building(
            size,
            self.building_type,
/// Terrain builder with fluent API
pub struct TerrainBuilder<'a> {
    size: Option<Vec2>,
impl<'a> TerrainBuilder<'a> {
    /// Set terrain position
    /// Set terrain size
    pub fn with_size(mut self, size: Vec2) -> Self {
    /// Add custom component to terrain
    /// Spawn the terrain entity
        let size = self.size.unwrap_or(Vec2::new(1000.0, 1000.0));
        // Create the terrain entity
        let entity = self.builder.factory.spawn_terrain(
/// Water builder with fluent API
pub struct WaterBuilder<'a> {
    depth: Option<f32>,
impl<'a> WaterBuilder<'a> {
            depth: None,
    /// Set water position
    /// Set water size
    /// Set water depth
    pub fn with_depth(mut self, depth: f32) -> Self {
        self.depth = Some(depth);
    /// Add custom component to water
    /// Spawn the water entity
        let position = self.position.unwrap_or(self.builder.factory.config.world.lake_position);
        let size = self.size.unwrap_or(Vec2::splat(self.builder.factory.config.world.lake_size));
        let depth = self.depth.unwrap_or(self.builder.factory.config.world.lake_depth);
        // Create the water entity
        let entity = self.builder.factory.spawn_water_body(
            depth,
/// Tree builder with fluent API
pub struct TreeBuilder<'a> {
    trunk_radius: Option<f32>,
impl<'a> TreeBuilder<'a> {
            trunk_radius: None,
    /// Set tree position
    /// Set tree height
    /// Set trunk radius
    pub fn with_trunk_radius(mut self, radius: f32) -> Self {
        self.trunk_radius = Some(radius);
    /// Add custom component to tree
    /// Spawn the tree entity
        let height = self.height.unwrap_or(8.0);
        let trunk_radius = self.trunk_radius.unwrap_or(0.3);
        // Create the tree entity
        let entity = self.builder.factory.spawn_tree(
            trunk_radius,
/// Particle effect builder with fluent API
pub struct ParticleBuilder<'a> {
    effect_type: ParticleEffectType,
impl<'a> ParticleBuilder<'a> {
    fn new(builder: EntityBuilder<'a>, effect_type: ParticleEffectType) -> Self {
            effect_type,
    /// Set particle effect position
    /// Add custom component to particle effect
    /// Spawn the particle effect entity
        // Create the particle effect entity
        let entity = self.builder.factory.spawn_particle_effect(
            self.effect_type,
/// Road builder with fluent API
pub struct RoadBuilder<'a> {
impl<'a> RoadBuilder<'a> {
    /// Set road position
    /// Set road size
    /// Add custom component to road
    /// Spawn the road entity
        let size = self.size.unwrap_or(Vec3::new(10.0, 0.1, 100.0));
        // Create the road entity
        let entity = self.builder.factory.spawn_road_entity(
/// Batch builder for creating multiple entities efficiently
pub struct BatchBuilder<'a> {
impl<'a> BatchBuilder<'a> {
    pub fn new(builder: EntityBuilder<'a>) -> Self {
        Self { builder }
    /// Spawn multiple vehicles in batch
    pub fn spawn_vehicles(
        self,
        vehicle_specs: Vec<(VehicleType, Vec3, Color)>,
    ) -> Result<Vec<Entity>, BundleError> {
        self.builder.factory.spawn_vehicle_batch(
            vehicle_specs,
        )
    /// Spawn multiple NPCs in batch
    pub fn spawn_npcs(
        npc_specs: Vec<(Vec3, NPCAppearance)>,
        self.builder.factory.spawn_npc_batch(
            npc_specs,
    /// Spawn multiple buildings in batch
    pub fn spawn_buildings(
        building_specs: Vec<(Vec3, Vec3, BuildingType, Color)>,
        self.builder.factory.spawn_building_batch(
            building_specs,
/// Convenience trait for easy factory access
pub trait EntityBuilderExt {
    fn entity_builder<'a>(
        &'a self,
    ) -> EntityBuilder<'a>;
impl EntityBuilderExt for UnifiedEntityFactory {
    ) -> EntityBuilder<'a> {
        EntityBuilder::new(self, commands, meshes, materials)
