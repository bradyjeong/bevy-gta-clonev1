// Test utilities for gameplay_sim tests
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_sim::prelude::*;
use std::path::Path;

/// Test configuration for deterministic testing
pub struct TestConfig {
    pub fixed_timestep: f32,
    pub max_test_duration: f32,
    pub epsilon: f32,
}
impl Default for TestConfig {
    fn default() -> Self {
        Self {
            fixed_timestep: 1.0 / 60.0, // 60 FPS
            max_test_duration: 10.0,    // 10 seconds
            epsilon: 1e-6,              // Float comparison tolerance
        }
    }
/// Create a headless test app for simulation testing
pub fn create_test_app() -> App {
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
    app.insert_resource(GameConfig::default());
    // Add simulation plugin
    app.add_plugins(SimulationPlugin);
    // Set fixed timestep for deterministic testing
    app.insert_resource(FixedTime::new_from_secs(1.0 / 60.0));
    app
/// Create a deterministic test scene with known entities
pub fn setup_test_scene(app: &mut App) -> (Entity, Entity, Entity) {
    let mut world = app.world_mut();
    // Create a player entity
    let player = world.spawn((
        Player::default(),
        ActiveEntity,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Velocity::default(),
        RigidBody::Dynamic,
        Collider::capsule_y(0.5, 0.5),
        HumanMovement::default(),
        HumanBehavior::default(),
        HumanAnimation::default(),
    )).id();
    // Create a test vehicle
    let vehicle = world.spawn((
        Car::default(),
        Transform::from_xyz(10.0, 0.0, 0.0),
        Collider::cuboid(2.0, 1.0, 4.0),
        RealisticVehicle::default(),
        VehicleDynamics::default(),
        EnginePhysics::default(),
        VehicleSuspension::default(),
        TirePhysics::default(),
    // Create static ground
    let ground = world.spawn((
        Transform::from_xyz(0.0, -1.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(100.0, 1.0, 100.0),
    (player, vehicle, ground)
/// Create a supercar for physics testing
pub fn create_test_supercar(app: &mut App) -> Entity {
    world.spawn((
        SuperCar::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Collider::cuboid(2.0, 1.0, 4.5),
        AdditionalMassProperties::Mass(1800.0), // Chiron weight
    )).id()
/// Run simulation for a fixed number of steps
pub fn run_simulation_steps(app: &mut App, steps: u32) {
    for _ in 0..steps {
        app.update();
/// Run simulation for a fixed duration
pub fn run_simulation_duration(app: &mut App, duration: f32) {
    let steps = (duration / (1.0 / 60.0)) as u32;
    run_simulation_steps(app, steps);
/// Compare two Vec3 values with epsilon tolerance
pub fn vec3_equals(a: Vec3, b: Vec3, epsilon: f32) -> bool {
    (a - b).length() < epsilon
/// Compare two f32 values with epsilon tolerance
pub fn f32_equals(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
/// Load golden data from CSV file
pub fn load_golden_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut data = Vec::new();
    for result in reader.records() {
        let record = result?;
        let row: Vec<f32> = record.iter()
            .map(|field| field.parse::<f32>())
            .collect::<Result<Vec<_>, _>>()?;
        data.push(row);
    Ok(data)
/// Save trajectory data to CSV
pub fn save_trajectory_csv<P: AsRef<Path>>(
    path: P,
    data: &[(f32, Vec3, Vec3)], // (time, position, velocity)
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = csv::Writer::from_path(path)?;
    // Write header
    writer.write_record(&["time", "pos_x", "pos_y", "pos_z", "vel_x", "vel_y", "vel_z"])?;
    // Write data
    for (time, pos, vel) in data {
        writer.write_record(&[
            time.to_string(),
            pos.x.to_string(),
            pos.y.to_string(),
            pos.z.to_string(),
            vel.x.to_string(),
            vel.y.to_string(),
            vel.z.to_string(),
        ])?;
    writer.flush()?;
    Ok(())
/// Capture vehicle trajectory during simulation
pub fn capture_vehicle_trajectory(
    app: &mut App,
    vehicle_entity: Entity,
    duration: f32,
) -> Vec<(f32, Vec3, Vec3)> {
    let mut trajectory = Vec::new();
    for step in 0..steps {
        let time = step as f32 / 60.0;
        
        // Get current state
        let world = app.world();
        if let Some(transform) = world.get::<Transform>(vehicle_entity) {
            if let Some(velocity) = world.get::<Velocity>(vehicle_entity) {
                trajectory.push((time, transform.translation, velocity.linvel));
            }
        // Run one step
    trajectory
/// Physics validation helper
pub struct PhysicsValidator;
impl PhysicsValidator {
    pub fn validate_velocity(velocity: &Velocity, max_speed: f32) -> Result<(), String> {
        if !velocity.linvel.is_finite() {
            return Err("Linear velocity is not finite".to_string());
        if !velocity.angvel.is_finite() {
            return Err("Angular velocity is not finite".to_string());
        if velocity.linvel.length() > max_speed {
            return Err(format!("Speed {} exceeds maximum {}", velocity.linvel.length(), max_speed));
        Ok(())
    pub fn validate_position(transform: &Transform, bounds: f32) -> Result<(), String> {
        if !transform.translation.is_finite() {
            return Err("Position is not finite".to_string());
        if transform.translation.length() > bounds {
            return Err(format!("Position {} exceeds bounds {}", transform.translation.length(), bounds));
    pub fn validate_mass(mass: f32, min_mass: f32, max_mass: f32) -> Result<(), String> {
        if !mass.is_finite() {
            return Err("Mass is not finite".to_string());
        if mass < min_mass {
            return Err(format!("Mass {} below minimum {}", mass, min_mass));
        if mass > max_mass {
            return Err(format!("Mass {} exceeds maximum {}", mass, max_mass));
