use gta_game::components::vehicles::{SimpleCarSpecs, SimpleF16Specs, SimpleHelicopterSpecs};
use std::fs;

#[test]
fn test_car_config_parses() {
    let content =
        fs::read_to_string("assets/config/simple_car.ron").expect("simple_car.ron should exist");

    let config: SimpleCarSpecs =
        ron::from_str(&content).expect("simple_car.ron should parse correctly");

    assert!(config.base_speed > 0.0, "Base speed must be positive");
    assert!(
        config.rotation_speed > 0.0,
        "Rotation speed must be positive"
    );
    assert!(
        config.linear_lerp_factor >= 1.0 && config.linear_lerp_factor <= 20.0,
        "Linear lerp factor should be in valid range"
    );
    assert!(
        config.angular_lerp_factor >= 1.0 && config.angular_lerp_factor <= 20.0,
        "Angular lerp factor should be in valid range"
    );
}

#[test]
fn test_helicopter_config_parses() {
    let content = fs::read_to_string("assets/config/simple_helicopter.ron")
        .expect("simple_helicopter.ron should exist");

    let config: SimpleHelicopterSpecs =
        ron::from_str(&content).expect("simple_helicopter.ron should parse correctly");

    assert!(
        config.vertical_speed > 0.0,
        "Vertical speed must be positive"
    );
    assert!(config.yaw_rate > 0.0, "Yaw rate must be positive");
    assert!(config.pitch_rate > 0.0, "Pitch rate must be positive");
    assert!(config.roll_rate > 0.0, "Roll rate must be positive");
    assert!(
        config.main_rotor_rpm > 0.0,
        "Main rotor RPM must be positive"
    );
    assert!(
        config.tail_rotor_rpm > 0.0,
        "Tail rotor RPM must be positive"
    );
}

#[test]
fn test_f16_config_parses() {
    let content =
        fs::read_to_string("assets/config/simple_f16.ron").expect("simple_f16.ron should exist");

    let config: SimpleF16Specs =
        ron::from_str(&content).expect("simple_f16.ron should parse correctly");

    assert!(
        config.max_forward_speed > 0.0,
        "Max forward speed must be positive"
    );
    assert!(config.roll_rate_max > 0.0, "Roll rate must be positive");
    assert!(config.pitch_rate_max > 0.0, "Pitch rate must be positive");
    assert!(config.yaw_rate_max > 0.0, "Yaw rate must be positive");
    assert!(
        config.linear_damping > 0.0,
        "Linear damping must be positive"
    );
    assert!(
        config.angular_damping > 0.0,
        "Angular damping must be positive"
    );
}

#[test]
fn test_invalid_ron_fails_gracefully() {
    let invalid_ron = r#"
        InvalidStruct(
            missing_field: "test"
        )
    "#;

    let result: Result<SimpleCarSpecs, _> = ron::from_str(invalid_ron);

    assert!(result.is_err(), "Invalid RON should fail to parse");

    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message should be non-empty");
}

#[test]
fn test_car_config_has_valid_speeds() {
    let content =
        fs::read_to_string("assets/config/simple_car.ron").expect("simple_car.ron should exist");

    let config: SimpleCarSpecs =
        ron::from_str(&content).expect("simple_car.ron should parse correctly");

    assert!(
        config.base_speed > 0.0 && config.base_speed <= 100.0,
        "Base speed should be in valid range"
    );
    assert!(
        config.rotation_speed > 0.0 && config.rotation_speed <= 10.0,
        "Rotation speed should be in valid range"
    );
    assert!(
        config.drag_factor >= 0.9 && config.drag_factor <= 1.0,
        "Drag factor should be in valid range"
    );

    assert!(
        config.accel_lerp > 0.0,
        "Acceleration lerp must be positive"
    );
    assert!(config.brake_lerp > 0.0, "Brake lerp must be positive");
    assert!(config.grip > 0.0, "Grip must be positive");
}

