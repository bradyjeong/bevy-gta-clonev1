use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use crate::systems::distance_cache::MovementTracker;
use crate::systems::world::unified_world::UnifiedChunkEntity;
use crate::systems::world::unified_distance_culling::UnifiedCullable;

/// Type alias for old NPCBehavior to maintain compatibility
pub type NPCBehavior = NPCBehaviorComponent;
/// Particle effect types for the unified system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleEffectType {
    Exhaust,
    Smoke,
    Fire,
    Water,
    Dust,
    Explosion,
    Spark,
}
/// CRITICAL: Generic Bundle System - Trait-based bundle creation with type safety
/// Eliminates 200+ duplicate bundle patterns with compile-time validation
/// Core trait for bundle specifications
pub trait BundleSpec: Send + Sync + 'static {
    type Bundle: Bundle;
    
    /// Create bundle with configuration validation
    fn create_bundle(self, config: &GameConfig) -> Self::Bundle;
    /// Validate specification parameters
    fn validate(&self, config: &GameConfig) -> Result<(), BundleError>;
/// Bundle creation errors with detailed context
#[derive(Debug)]
pub enum BundleError {
    PositionOutOfBounds { position: Vec3, max_coord: f32 },
    InvalidSize { size: Vec3, min_size: f32, max_size: f32 },
    InvalidMass { mass: f32, min_mass: f32, max_mass: f32 },
    InvalidVelocity { velocity: f32, max_velocity: f32 },
    InvalidEntityType { entity_type: String },
impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::PositionOutOfBounds { position, max_coord } => {
                write!(f, "Position {:?} is out of bounds (max: {})", position, max_coord)
            }
            BundleError::InvalidSize { size, min_size, max_size } => {
                write!(f, "Size {:?} is invalid (min: {}, max: {})", size, min_size, max_size)
            BundleError::InvalidMass { mass, min_mass, max_mass } => {
                write!(f, "Mass {} is invalid (min: {}, max: {})", mass, min_mass, max_mass)
            BundleError::InvalidVelocity { velocity, max_velocity } => {
                write!(f, "Velocity {} exceeds maximum (max: {})", velocity, max_velocity)
            BundleError::InvalidEntityType { entity_type } => {
                write!(f, "Invalid entity type: {}", entity_type)
        }
    }
impl std::error::Error for BundleError {}
/// Vehicle Bundle Specification
#[derive(Debug, Clone)]
pub struct VehicleBundleSpec {
    pub vehicle_type: VehicleType,
    pub position: Vec3,
    pub color: Color,
    pub max_speed_override: Option<f32>,
    pub mass_override: Option<f32>,
    pub include_physics: bool,
    pub include_collision: bool,
    pub include_visibility: bool,
impl BundleSpec for VehicleBundleSpec {
    type Bundle = VehicleBundle;
    fn create_bundle(self, config: &GameConfig) -> Self::Bundle {
        // Get vehicle type configuration
        let vehicle_config = match self.vehicle_type {
            VehicleType::BasicCar => &config.vehicles.basic_car,
            VehicleType::SuperCar => &config.vehicles.super_car,
            VehicleType::Helicopter => &config.vehicles.helicopter,
            VehicleType::F16 => &config.vehicles.f16,
        };
        
        // Apply overrides with validation
        let max_speed = self.max_speed_override
            .unwrap_or(vehicle_config.max_speed)
            .clamp(10.0, config.physics.max_velocity);
            
        let mass = self.mass_override
            .unwrap_or(vehicle_config.mass)
            .clamp(config.physics.min_mass, config.physics.max_mass);
        // Create validated transform
        let transform = Transform::from_translation(self.position.clamp(
            Vec3::splat(config.physics.min_world_coord),
            Vec3::splat(config.physics.max_world_coord)
        ));
        // Build bundle with configuration
        VehicleBundle {
            vehicle_type: self.vehicle_type,
            vehicle_state: VehicleState {
                color: self.color,
                max_speed,
                acceleration: vehicle_config.acceleration,
                damage: 0.0,
                fuel: 100.0,
                current_lod: VehicleLOD::Full,
                last_lod_check: 0.0,
                vehicle_type: self.vehicle_type,
            },
            transform,
            visibility: Visibility::Visible,
            rigid_body: if self.include_physics { RigidBody::Dynamic } else { RigidBody::Fixed },
            collider: if self.include_collision {
                Collider::cuboid(
                    vehicle_config.collider_size.x / 2.0,
                    vehicle_config.collider_size.y / 2.0,
                    vehicle_config.collider_size.z / 2.0,
                )
            } else {
                Collider::ball(0.1) // Minimal collider
            collision_groups: CollisionGroups::new(config.physics.vehicle_group(), Group::ALL),
            additional_mass: AdditionalMassProperties::Mass(mass),
            velocity: Velocity::zero(),
            damping: Damping { 
                linear_damping: vehicle_config.linear_damping, 
                angular_damping: vehicle_config.angular_damping 
            cullable: if self.include_visibility {
                UnifiedCullable::vehicle()
    fn validate(&self, config: &GameConfig) -> Result<(), BundleError> {
        // Validate position bounds
        if self.position.x.abs() > config.physics.max_world_coord ||
           self.position.z.abs() > config.physics.max_world_coord {
            return Err(BundleError::PositionOutOfBounds {
                position: self.position,
                max_coord: config.physics.max_world_coord,
            });
        // Validate overrides if present
        if let Some(max_speed) = self.max_speed_override {
            if max_speed <= 0.0 || max_speed > config.physics.max_velocity {
                return Err(BundleError::InvalidVelocity {
                    velocity: max_speed,
                    max_velocity: config.physics.max_velocity,
                });
        if let Some(mass) = self.mass_override {
            if mass < config.physics.min_mass || mass > config.physics.max_mass {
                return Err(BundleError::InvalidMass {
                    mass,
                    min_mass: config.physics.min_mass,
                    max_mass: config.physics.max_mass,
        Ok(())
/// NPC Bundle Specification
pub struct NPCBundleSpec {
    pub height: f32,
    pub build: f32,
    pub appearance: NPCAppearance,
    pub behavior: Option<NPCBehavior>,
    pub include_ai: bool,
impl BundleSpec for NPCBundleSpec {
    type Bundle = NPCBundle;
        // Validate and clamp parameters
        let _height = self.height.clamp(0.5, 3.0);
        let build = self.build.clamp(0.3, 2.0);
        NPCBundle {
            npc_marker: NPCState {
                npc_type: NPCType::Civilian,
                appearance: self.appearance,
                behavior: NPCBehaviorType::Wandering,
                target_position: self.position,
                speed: config.npc.walk_speed,
                current_lod: NPCLOD::Full,
            npc_behavior: self.behavior.unwrap_or(NPCBehavior {
                last_update: 0.0,
                update_interval: config.npc.update_intervals.close_interval,
            }),
            npc_appearance: self.appearance,
            movement_controller: MovementController {
                current_speed: 0.0,
                max_speed: config.npc.walk_speed,
                stamina: 100.0,
            visibility: Visibility::Inherited,
            collider: Collider::capsule_y(config.npc.capsule_height, config.npc.capsule_radius),
            collision_groups: CollisionGroups::new(config.physics.character_group(), Group::ALL),
            additional_mass: AdditionalMassProperties::Mass(70.0 * build),
            cullable: UnifiedCullable::npc(),
            movement_tracker: MovementTracker::new(self.position, 8.0), // Track NPC movement with 8m threshold
        // Validate physical parameters
        if self.height < 0.5 || self.height > 3.0 {
            return Err(BundleError::InvalidSize {
                size: Vec3::new(self.build, self.height, self.build),
                min_size: 0.5,
                max_size: 3.0,
        if self.build < 0.3 || self.build > 2.0 {
                min_size: 0.3,
                max_size: 2.0,
/// Building Bundle Specification
pub struct BuildingBundleSpec {
    pub size: Vec3,
    pub building_type: BuildingType,
    pub lod_level: u8,
impl BundleSpec for BuildingBundleSpec {
    type Bundle = BuildingBundle;
        // Validate and clamp size
        let size = Vec3::new(
            self.size.x.clamp(1.0, config.physics.max_collider_size),
            self.size.y.clamp(1.0, config.physics.max_collider_size),
            self.size.z.clamp(1.0, config.physics.max_collider_size),
        );
        BuildingBundle {
            building_marker: Building {
                building_type: self.building_type,
                height: size.y,
                scale: size,
            rigid_body: RigidBody::Fixed,
                Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
                Collider::ball(0.1) // Minimal collider for LOD
            collision_groups: CollisionGroups::new(config.physics.static_group(), Group::ALL),
            cullable: UnifiedCullable::building(),
        // Validate size bounds
        if self.size.x < config.physics.min_collider_size || self.size.x > config.physics.max_collider_size ||
           self.size.y < config.physics.min_collider_size || self.size.y > config.physics.max_collider_size ||
           self.size.z < config.physics.min_collider_size || self.size.z > config.physics.max_collider_size {
                size: self.size,
                min_size: config.physics.min_collider_size,
                max_size: config.physics.max_collider_size,
/// Physics Bundle Specification - For standalone physics objects
pub struct PhysicsBundleSpec {
    pub collider_shape: ColliderShape,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub collision_group: Group,
    pub is_dynamic: bool,
pub enum ColliderShape {
    Box(Vec3),
    Sphere(f32),
    Capsule { radius: f32, height: f32 },
    Cylinder { radius: f32, height: f32 },
impl BundleSpec for PhysicsBundleSpec {
    type Bundle = PhysicsBundle;
        // Validate and clamp mass
        let mass = self.mass.clamp(config.physics.min_mass, config.physics.max_mass);
        // Validate and clamp friction/restitution
        let friction = self.friction.clamp(0.0, 2.0);
        let restitution = self.restitution.clamp(0.0, 1.0);
        // Create collider based on shape
        let collider = match self.collider_shape {
            ColliderShape::Box(size) => {
                let clamped_size = Vec3::new(
                    size.x.clamp(config.physics.min_collider_size, config.physics.max_collider_size),
                    size.y.clamp(config.physics.min_collider_size, config.physics.max_collider_size),
                    size.z.clamp(config.physics.min_collider_size, config.physics.max_collider_size),
                );
                Collider::cuboid(clamped_size.x / 2.0, clamped_size.y / 2.0, clamped_size.z / 2.0)
            ColliderShape::Sphere(radius) => {
                let clamped_radius = radius.clamp(config.physics.min_collider_size, config.physics.max_collider_size);
                Collider::ball(clamped_radius)
            ColliderShape::Capsule { radius, height } => {
                let clamped_height = height.clamp(config.physics.min_collider_size, config.physics.max_collider_size);
                Collider::capsule_y(clamped_height, clamped_radius)
            ColliderShape::Cylinder { radius, height } => {
                Collider::cylinder(clamped_height, clamped_radius)
        PhysicsBundle {
            rigid_body: if self.is_dynamic { RigidBody::Dynamic } else { RigidBody::Fixed },
            collider,
            collision_groups: CollisionGroups::new(self.collision_group, Group::ALL),
                linear_damping: config.physics.linear_damping, 
                angular_damping: config.physics.angular_damping 
            friction: Friction::coefficient(friction),
            restitution: Restitution::coefficient(restitution),
        // Validate mass
        if self.mass < config.physics.min_mass || self.mass > config.physics.max_mass {
            return Err(BundleError::InvalidMass {
                mass: self.mass,
                min_mass: config.physics.min_mass,
                max_mass: config.physics.max_mass,
        // Validate collider shape dimensions
        match &self.collider_shape {
                if size.x < config.physics.min_collider_size || size.x > config.physics.max_collider_size ||
                   size.y < config.physics.min_collider_size || size.y > config.physics.max_collider_size ||
                   size.z < config.physics.min_collider_size || size.z > config.physics.max_collider_size {
                    return Err(BundleError::InvalidSize {
                        size: *size,
                        min_size: config.physics.min_collider_size,
                        max_size: config.physics.max_collider_size,
                    });
                }
                if *radius < config.physics.min_collider_size || *radius > config.physics.max_collider_size {
                        size: Vec3::splat(*radius),
            ColliderShape::Capsule { radius, height } | ColliderShape::Cylinder { radius, height } => {
                if *radius < config.physics.min_collider_size || *radius > config.physics.max_collider_size ||
                   *height < config.physics.min_collider_size || *height > config.physics.max_collider_size {
                        size: Vec3::new(*radius, *height, *radius),
/// Generic Bundle Factory - Unified creation interface
pub struct GenericBundleFactory;
impl GenericBundleFactory {
    /// Create bundle with validation and configuration
    pub fn create<T: BundleSpec>(
        spec: T,
        config: &GameConfig,
    ) -> Result<T::Bundle, BundleError> {
        // Validate specification
        spec.validate(config)?;
        // Create bundle with validated configuration
        Ok(spec.create_bundle(config))
    /// Create multiple bundles with batch validation
    pub fn create_batch<T: BundleSpec>(
        specs: Vec<T>,
    ) -> Result<Vec<T::Bundle>, BundleError> {
        // Validate all specs first
        for spec in &specs {
            spec.validate(config)?;
        // Create all bundles
        Ok(specs.into_iter().map(|spec| spec.create_bundle(config)).collect())
/// Convenience builder functions for common bundle types
    /// Create vehicle bundle with validation
    pub fn vehicle(
        vehicle_type: VehicleType,
        position: Vec3,
        color: Color,
    ) -> Result<VehicleBundle, BundleError> {
        let spec = VehicleBundleSpec {
            vehicle_type,
            position,
            color,
            max_speed_override: None,
            mass_override: None,
            include_physics: true,
            include_collision: true,
            include_visibility: true,
        Self::create(spec, config)
    /// Create NPC bundle with validation
    pub fn npc(
        height: f32,
        build: f32,
        appearance: NPCAppearance,
    ) -> Result<NPCBundle, BundleError> {
        let spec = NPCBundleSpec {
            height,
            build,
            appearance,
            behavior: None,
            include_ai: true,
    /// Create building bundle with validation
    pub fn building(
        size: Vec3,
        building_type: BuildingType,
    ) -> Result<BuildingBundle, BundleError> {
        let spec = BuildingBundleSpec {
            size,
            building_type,
            lod_level: 0,
    /// Create physics object bundle with validation
    pub fn physics_object(
        collider_shape: ColliderShape,
        mass: f32,
        collision_group: Group,
        is_dynamic: bool,
    ) -> Result<PhysicsBundle, BundleError> {
        let spec = PhysicsBundleSpec {
            collider_shape,
            mass,
            friction: config.physics.ground_friction,
            restitution: 0.3,
            collision_group,
            is_dynamic,
/// Bundle utility functions for common entity patterns
    /// Create dynamic content bundle for world entities
    pub fn dynamic_content(
        content_type: ContentType,
        _max_distance: f32,
    ) -> DynamicContentBundle {
        DynamicContentBundle {
            dynamic_content: DynamicContent { content_type },
            transform: Transform::from_translation(position),
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
            cullable: UnifiedCullable::vegetation(),
    /// Create dynamic physics bundle for moving objects
    pub fn dynamic_physics(
        collider: Collider,
        collision_groups: CollisionGroups,
    ) -> DynamicPhysicsBundle {
        DynamicPhysicsBundle {
            rigid_body: RigidBody::Dynamic,
            collision_groups,
    /// Create vehicle bundle for cars
    pub fn dynamic_vehicle(
        damping: Damping,
    ) -> DynamicVehicleBundle {
        DynamicVehicleBundle {
            dynamic_content: DynamicContent { content_type: ContentType::Vehicle },
            car: Car,
            transform: Transform::from_xyz(position.x, position.y, position.z),
            collider: Collider::cuboid(1.0, 0.5, 2.0),
            damping,
            locked_axes: LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            cullable: UnifiedCullable::vehicle(),
    /// Create vegetation bundle for trees
    pub fn vegetation(
    ) -> VegetationBundle {
        VegetationBundle {
            dynamic_content: DynamicContent { content_type: ContentType::Tree },
    /// Create static physics bundle for immobile objects
    pub fn static_physics(
    ) -> StaticPhysicsBundle {
        StaticPhysicsBundle {
    /// Create unified chunk bundle
    pub fn unified_chunk(
        chunk_coord: (i32, i32),
        layer: crate::systems::world::unified_world::ContentLayer,
    ) -> UnifiedChunkBundle {
        UnifiedChunkBundle {
            chunk_entity: UnifiedChunkEntity { 
                coord: crate::systems::world::unified_world::ChunkCoord::new(chunk_coord.0, chunk_coord.1), 
                layer: layer.to_layer_id()
