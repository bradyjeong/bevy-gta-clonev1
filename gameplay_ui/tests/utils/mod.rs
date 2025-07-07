//! Test utilities for gameplay_ui tests
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::prelude::*;
use gameplay_ui::prelude::*;
use gameplay_ui::systems::ui::bugatti_telemetry::BugattiTelemetryState;
use gameplay_ui::systems::performance_monitor::UnifiedPerformanceTracker;
use gameplay_ui::systems::performance_dashboard::PerformanceDashboard;
use std::collections::HashMap;

/// Test configuration for UI testing
pub struct UiTestConfig {
    pub headless: bool,
    pub fixed_timestep: f32,
    pub max_test_duration: f32,
    pub ui_update_interval: f32,
}
impl Default for UiTestConfig {
    fn default() -> Self {
        Self {
            headless: true,
            fixed_timestep: 1.0 / 60.0,
            max_test_duration: 5.0,
            ui_update_interval: 0.033, // 30 FPS for UI updates
        }
    }
/// Create a headless test app for UI testing
pub fn create_ui_test_app() -> App {
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
    // Add UI plugin
    app.add_plugins(UiPlugin);
    // Set fixed timestep
    app.insert_resource(FixedTime::new_from_secs(1.0 / 60.0));
    // Initialize input resource for testing
    app.init_resource::<ButtonInput<KeyCode>>();
    // Add game state
    app.init_state::<GameState>();
    app
/// Create a test scene with UI elements
pub fn setup_ui_test_scene(app: &mut App) -> (Entity, Entity) {
    let mut world = app.world_mut();
    // Create a player entity for UI context
    let player = world.spawn((
        Player::default(),
        ActiveEntity,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Velocity::default(),
        RigidBody::Dynamic,
        Collider::capsule_y(0.5, 0.5),
    )).id();
    // Create a supercar for telemetry testing
    let supercar = world.spawn((
        Car::default(),
        SuperCar::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Collider::cuboid(2.0, 1.0, 4.5),
    (player, supercar)
/// Mock supercar data for testing
pub fn create_test_supercar_data() -> SuperCar {
    SuperCar {
        rpm: 4200.0,
        max_rpm: 7000.0,
        max_speed: 261.0,
        turbo_stage: 4,
        driving_mode: game_core::components::DrivingMode::Sport,
        launch_control: true,
        launch_control_engaged: false,
        g_force_lateral: 0.5,
        g_force_longitudinal: 0.8,
        ..default()
/// Simulate key press for testing
pub fn simulate_key_press(app: &mut App, key: KeyCode) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.press(key);
    app.update();
    input.release(key);
/// Simulate key hold for testing
pub fn simulate_key_hold(app: &mut App, key: KeyCode, duration_frames: u32) {
    for _ in 0..duration_frames {
        app.update();
/// Run UI updates for specified duration
pub fn run_ui_updates(app: &mut App, duration_seconds: f32) {
    let frames = (duration_seconds / (1.0 / 60.0)) as u32;
    for _ in 0..frames {
/// Check if UI element exists with specific component
pub fn ui_element_exists<T: Component>(app: &App) -> bool {
    app.world().query::<&T>().iter(app.world()).count() > 0
/// Get telemetry state for testing
pub fn get_telemetry_state(app: &App) -> &BugattiTelemetryState {
    app.world().resource::<BugattiTelemetryState>()
/// Get performance tracker for testing
pub fn get_performance_tracker(app: &App) -> &UnifiedPerformanceTracker {
    app.world().resource::<UnifiedPerformanceTracker>()
/// Mock performance data for testing
pub fn create_mock_performance_data() -> HashMap<String, f32> {
    let mut data = HashMap::new();
    data.insert("physics_update".to_string(), 2.5);
    data.insert("rendering".to_string(), 8.3);
    data.insert("ui_update".to_string(), 1.2);
    data.insert("culling".to_string(), 0.8);
    data.insert("spawning".to_string(), 0.3);
    data
/// Telemetry validator for testing
pub struct TelemetryValidator;
impl TelemetryValidator {
    pub fn validate_speed_calculation(rpm: f32, max_rpm: f32, max_speed: f32) -> Result<f32, String> {
        if rpm < 0.0 {
            return Err("RPM cannot be negative".to_string());
        if max_rpm <= 0.0 {
            return Err("Max RPM must be positive".to_string());
        if max_speed <= 0.0 {
            return Err("Max speed must be positive".to_string());
        
        let calculated_speed = (rpm / max_rpm) * max_speed;
        if calculated_speed > max_speed {
            return Err("Calculated speed exceeds maximum".to_string());
        Ok(calculated_speed)
    pub fn validate_turbo_stage(stage: u8) -> Result<(), String> {
        if stage > 4 {
            return Err("Turbo stage cannot exceed 4".to_string());
        Ok(())
    pub fn validate_g_force(lateral: f32, longitudinal: f32) -> Result<f32, String> {
        if !lateral.is_finite() || !longitudinal.is_finite() {
            return Err("G-force values must be finite".to_string());
        let total_g = (lateral.powi(2) + longitudinal.powi(2)).sqrt();
        if total_g > 5.0 {
            return Err("G-force exceeds realistic limits".to_string());
        Ok(total_g)
/// UI state validator for testing
pub struct UiStateValidator;
impl UiStateValidator {
    pub fn validate_visibility_state(visible: bool, should_be_visible: bool) -> Result<(), String> {
        if visible != should_be_visible {
            return Err(format!("Visibility mismatch: expected {}, got {}", should_be_visible, visible));
    pub fn validate_update_interval(interval: f32) -> Result<(), String> {
        if interval <= 0.0 {
            return Err("Update interval must be positive".to_string());
        if interval > 1.0 {
            return Err("Update interval too high (>1s)".to_string());
/// Performance metrics validator
pub struct PerformanceValidator;
impl PerformanceValidator {
    pub fn validate_frame_time(frame_time: f32) -> Result<(), String> {
        if !frame_time.is_finite() {
            return Err("Frame time must be finite".to_string());
        if frame_time < 0.0 {
            return Err("Frame time cannot be negative".to_string());
        if frame_time > 100.0 {
            return Err("Frame time too high (>100ms)".to_string());
    pub fn validate_fps(fps: f32) -> Result<(), String> {
        if !fps.is_finite() {
            return Err("FPS must be finite".to_string());
        if fps < 0.0 {
            return Err("FPS cannot be negative".to_string());
        if fps > 1000.0 {
            return Err("FPS too high (>1000)".to_string());
    pub fn validate_memory_usage(memory_gb: f32) -> Result<(), String> {
        if !memory_gb.is_finite() {
            return Err("Memory usage must be finite".to_string());
        if memory_gb < 0.0 {
            return Err("Memory usage cannot be negative".to_string());
        if memory_gb > 64.0 {
            return Err("Memory usage too high (>64GB)".to_string());
/// Test data generator for property testing
pub struct TestDataGenerator;
impl TestDataGenerator {
    pub fn generate_rpm_values() -> Vec<f32> {
        vec![
            0.0, 1000.0, 2000.0, 3000.0, 4000.0, 5000.0, 6000.0, 7000.0,
            500.5, 1234.7, 6789.3, 999.9, 7000.1, -100.0, f32::INFINITY
        ]
    pub fn generate_speed_values() -> Vec<f32> {
            0.0, 30.0, 60.0, 120.0, 180.0, 240.0, 261.0,
            15.5, 87.3, 199.7, 260.9, 300.0, -50.0, f32::NAN
    pub fn generate_turbo_stages() -> Vec<u8> {
        vec![0, 1, 2, 3, 4, 5, 10, 255]
    pub fn generate_g_force_values() -> Vec<(f32, f32)> {
            (0.0, 0.0), (0.5, 0.3), (1.0, 0.8), (1.5, 1.2),
            (2.0, 1.5), (0.0, 3.0), (4.0, 0.0), (2.5, 2.5),
            (-1.0, 0.5), (0.5, -1.0), (f32::INFINITY, 0.0),
            (0.0, f32::NAN), (10.0, 10.0)
/// Mock input events for testing
pub fn create_mock_input_sequence() -> Vec<KeyCode> {
    vec![
        KeyCode::F3,  // Toggle performance overlay
        KeyCode::F4,  // Toggle telemetry
        KeyCode::Escape, // Menu
        KeyCode::Tab,    // Switch mode
        KeyCode::Enter,  // Confirm
        KeyCode::Space,  // Action
    ]
/// Test assertions for UI elements
pub struct UiAssertions;
impl UiAssertions {
    pub fn assert_ui_visible<T: Component>(app: &App, should_be_visible: bool) {
        let visible = ui_element_exists::<T>(app);
        assert_eq!(visible, should_be_visible, "UI element visibility mismatch");
    pub fn assert_telemetry_accuracy(calculated: f32, expected: f32, tolerance: f32) {
        let diff = (calculated - expected).abs();
        assert!(diff <= tolerance, "Telemetry calculation inaccurate: {} vs {} (tolerance: {})", 
                calculated, expected, tolerance);
    pub fn assert_performance_within_bounds(value: f32, min: f32, max: f32) {
        assert!(value >= min && value <= max, 
                "Performance metric {} outside bounds [{}, {}]", value, min, max);