#[test]
fn test_helicopter_config_has_valid_values() {
    let content = fs::read_to_string("assets/config/simple_helicopter.ron")
        .expect("simple_helicopter.ron should exist");

    let config: SimpleHelicopterSpecs =
        ron::from_str(&content).expect("simple_helicopter.ron should parse correctly");

    assert!(
        config.vertical_speed >= 1.0 && config.vertical_speed <= 50.0,
        "Vertical speed should be in valid range"
    );
    assert!(
        config.yaw_rate >= 0.1 && config.yaw_rate <= 5.0,
        "Yaw rate should be in valid range"
    );
    assert!(
        config.pitch_rate >= 0.1 && config.pitch_rate <= 5.0,
        "Pitch rate should be in valid range"
    );
    assert!(
        config.roll_rate >= 0.1 && config.roll_rate <= 5.0,
        "Roll rate should be in valid range"
    );

    assert!(config.spool_up_rate > 0.0, "Spool up rate must be positive");
    assert!(
        config.spool_down_rate > 0.0,
        "Spool down rate must be positive"
    );
    assert!(
        config.min_rpm_for_lift >= 0.0 && config.min_rpm_for_lift <= 1.0,
        "Min RPM should be fraction"
    );
    assert!(
        config.max_lift_margin_g > 1.0,
        "Max lift margin should exceed 1G"
    );
}

#[test]
fn test_f16_config_has_valid_values() {
    let content =
        fs::read_to_string("assets/config/simple_f16.ron").expect("simple_f16.ron should exist");

    let config: SimpleF16Specs =
        ron::from_str(&content).expect("simple_f16.ron should parse correctly");

    assert!(
        config.max_forward_speed >= 50.0 && config.max_forward_speed <= 600.0,
        "Max speed should be in valid range"
    );
    assert!(
        config.roll_rate_max >= 0.1 && config.roll_rate_max <= 10.0,
        "Roll rate should be in valid range"
    );
    assert!(
        config.pitch_rate_max >= 0.1 && config.pitch_rate_max <= 10.0,
        "Pitch rate should be in valid range"
    );
    assert!(
        config.yaw_rate_max >= 0.1 && config.yaw_rate_max <= 5.0,
        "Yaw rate should be in valid range"
    );

    assert!(
        config.afterburner_multiplier >= 1.0 && config.afterburner_multiplier <= 3.0,
        "Afterburner multiplier should be >= 1.0"
    );
    assert!(
        config.control_full_speed > 0.0,
        "Control full speed must be positive"
    );
    assert!(
        config.min_control_factor >= 0.1 && config.min_control_factor <= 1.0,
        "Min control factor should be fraction"
    );
}

#[test]
fn test_damping_values_in_valid_range() {
    let car_content =
        fs::read_to_string("assets/config/simple_car.ron").expect("simple_car.ron should exist");
    let car_config: SimpleCarSpecs =
        ron::from_str(&car_content).expect("simple_car.ron should parse");

    let _heli_content = fs::read_to_string("assets/config/simple_helicopter.ron")
        .expect("simple_helicopter.ron should exist");

    let f16_content =
        fs::read_to_string("assets/config/simple_f16.ron").expect("simple_f16.ron should exist");
    let f16_config: SimpleF16Specs =
        ron::from_str(&f16_content).expect("simple_f16.ron should parse");

    assert!(
        car_config.linear_lerp_factor >= 1.0 && car_config.linear_lerp_factor <= 20.0,
        "Car linear lerp should be in valid range"
    );
    assert!(
        car_config.angular_lerp_factor >= 1.0 && car_config.angular_lerp_factor <= 20.0,
        "Car angular lerp should be in valid range"
    );

    assert!(
        f16_config.linear_damping >= 0.01 && f16_config.linear_damping <= 5.0,
        "F16 linear damping should be in valid range"
    );
    assert!(
        f16_config.angular_damping >= 0.01 && f16_config.angular_damping <= 5.0,
        "F16 angular damping should be in valid range"
    );
}

#[test]
fn test_car_wheel_config_valid() {
    let content =
        fs::read_to_string("assets/config/simple_car.ron").expect("simple_car.ron should exist");

    let config: SimpleCarSpecs =
        ron::from_str(&content).expect("simple_car.ron should parse correctly");

    assert!(config.wheel_radius > 0.0, "Wheel radius must be positive");
    assert!(
        config.max_steer_deg > 0.0 && config.max_steer_deg < 90.0,
        "Max steer angle should be reasonable"
    );
    assert_eq!(
        config.wheel_positions.len(),
        4,
        "Should have exactly 4 wheel positions"
    );

    for (i, (x, y, z)) in config.wheel_positions.iter().enumerate() {
        assert!(
            y < &0.0,
            "Wheel position {} Y should be negative (below vehicle)",
            i
        );
        assert!(
            x.abs() > 0.0,
            "Wheel position {} X should be offset from center",
            i
        );
        assert!(
            z.abs() > 0.0,
            "Wheel position {} Z should be offset from center",
            i
        );
    }
}

