//! ───────────────────────────────────────────────
//! System:   Bugatti Telemetry
//! Purpose:  Processes user input and control mapping
//! Schedule: Update
//! Reads:    ActiveEntity, BugattiRpmGauge, Car, BugattiTelemetryState, BugattiSpeedometer
//! Writes:   Visibility, BugattiTelemetryState, Text
//! Invariants:
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;

/// Component marker for the Bugatti telemetry dashboard
#[derive(Component)]
pub struct BugattiTelemetryOverlay;

#[derive(Component)]
pub struct BugattiSpeedometer;

#[derive(Component)]
pub struct BugattiRpmGauge;

#[derive(Component)]
pub struct BugattiTurboIndicator;

#[derive(Component)]
pub struct BugattiInfoPanel;

/// Resource to track dashboard visibility and state
#[derive(Resource, Default)]
pub struct BugattiTelemetryState {
    pub visible: bool,
    pub last_update: f32,
    pub update_interval: f32,
}

impl BugattiTelemetryState {
    pub fn new() -> Self {
        Self {
            visible: false,
            last_update: 0.0,
            update_interval: 0.033, // ~30fps for smooth dashboard updates
        }
    }
}

/// System to handle F4 key toggle for Bugatti telemetry
pub fn bugatti_telemetry_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut telemetry_state: ResMut<BugattiTelemetryState>,
    mut overlay_query: Query<&mut Visibility, With<BugattiTelemetryOverlay>>,
    mut commands: Commands,
    supercar_query: Query<&SuperCar, (With<Car>, With<ActiveEntity>)>,
) {
    if keys.just_pressed(KeyCode::F4) {
        // Only show telemetry if we're driving a SuperCar
        if supercar_query.single().is_ok() {
            telemetry_state.visible = !telemetry_state.visible;
            
            // Toggle existing overlay visibility or create new one
            if let Ok(mut visibility) = overlay_query.single_mut() {
                *visibility = if telemetry_state.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            } else if telemetry_state.visible {
                // Create overlay if it doesn't exist
                spawn_bugatti_dashboard(&mut commands);
            }
        }
    }
}

/// Spawn the luxury Bugatti dashboard UI positioned at bottom center
fn spawn_bugatti_dashboard(commands: &mut Commands) {
    // Main dashboard container - positioned at bottom center
    let dashboard_root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(50.0),
            width: Val::Px(800.0),
            height: Val::Px(180.0),
            margin: UiRect {
                left: Val::Px(-400.0), // Center horizontally
                ..default()
            },
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)), // Almost opaque black
        BorderColor(Color::srgb(0.2, 0.7, 1.0)), // Bugatti blue glow
        BugattiTelemetryOverlay,
    )).id();

    // Left Panel - Speed and Performance
    let left_panel = commands.spawn((
        Node {
            width: Val::Px(180.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.9)),
        BorderColor(Color::srgb(0.1, 0.5, 0.8)),
    )).id();

    // Speedometer
    commands.spawn((
        Text::new("261\nMPH"),
        TextFont { font_size: 32.0, ..default() },
        TextColor(Color::srgb(0.9, 0.9, 1.0)),
        TextLayout::new_with_justify(JustifyText::Center),
        BugattiSpeedometer,
        ChildOf(left_panel),
    ));

    // Speed label
    commands.spawn((
        Text::new("SPEED"),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.6, 0.8, 1.0)),
        TextLayout::new_with_justify(JustifyText::Center),
        ChildOf(left_panel),
    ));

    // Center Panel - Engine Data
    let center_panel = commands.spawn((
        Node {
            width: Val::Px(240.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.05, 0.05, 0.9)),
        BorderColor(Color::srgb(0.8, 0.3, 0.1)),
    )).id();

    // RPM Display
    commands.spawn((
        Text::new("4200\nRPM"),
        TextFont { font_size: 28.0, ..default() },
        TextColor(Color::srgb(1.0, 0.7, 0.2)),
        TextLayout::new_with_justify(JustifyText::Center),
        BugattiRpmGauge,
        ChildOf(center_panel),
    ));

    // Gear indicator
    commands.spawn((
        Text::new("GEAR: 3"),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        TextLayout::new_with_justify(JustifyText::Center),
        ChildOf(center_panel),
    ));

    // Right Panel - Turbo and Systems
    let right_panel = commands.spawn((
        Node {
            width: Val::Px(180.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.15, 0.05, 0.9)),
        BorderColor(Color::srgb(0.1, 0.8, 0.3)),
    )).id();

    // Turbo Status
    commands.spawn((
        Text::new("QUAD\nTURBO"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::srgb(0.2, 1.0, 0.4)),
        TextLayout::new_with_justify(JustifyText::Center),
        BugattiTurboIndicator,
        ChildOf(right_panel),
    ));

    // Turbo pressure
    commands.spawn((
        Text::new("4/4 ACTIVE"),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.6, 0.9, 0.7)),
        TextLayout::new_with_justify(JustifyText::Center),
        ChildOf(right_panel),
    ));

    // Bottom Info Panel
    let info_panel = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(-60.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        BorderColor(Color::srgb(0.3, 0.3, 0.6)),
        ChildOf(dashboard_root),
    )).id();

    // Info text sections
    commands.spawn((
        Text::new("SPORT • LAUNCH: READY • G: 0.0G"),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 1.0)),
        BugattiInfoPanel,
        ChildOf(info_panel),
    ));

    // Add children to root
    commands.entity(dashboard_root).add_children(&[left_panel, center_panel, right_panel]);
}

