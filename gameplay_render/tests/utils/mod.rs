//! Test utilities for gameplay_render tests
//! Provides helpers for creating test scenarios, entities, and configurations

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_render::prelude::*;
use std::time::Duration;
/// Test configuration for rendering tests
pub struct RenderTestConfig {
    pub fixed_timestep: f32,
    pub max_test_duration: f32,
    pub epsilon: f32,
    pub frame_budget_ms: f32,
}
impl Default for RenderTestConfig {
    fn default() -> Self {
        Self {
            fixed_timestep: 1.0 / 60.0,    // 60 FPS
            max_test_duration: 5.0,        // 5 seconds
            epsilon: 1e-6,                 // Float comparison tolerance
            frame_budget_ms: 16.67,        // 60 FPS frame budget
        }
    }
/// Create a headless test app for rendering tests
pub fn create_render_test_app() -> App {
    let mut app = App::new();
    
    // Use minimal plugins for headless testing
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        HierarchyPlugin,
        AssetPlugin::default(),
        RapierPhysicsPlugin::<NoUserData>::default(),
    ));
    // Add game configuration
    app.insert_resource(create_test_game_config());
    // Add rendering plugin
    app.add_plugins(RenderPlugin);
    // Set fixed timestep for deterministic testing
    app.insert_resource(FixedTime::new_from_secs(1.0 / 60.0));
    // Initialize performance counters
    app.init_resource::<game_core::config::performance_config::PerformanceCounters>();
    app
/// Create test game configuration with oracle-specified distances
pub fn create_test_game_config() -> GameConfig {
    let mut config = GameConfig::default();
    // Set oracle-specified LOD distances
    config.world.lod_distances = [150.0, 300.0, 500.0]; // Vehicle distances
    config.world.streaming_radius = 300.0; // Building distance
    config.npc.update_intervals.close_distance = 100.0; // NPC close distance
    config.npc.update_intervals.far_distance = 200.0; // NPC far distance
    config
/// Create a test camera entity
pub fn create_test_camera(app: &mut App) -> Entity {
    let mut world = app.world_mut();
    world.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    )).id()