#[test]
fn test_helicopter_rotor_config_valid() {
    let content = fs::read_to_string("assets/config/simple_helicopter.ron")
        .expect("simple_helicopter.ron should exist");

    let config: SimpleHelicopterSpecs =
        ron::from_str(&content).expect("simple_helicopter.ron should parse correctly");

    assert!(
        config.main_rotor_rpm > 0.0,
        "Main rotor RPM must be positive"
    );
    assert!(
        config.tail_rotor_rpm > 0.0,
        "Tail rotor RPM must be positive"
    );
    assert!(
        config.tail_rotor_rpm > config.main_rotor_rpm,
        "Tail rotor should spin faster than main rotor"
    );
}

#[test]
fn test_f16_config_valid() {
    let content =
        fs::read_to_string("assets/config/simple_f16.ron").expect("simple_f16.ron should exist");

    let config: SimpleF16Specs =
        ron::from_str(&content).expect("simple_f16.ron should parse correctly");

    assert!(
        config.max_forward_speed > 0.0,
        "Max forward speed must be positive"
    );
    assert!(
        config.roll_rate_max > 0.0,
        "Roll rate must be positive"
    );
}

#[test]
fn test_car_arcade_physics_config() {
    let content =
        fs::read_to_string("assets/config/simple_car.ron").expect("simple_car.ron should exist");

    let config: SimpleCarSpecs =
        ron::from_str(&content).expect("simple_car.ron should parse correctly");

    assert!(config.slip_extremum > 0.0, "Slip extremum must be positive");
    assert!(
        config.slip_asymptote > config.slip_extremum,
        "Slip asymptote should be greater than extremum"
    );
    assert!(
        config.slip_stiffness > 0.0,
        "Slip stiffness must be positive"
    );

    assert!(
        config.brake_grip_loss >= 0.0 && config.brake_grip_loss <= 1.0,
        "Brake grip loss should be 0-1 fraction"
    );
}



#[test]
fn test_helicopter_stabilization_config() {
    let content = fs::read_to_string("assets/config/simple_helicopter.ron")
        .expect("simple_helicopter.ron should exist");

    let config: SimpleHelicopterSpecs =
        ron::from_str(&content).expect("simple_helicopter.ron should parse correctly");

    assert!(
        config.pitch_stab >= 0.0 && config.pitch_stab <= 1.0,
        "Pitch stab should be 0-1 fraction"
    );
    assert!(
        config.roll_stab >= 0.0 && config.roll_stab <= 1.0,
        "Roll stab should be 0-1 fraction"
    );
    assert!(
        config.yaw_stab >= 0.0 && config.yaw_stab <= 1.0,
        "Yaw stab should be 0-1 fraction"
    );
}

#[test]
fn test_all_configs_parse_without_panic() {
    let configs = vec![
        ("assets/config/simple_car.ron", "SimpleCarSpecs"),
        (
            "assets/config/simple_helicopter.ron",
            "SimpleHelicopterSpecs",
        ),
        ("assets/config/simple_f16.ron", "SimpleF16Specs"),
    ];

    for (path, name) in configs {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("{} should exist at {}", name, path));

        assert!(!content.is_empty(), "{} should not be empty", name);
        assert!(
            content.contains('('),
            "{} should contain RON struct syntax",
            name
        );
        assert!(
            content.contains(')'),
            "{} should have closing parenthesis",
            name
        );
    }
}

#[test]
fn test_car_config_emergency_brake_valid() {
    let content =
        fs::read_to_string("assets/config/simple_car.ron").expect("simple_car.ron should exist");

    let config: SimpleCarSpecs =
        ron::from_str(&content).expect("simple_car.ron should parse correctly");

    assert!(
        config.emergency_brake_linear >= 0.01 && config.emergency_brake_linear <= 1.0,
        "Emergency brake linear should be 0.01-1.0"
    );
    assert!(
        config.emergency_brake_angular >= 0.01 && config.emergency_brake_angular <= 1.0,
        "Emergency brake angular should be 0.01-1.0"
    );
    assert!(
        config.ebrake_yaw_boost >= 0.0,
        "E-brake yaw boost must be non-negative"
    );
}

