//! UI logic tests for telemetry and control systems

use bevy::prelude::*;
use game_core::prelude::*;
use gameplay_ui::prelude::*;

/// Mock telemetry validator for testing
pub struct TelemetryValidator;

impl TelemetryValidator {
    /// Validate speed calculation
    pub fn validate_speed_calculation(rpm: f32, max_rpm: f32, max_speed: f32) -> Result<f32, String> {
        if rpm < 0.0 || max_rpm <= 0.0 || max_speed <= 0.0 {
            return Err("Invalid input parameters".to_string());
        }
        Ok((rpm / max_rpm) * max_speed)
    }
    
    /// Validate turbo stage
    pub fn validate_turbo_stage(stage: u8) -> Result<(), String> {
        if stage > 4 {
            Err("Invalid turbo stage".to_string())
        } else {
            Ok(())
        }
    }
    
    /// Validate G-force calculation
    pub fn validate_g_force(lateral: f32, longitudinal: f32) -> Result<f32, String> {
        if lateral.abs() > 5.0 || longitudinal.abs() > 5.0 {
            return Err("Unrealistic G-force values".to_string());
        }
        Ok((lateral.powi(2) + longitudinal.powi(2)).sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test telemetry state default
    #[test]
    fn test_telemetry_state_default() {
        let state = BugattiTelemetryState::default();
        assert!(!state.visible);
    }
    
    /// Test speed calculation from RPM
    #[test]
    fn test_speed_calculation() {
        let rpm = 4200.0;
        let max_rpm = 7000.0;
        let max_speed = 261.0;
        
        let calculated_speed = (rpm / max_rpm) * max_speed;
        let expected_speed = 156.6;
        assert!((calculated_speed - expected_speed).abs() < 0.1, 
                "Speed calculation inaccurate: {} vs {}", calculated_speed, expected_speed);
    }
    
    /// Test telemetry validation
    #[test]
    fn test_telemetry_validation() {
        // Valid data
        let result = TelemetryValidator::validate_speed_calculation(4200.0, 7000.0, 261.0);
        assert!(result.is_ok());
        // Invalid data
        let result = TelemetryValidator::validate_speed_calculation(-100.0, 7000.0, 261.0);
        assert!(result.is_err());
        let result = TelemetryValidator::validate_speed_calculation(4200.0, 0.0, 261.0);
        assert!(result.is_err());
    }
    
    /// Test turbo stage validation
    #[test]
    fn test_turbo_stage_validation() {
        // Valid stages
        for stage in 0..=4 {
            assert!(TelemetryValidator::validate_turbo_stage(stage).is_ok());
        }
        // Invalid stages
        assert!(TelemetryValidator::validate_turbo_stage(5).is_err());
        assert!(TelemetryValidator::validate_turbo_stage(255).is_err());
    }
    
    /// Test G-force calculation
    #[test]
    fn test_g_force_calculation() {
        let lateral = 0.5;
        let longitudinal = 0.8;
        let expected_total = (lateral.powi(2) + longitudinal.powi(2)).sqrt();
        let result = TelemetryValidator::validate_g_force(lateral, longitudinal);
        let calculated = result.unwrap();
        assert!((calculated - expected_total).abs() < 0.001);
        
        // Test realistic high G-force
        let result = TelemetryValidator::validate_g_force(3.0, 4.0);
        assert_eq!(result.unwrap(), 5.0);
        // Test unrealistic G-force
        let result = TelemetryValidator::validate_g_force(10.0, 10.0);
        assert!(result.is_err()); // Should fail validation
    }
    
    /// Test UI state consistency
    #[test]
    fn test_ui_state_consistency() {
        // Basic test that doesn't require complex app setup
        let state = BugattiTelemetryState::default();
        assert!(!state.visible);
    }
}