/// Create a test vehicle at a specific distance from camera
pub fn create_test_vehicle_at_distance(app: &mut App, distance: f32) -> Entity {
    let position = Vec3::new(distance, 0.0, 0.0);
        Car::default(),
        ActiveEntity,
        LodLevel::High,
        Transform::from_translation(position),
        Visibility::Visible,
        Cullable::default(),
        RigidBody::Dynamic,
        Collider::cuboid(2.0, 1.0, 4.0),
        Velocity::default(),
/// Create a test NPC at a specific distance from camera
pub fn create_test_npc_at_distance(app: &mut App, distance: f32) -> Entity {
    let position = Vec3::new(0.0, 0.0, distance);
        NPC::default(),
        Collider::capsule_y(0.5, 0.5),
/// Create a test building at a specific distance from camera
pub fn create_test_building_at_distance(app: &mut App, distance: f32) -> Entity {
    let position = Vec3::new(distance * 0.7, 0.0, distance * 0.7);
        Building::default(),
        RigidBody::Fixed,
        Collider::cuboid(5.0, 10.0, 5.0),
/// Create test vegetation at a specific distance from camera
pub fn create_test_vegetation_at_distance(app: &mut App, distance: f32) -> Entity {
        VegetationBatchable::default(),
        VegetationLOD {
            detail_level: VegetationDetailLevel::Full,
            distance_to_player: distance,
        },
/// Run rendering simulation for a fixed number of frames
pub fn run_render_simulation_frames(app: &mut App, frames: u32) {
    for _ in 0..frames {
        app.update();
/// Run rendering simulation for a fixed duration
pub fn run_render_simulation_duration(app: &mut App, duration: f32) {
    let frames = (duration / (1.0 / 60.0)) as u32;
    run_render_simulation_frames(app, frames);
/// Test scene builder for consistent test scenarios
pub struct TestSceneBuilder {
    pub camera_entity: Option<Entity>,
    pub vehicle_entities: Vec<Entity>,
    pub npc_entities: Vec<Entity>,
    pub building_entities: Vec<Entity>,
    pub vegetation_entities: Vec<Entity>,
impl TestSceneBuilder {
    pub fn new() -> Self {
            camera_entity: None,
            vehicle_entities: Vec::new(),
            npc_entities: Vec::new(),
            building_entities: Vec::new(),
            vegetation_entities: Vec::new(),
    pub fn with_camera(mut self, app: &mut App) -> Self {
        self.camera_entity = Some(create_test_camera(app));
        self
    pub fn with_vehicle_at_distance(mut self, app: &mut App, distance: f32) -> Self {
        let entity = create_test_vehicle_at_distance(app, distance);
        self.vehicle_entities.push(entity);
    pub fn with_npc_at_distance(mut self, app: &mut App, distance: f32) -> Self {
        let entity = create_test_npc_at_distance(app, distance);
        self.npc_entities.push(entity);
    pub fn with_building_at_distance(mut self, app: &mut App, distance: f32) -> Self {
        let entity = create_test_building_at_distance(app, distance);
        self.building_entities.push(entity);
    pub fn with_vegetation_at_distance(mut self, app: &mut App, distance: f32) -> Self {
        let entity = create_test_vegetation_at_distance(app, distance);
        self.vegetation_entities.push(entity);
    pub fn build(self) -> TestScene {
        TestScene {
            camera_entity: self.camera_entity,
            vehicle_entities: self.vehicle_entities,
            npc_entities: self.npc_entities,
            building_entities: self.building_entities,
            vegetation_entities: self.vegetation_entities,
/// Test scene with entities at known distances
pub struct TestScene {
/// Performance measurement utilities
pub struct PerformanceMeasurement {
    pub frame_time_ms: f32,
    pub lod_updates: u32,
    pub entities_processed: u32,
    pub culled_entities: u32,
    pub render_operations: u32,
impl PerformanceMeasurement {
    pub fn capture(app: &App) -> Self {
        let counters = app.world().resource::<game_core::config::performance_config::PerformanceCounters>();
        
            frame_time_ms: counters.frame_time_ms,
            lod_updates: counters.lod_updates,
            entities_processed: counters.entity_count,
            culled_entities: counters.culled_entities,
            render_operations: counters.render_operations,
/// Assertions for LOD levels
pub fn assert_lod_level(app: &App, entity: Entity, expected_level: LodLevel) {
    let world = app.world();
    let lod_level = world.get::<LodLevel>(entity)
        .expect("Entity should have LOD component");
    assert_eq!(*lod_level, expected_level, 
        "Entity {:?} should have LOD level {:?}, but has {:?}", 
        entity, expected_level, lod_level);
/// Assertions for vegetation LOD levels
pub fn assert_vegetation_lod_level(app: &App, entity: Entity, expected_level: VegetationDetailLevel) {
    let veg_lod = world.get::<VegetationLOD>(entity)
        .expect("Entity should have VegetationLOD component");
    assert_eq!(veg_lod.detail_level, expected_level,
        "Entity {:?} should have vegetation LOD level {:?}, but has {:?}",
        entity, expected_level, veg_lod.detail_level);
/// Assertions for visibility
pub fn assert_visibility(app: &App, entity: Entity, expected_visibility: Visibility) {
    let visibility = world.get::<Visibility>(entity)
        .expect("Entity should have Visibility component");
    assert_eq!(*visibility, expected_visibility,
        "Entity {:?} should have visibility {:?}, but has {:?}",
        entity, expected_visibility, visibility);
/// Assertions for component presence
pub fn assert_has_component<T: Component>(app: &App, entity: Entity, component_name: &str) {
    assert!(world.get::<T>(entity).is_some(),
        "Entity {:?} should have {} component", entity, component_name);
/// Assertions for component absence
pub fn assert_lacks_component<T: Component>(app: &App, entity: Entity, component_name: &str) {
    assert!(world.get::<T>(entity).is_none(),
        "Entity {:?} should not have {} component", entity, component_name);
/// Distance calculation helper
pub fn calculate_distance_to_camera(app: &App, entity: Entity, camera_entity: Entity) -> f32 {
    let entity_transform = world.get::<Transform>(entity)
        .expect("Entity should have Transform");
    let camera_transform = world.get::<Transform>(camera_entity)
        .expect("Camera should have Transform");
    entity_transform.translation.distance(camera_transform.translation)
/// Float comparison with epsilon
pub fn f32_equals(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
/// Vec3 comparison with epsilon
pub fn vec3_equals(a: Vec3, b: Vec3, epsilon: f32) -> bool {
    (a - b).length() < epsilon
/// Default implementation for TestSceneBuilder
impl Default for TestSceneBuilder {
        Self::new()