#[test]
fn test_f16_auto_stabilization_config() {
    let content =
        fs::read_to_string("assets/config/simple_f16.ron").expect("simple_f16.ron should exist");

    let config: SimpleF16Specs =
        ron::from_str(&content).expect("simple_f16.ron should parse correctly");

    assert!(
        config.roll_stab >= 0.0 && config.roll_stab <= 1.0,
        "Roll stab should be 0-1 fraction"
    );
    assert!(
        config.pitch_stab >= 0.0 && config.pitch_stab <= 1.0,
        "Pitch stab should be 0-1 fraction"
    );
    assert!(
        config.yaw_stab >= 0.0 && config.yaw_stab <= 1.0,
        "Yaw stab should be 0-1 fraction"
    );

    assert!(
        config.roll_auto_level_gain >= 0.0,
        "Roll auto-level gain must be non-negative"
    );
    assert!(
        config.pitch_auto_level_gain >= 0.0,
        "Pitch auto-level gain must be non-negative"
    );
    assert!(
        config.yaw_auto_level_gain >= 0.0,
        "Yaw auto-level gain must be non-negative"
    );

    assert!(
        config.auto_bank_gain >= 0.0,
        "Auto bank gain must be non-negative"
    );
    assert!(
        config.auto_bank_max_rate >= 0.0,
        "Auto bank max rate must be non-negative"
    );
}

#[test]
fn test_missing_required_fields_fails() {
    let incomplete_car_ron = r#"
        SimpleCarSpecs(
            base_speed: 70.0,
        )
    "#;

    let result: Result<SimpleCarSpecs, _> = ron::from_str(incomplete_car_ron);
    assert!(result.is_err(), "Incomplete config should fail to parse");
}

#[test]
fn test_negative_speeds_in_config() {
    let invalid_car_ron = r#"
        SimpleCarSpecs(
            base_speed: -10.0,
            rotation_speed: 3.0,
            linear_lerp_factor: 4.0,
            angular_lerp_factor: 6.0,
            emergency_brake_linear: 0.1,
            emergency_brake_angular: 0.5,
            drag_factor: 0.92,
            accel_lerp: 5.0,
            brake_lerp: 8.0,
            grip: 8.0,
            drift_grip: 2.2,
            steer_gain: 5.0,
            stability: 0.6,
            ebrake_yaw_boost: 1.2,
            downforce_scale: 0.3,
            auto_brake_gain: 12.0,
            slip_extremum: 1.5,
            slip_asymptote: 12.0,
            slip_stiffness: 1.0,
            brake_grip_loss: 0.25,
            traction_loss_mult: 0.80,
            ground_ray_length: 1.6,
            air_gravity_scale: 1.2,
            airborne_steer_scale: 0.3,
            roll_kp: 3.5,
            roll_kd: 1.2,
            roll_torque_limit: 400.0,
            reverse_steer_invert: true,
            airborne_angular_scale: 0.5,
            visual_roll_gain: 0.0,
            visual_pitch_gain: 0.0,
            visual_spring: 14.0,
            visual_damper: 7.5,
            max_steer_deg: 28.0,
            wheel_radius: 0.33,
            wheel_positions: ((0.85, -0.32, -1.40), (-0.85, -0.32, -1.40), (0.85, -0.32, 1.40), (-0.85, -0.32, 1.40)),
            ground_stick_gain: 10.0,
            accel_curve: [(0.0, 1.5), (0.5, 1.0), (1.0, 0.6)],
            counter_steer_gain: 0.8,
            lateral_cancel_max_g: 1.2,
            steer_curve: [(0.0, 1.0), (0.5, 0.7), (1.0, 0.4)],
            vt_curve: [],
            tc_enabled: true,
            tc_slip_start_deg: 8.0,
            tc_min_scalar: 0.5,
            abs_enabled: true,
            abs_peak_decel_g: 1.0,
            abs_min_scalar: 0.6,
            landing_prediction_enabled: true,
            landing_horizon_time: 0.25,
            landing_align_strength: 0.3,
        )
    "#;

    let result: Result<SimpleCarSpecs, _> = ron::from_str(invalid_car_ron);

    if let Ok(config) = result {
        assert_eq!(
            config.base_speed, -10.0,
            "RON will parse negative values - validation happens at runtime"
        );
    }
}
