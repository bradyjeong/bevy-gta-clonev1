//! Tests for UI logic - state machines, telemetry computations, HUD updates

use bevy::prelude::*;
use game_core::prelude::*;
use gameplay_ui::prelude::*;
use gameplay_ui::systems::ui::bugatti_telemetry::*;
use crate::utils::*;
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test telemetry state initialization
    #[test]
    fn test_telemetry_state_initialization() {
        let state = BugattiTelemetryState::new();
        assert!(!state.visible);
        assert_eq!(state.last_update, 0.0);
        assert_eq!(state.update_interval, 0.033);
    }
    /// Test telemetry state default
    fn test_telemetry_state_default() {
        let state = BugattiTelemetryState::default();
    /// Test speed calculation from RPM
    fn test_speed_calculation() {
        let rpm = 4200.0;
        let max_rpm = 7000.0;
        let max_speed = 261.0;
        
        let calculated_speed = (rpm / max_rpm) * max_speed;
        let expected_speed = 156.6;
        assert!((calculated_speed - expected_speed).abs() < 0.1, 
                "Speed calculation inaccurate: {} vs {}", calculated_speed, expected_speed);
    /// Test telemetry validation
    fn test_telemetry_validation() {
        // Valid data
        let result = TelemetryValidator::validate_speed_calculation(4200.0, 7000.0, 261.0);
        assert!(result.is_ok());
        // Invalid data
        let result = TelemetryValidator::validate_speed_calculation(-100.0, 7000.0, 261.0);
        assert!(result.is_err());
        let result = TelemetryValidator::validate_speed_calculation(4200.0, 0.0, 261.0);
    /// Test turbo stage validation
    fn test_turbo_stage_validation() {
        // Valid stages
        for stage in 0..=4 {
            assert!(TelemetryValidator::validate_turbo_stage(stage).is_ok());
        }
        // Invalid stages
        assert!(TelemetryValidator::validate_turbo_stage(5).is_err());
        assert!(TelemetryValidator::validate_turbo_stage(255).is_err());
    /// Test G-force calculation
    fn test_g_force_calculation() {
        let lateral = 0.5;
        let longitudinal = 0.8;
        let expected_total = (lateral.powi(2) + longitudinal.powi(2)).sqrt();
        let result = TelemetryValidator::validate_g_force(lateral, longitudinal);
        let calculated = result.unwrap();
        assert!((calculated - expected_total).abs() < 0.001);
    /// Test HUD update interval validation
    fn test_update_interval_validation() {
        // Valid intervals
        assert!(UiStateValidator::validate_update_interval(0.033).is_ok());
        assert!(UiStateValidator::validate_update_interval(0.1).is_ok());
        assert!(UiStateValidator::validate_update_interval(0.5).is_ok());
        // Invalid intervals
        assert!(UiStateValidator::validate_update_interval(0.0).is_err());
        assert!(UiStateValidator::validate_update_interval(-0.1).is_err());
        assert!(UiStateValidator::validate_update_interval(2.0).is_err());
    /// Test telemetry formatting
    fn test_telemetry_text_formatting() {
        let supercar = create_test_supercar_data();
        // Test speed formatting
        let speed_text = format!("{:.0}\nMPH", supercar.max_speed);
        assert_eq!(speed_text, "261\nMPH");
        // Test RPM formatting
        let rpm_text = format!("{:.0}\nRPM", supercar.rpm);
        assert_eq!(rpm_text, "4200\nRPM");
        // Test turbo status
        let turbo_text = match supercar.turbo_stage {
            0 => "TURBO\nOFF",
            1 => "TURBO\n1/4",
            2 => "TURBO\n2/4",
            3 => "TURBO\n3/4",
            4 => "QUAD\nTURBO",
            _ => "TURBO\nMAX",
        };
        assert_eq!(turbo_text, "QUAD\nTURBO");
    /// Test driving mode display
    fn test_driving_mode_display() {
        let modes = vec![
            (DrivingMode::Comfort, "COMFORT"),
            (DrivingMode::Sport, "SPORT"),
            (DrivingMode::Track, "TRACK"),
            (DrivingMode::Custom, "CUSTOM"),
        ];
        for (mode, expected) in modes {
            let display = match mode {
                DrivingMode::Comfort => "COMFORT",
                DrivingMode::Sport => "SPORT",
                DrivingMode::Track => "TRACK",
                DrivingMode::Custom => "CUSTOM",
            };
            assert_eq!(display, expected);
    /// Test launch control status
    fn test_launch_control_status() {
        let statuses = vec![
            (false, false, "OFF"),
            (true, false, "READY"),
            (true, true, "LAUNCHING"),
        for (enabled, engaged, expected) in statuses {
            let status = if engaged {
                "LAUNCHING"
            } else if enabled {
                "READY"
            } else {
                "OFF"
            assert_eq!(status, expected);
    /// Test telemetry state machine
    fn test_telemetry_state_machine() {
        let mut app = create_ui_test_app();
        let (_player, _supercar) = setup_ui_test_scene(&mut app);
        // Initial state should be hidden
        let state = get_telemetry_state(&app);
        // Simulate F4 press to toggle visibility
        simulate_key_press(&mut app, KeyCode::F4);
        // After processing, state should be visible
        assert!(state.visible);
    /// Test telemetry update timing
    fn test_telemetry_update_timing() {
        // Enable telemetry
        // Run for several frames
        run_ui_updates(&mut app, 0.5);
        // Check that last_update has been incremented
        assert!(state.last_update > 0.0);
    /// Test supercar requirement for telemetry
    fn test_supercar_requirement() {
        // Create only player, no supercar
        let mut world = app.world_mut();
        world.spawn((
            Player::default(),
            ActiveEntity,
            Transform::default(),
            GlobalTransform::default(),
        ));
        // Try to toggle telemetry without supercar
        // Should remain hidden
    /// Test telemetry accuracy with edge cases
    fn test_telemetry_edge_cases() {
        // Test with zero RPM
        let result = TelemetryValidator::validate_speed_calculation(0.0, 7000.0, 261.0);
        assert_eq!(result.unwrap(), 0.0);
        // Test with max RPM
        let result = TelemetryValidator::validate_speed_calculation(7000.0, 7000.0, 261.0);
        assert_eq!(result.unwrap(), 261.0);
        // Test with infinite values
        let result = TelemetryValidator::validate_g_force(f32::INFINITY, 0.0);
        let result = TelemetryValidator::validate_g_force(0.0, f32::NAN);
    /// Test G-force extreme values
    fn test_g_force_extreme_values() {
        // Test realistic high G-force
        let result = TelemetryValidator::validate_g_force(3.0, 4.0);
        assert_eq!(result.unwrap(), 5.0);
        // Test unrealistic G-force
        let result = TelemetryValidator::validate_g_force(10.0, 10.0);
    /// Test UI state consistency
    fn test_ui_state_consistency() {
        // Test multiple toggles
        for i in 0..5 {
            simulate_key_press(&mut app, KeyCode::F4);
            let state = get_telemetry_state(&app);
            assert_eq!(state.visible, i % 2 == 0); // Toggle pattern
}