/// System to update the Bugatti dashboard with real-time data
pub fn update_bugatti_telemetry_system(
    time: Res<Time>,
    mut telemetry_state: ResMut<BugattiTelemetryState>,
    mut speedometer_query: Query<&mut Text, (With<BugattiSpeedometer>, Without<BugattiRpmGauge>, Without<BugattiTurboIndicator>, Without<BugattiInfoPanel>)>,
    mut rpm_query: Query<&mut Text, (With<BugattiRpmGauge>, Without<BugattiSpeedometer>, Without<BugattiTurboIndicator>, Without<BugattiInfoPanel>)>,
    mut turbo_query: Query<&mut Text, (With<BugattiTurboIndicator>, Without<BugattiSpeedometer>, Without<BugattiRpmGauge>, Without<BugattiInfoPanel>)>,
    mut info_query: Query<&mut Text, (With<BugattiInfoPanel>, Without<BugattiSpeedometer>, Without<BugattiRpmGauge>, Without<BugattiTurboIndicator>)>,
    supercar_query: Query<&SuperCar, (With<Car>, With<ActiveEntity>)>,
    current_state: Res<State<GameState>>,
) {
    if let Ok(supercar) = supercar_query.single() {
        // Update at specified interval for smooth dashboard
        telemetry_state.last_update += time.delta_secs();
        if telemetry_state.last_update >= telemetry_state.update_interval {
            telemetry_state.last_update = 0.0;
            
            // Only show telemetry when actually driving
            if *current_state.get() == GameState::Driving {
                // Update Speedometer
                if let Ok(mut speed_text) = speedometer_query.single_mut() {
                    let current_speed = (supercar.rpm / supercar.max_rpm) * supercar.max_speed;
                    speed_text.0 = format!("{:.0}\nMPH", current_speed.min(supercar.max_speed));
                }
                
                // Update RPM Gauge
                if let Ok(mut rpm_text) = rpm_query.single_mut() {
                    rpm_text.0 = format!("{:.0}\nRPM", supercar.rpm);
                }
                
                // Update Turbo Status
                if let Ok(mut turbo_text) = turbo_query.single_mut() {
                    let turbo_status = match supercar.turbo_stage {
                        0 => "TURBO\nOFF",
                        1 => "TURBO\n1/4",
                        2 => "TURBO\n2/4", 
                        3 => "TURBO\n3/4",
                        4 => "QUAD\nTURBO",
                        _ => "TURBO\nMAX",
                    };
                    turbo_text.0 = turbo_status.to_string();
                }
                
                // Update Info Panel
                if let Ok(mut info_text) = info_query.single_mut() {
                    let mode = match supercar.driving_mode {
                        crate::components::DrivingMode::Comfort => "COMFORT",
                        crate::components::DrivingMode::Sport => "SPORT",
                        crate::components::DrivingMode::Track => "TRACK",
                        crate::components::DrivingMode::Custom => "CUSTOM",
                    };
                    
                    let launch = if supercar.launch_control_engaged {
                        "LAUNCHING"
                    } else if supercar.launch_control {
                        "READY"
                    } else {
                        "OFF"
                    };
                    
                    let total_g = (supercar.g_force_lateral.powi(2) + supercar.g_force_longitudinal.powi(2)).sqrt();
                    
                    info_text.0 = format!("{} • LAUNCH: {} • G: {:.1}G", mode, launch, total_g);
                }
            }
        }
    }
}
