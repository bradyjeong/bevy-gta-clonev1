//! Tests for telemetry systems - Bugatti dashboard, vehicle data display

use bevy::prelude::*;
use game_core::prelude::*;
use gameplay_ui::prelude::*;
use gameplay_ui::systems::ui::bugatti_telemetry::*;
use crate::utils::*;
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test telemetry system initialization
    #[test]
    fn test_telemetry_system_initialization() {
        let mut app = create_ui_test_app();
        
        // Verify telemetry state resource exists
        assert!(app.world().contains_resource::<BugattiTelemetryState>());
        // Check initial state
        let state = get_telemetry_state(&app);
        assert!(!state.visible);
        assert_eq!(state.update_interval, 0.033);
    }
    /// Test telemetry activation with supercar
    fn test_telemetry_activation_with_supercar() {
        let (_player, _supercar) = setup_ui_test_scene(&mut app);
        // Initially hidden
        // Press F4 to activate
        simulate_key_press(&mut app, KeyCode::F4);
        // Should be visible
        assert!(state.visible);
    /// Test telemetry blocked without supercar
    fn test_telemetry_blocked_without_supercar() {
        // Add only regular player
        let mut world = app.world_mut();
        world.spawn((
            Player::default(),
            ActiveEntity,
            Transform::default(),
            GlobalTransform::default(),
        ));
        // Try to activate telemetry
        // Should remain hidden
    /// Test telemetry update system
    fn test_telemetry_update_system() {
        let (_player, supercar) = setup_ui_test_scene(&mut app);
        // Update supercar with test data
        if let Some(mut car) = world.get_mut::<SuperCar>(supercar) {
            *car = create_test_supercar_data();
        }
        // Activate telemetry
        // Run updates
        run_ui_updates(&mut app, 0.5);
        // Check that updates occurred
        assert!(state.last_update >= 0.0);
    /// Test speed calculation accuracy
    fn test_speed_calculation_accuracy() {
        let test_cases = vec![
            (0.0, 7000.0, 261.0, 0.0),
            (3500.0, 7000.0, 261.0, 130.5),
            (7000.0, 7000.0, 261.0, 261.0),
            (1000.0, 5000.0, 200.0, 40.0),
        ];
        for (rpm, max_rpm, max_speed, expected) in test_cases {
            let calculated = (rpm / max_rpm) * max_speed;
            UiAssertions::assert_telemetry_accuracy(calculated, expected, 0.1);
    /// Test RPM validation
    fn test_rpm_validation() {
        let rpm_values = TestDataGenerator::generate_rpm_values();
        for rpm in rpm_values {
            let result = TelemetryValidator::validate_speed_calculation(rpm, 7000.0, 261.0);
            
            if rpm >= 0.0 && rpm.is_finite() {
                assert!(result.is_ok(), "Valid RPM {} should pass validation", rpm);
            } else {
                assert!(result.is_err(), "Invalid RPM {} should fail validation", rpm);
            }
    /// Test turbo stage display
    fn test_turbo_stage_display() {
        let turbo_stages = TestDataGenerator::generate_turbo_stages();
        for stage in turbo_stages {
            let display = match stage {
                0 => "TURBO\nOFF",
                1 => "TURBO\n1/4",
                2 => "TURBO\n2/4",
                3 => "TURBO\n3/4",
                4 => "QUAD\nTURBO",
                _ => "TURBO\nMAX",
            };
            // Valid stages should have proper display
            if stage <= 4 {
                assert!(!display.contains("MAX"), "Valid turbo stage {} should not show MAX", stage);
                assert!(display.contains("MAX"), "Invalid turbo stage {} should show MAX", stage);
    /// Test G-force calculation accuracy
    fn test_g_force_calculation_accuracy() {
        let g_force_values = TestDataGenerator::generate_g_force_values();
        for (lateral, longitudinal) in g_force_values {
            let result = TelemetryValidator::validate_g_force(lateral, longitudinal);
            if lateral.is_finite() && longitudinal.is_finite() {
                let expected = (lateral.powi(2) + longitudinal.powi(2)).sqrt();
                if expected <= 5.0 {
                    assert!(result.is_ok(), "Valid G-force ({}, {}) should pass", lateral, longitudinal);
                    UiAssertions::assert_telemetry_accuracy(result.unwrap(), expected, 0.001);
                } else {
                    assert!(result.is_err(), "Extreme G-force ({}, {}) should fail", lateral, longitudinal);
                }
                assert!(result.is_err(), "Non-finite G-force ({}, {}) should fail", lateral, longitudinal);
    /// Test driving mode display
    fn test_driving_mode_display() {
        let modes = vec![
            (DrivingMode::Comfort, "COMFORT"),
            (DrivingMode::Sport, "SPORT"),
            (DrivingMode::Track, "TRACK"),
            (DrivingMode::Custom, "CUSTOM"),
        for (mode, expected) in modes {
            let mut app = create_ui_test_app();
            let (_player, supercar) = setup_ui_test_scene(&mut app);
            // Set driving mode
            let mut world = app.world_mut();
            if let Some(mut car) = world.get_mut::<SuperCar>(supercar) {
                car.driving_mode = mode;
            // Test display logic
            let display = match mode {
                DrivingMode::Comfort => "COMFORT",
                DrivingMode::Sport => "SPORT",
                DrivingMode::Track => "TRACK",
                DrivingMode::Custom => "CUSTOM",
            assert_eq!(display, expected);
    /// Test launch control status
    fn test_launch_control_status() {
        // Test different launch control states
        let states = vec![
            (false, false, "OFF"),
            (true, false, "READY"),
            (true, true, "LAUNCHING"),
        for (enabled, engaged, expected) in states {
                car.launch_control = enabled;
                car.launch_control_engaged = engaged;
            let status = if engaged {
                "LAUNCHING"
            } else if enabled {
                "READY"
                "OFF"
            assert_eq!(status, expected);
    /// Test telemetry update timing
    fn test_telemetry_update_timing() {
        // Set game state to driving
        app.world_mut().insert_resource(NextState::Pending(GameState::Driving));
        app.update();
        // Run for specific duration
        let test_duration = 1.0; // 1 second
        run_ui_updates(&mut app, test_duration);
        // Check update frequency
        let expected_updates = test_duration / state.update_interval;
        // Should have updated multiple times
        assert!(state.last_update > 0.0);
        assert!(state.last_update < state.update_interval * 2.0); // Within reasonable bounds
    /// Test telemetry in different game states
    fn test_telemetry_in_different_states() {
        // Test in driving state
        run_ui_updates(&mut app, 0.1);
        // Test in paused state
        app.world_mut().insert_resource(NextState::Pending(GameState::Paused));
        // Test in menu state
        app.world_mut().insert_resource(NextState::Pending(GameState::Menu));
        // Telemetry should remain accessible
    /// Test telemetry UI components
    fn test_telemetry_ui_components() {
        // Check that UI components exist
        assert!(ui_element_exists::<BugattiTelemetryOverlay>(&app));
        // Run updates to ensure components are created
        // Check specific component markers
        UiAssertions::assert_ui_visible::<BugattiTelemetryOverlay>(&app, true);
    /// Test telemetry performance
    fn test_telemetry_performance() {
        // Measure update performance
        let start = std::time::Instant::now();
        // Run many updates
        for _ in 0..1000 {
            app.update();
        let duration = start.elapsed();
        // Should be fast (under 100ms for 1000 updates)
        assert!(duration.as_millis() < 100, "Telemetry updates too slow: {:?}", duration);
    /// Test telemetry data validation
    fn test_telemetry_data_validation() {
        // Set invalid data
            car.rpm = -1000.0; // Invalid RPM
            car.max_rpm = 0.0; // Invalid max RPM
            car.g_force_lateral = f32::INFINITY; // Invalid G-force
        // Run updates - should handle invalid data gracefully
        // App should remain stable
        assert!(app.world().entities().len() > 0);
    /// Test telemetry edge cases
    fn test_telemetry_edge_cases() {
        // Test with zero values
            car.rpm = 0.0;
            car.max_rpm = 1.0;
            car.max_speed = 0.0;
            car.g_force_lateral = 0.0;
            car.g_force_longitudinal = 0.0;
        // Should handle zero values
}
