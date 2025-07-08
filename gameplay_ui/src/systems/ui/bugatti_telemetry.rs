//! ───────────────────────────────────────────────
//! System:   Bugatti Telemetry
//! Purpose:  Processes user input and control mapping
//! Schedule: Update
//! Reads:    ActiveEntity, BugattiRpmGauge, Car, BugattiTelemetryState, BugattiSpeedometer
//! Writes:   Visibility, BugattiTelemetryState, Text
//! Invariants:
//!   * Only active entities can be controlled
//! Owner:    @ui-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;

/// Component marker for the Bugatti telemetry dashboard
#[derive(Component)]
pub struct BugattiTelemetryOverlay;

/// Component marker for speedometer
#[derive(Component)]
pub struct BugattiSpeedometer;

/// Component marker for RPM gauge
#[derive(Component)]
pub struct BugattiRpmGauge;

/// Component marker for turbo indicator
#[derive(Component)]
pub struct BugattiTurboIndicator;

/// Component marker for info panel
#[derive(Component)]
pub struct BugattiInfoPanel;

/// Resource to track dashboard visibility and state
#[derive(Resource, Default)]
pub struct BugattiTelemetryState {
    /// Visibility state
    pub visible: bool,
    /// Last update time
    pub last_update: f32,
    /// Update interval
    pub update_interval: f32,
}

impl BugattiTelemetryState {
    /// Create new telemetry state
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
        if supercar_query.get_single().is_ok() {
            telemetry_state.visible = !telemetry_state.visible;
            
            // Toggle existing overlay visibility or create new one
            if let Ok(mut visibility) = overlay_query.get_single_mut() {
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
        Node {
            align_self: AlignSelf::Center,
            ..default()
        },
        BugattiSpeedometer,
    )).set_parent(left_panel);
    
    commands.entity(left_panel).set_parent(dashboard_root);
    
    // Center Panel - RPM and Turbo
    let center_panel = commands.spawn((
        Node {
            width: Val::Px(200.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceAround,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.05, 0.05, 0.9)),
        BorderColor(Color::srgb(0.8, 0.2, 0.2)),
    )).id();
    
    // RPM Gauge
    commands.spawn((
        Text::new("8500\nRPM"),
        TextFont { font_size: 28.0, ..default() },
        TextColor(Color::srgb(1.0, 0.3, 0.3)),
        Node {
            align_self: AlignSelf::Center,
            ..default()
        },
        BugattiRpmGauge,
    )).set_parent(center_panel);
    
    // Turbo Indicator
    commands.spawn((
        Text::new("TURBO\n2.5 BAR"),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(0.3, 1.0, 0.3)),
        Node {
            align_self: AlignSelf::Center,
            ..default()
        },
        BugattiTurboIndicator,
    )).set_parent(center_panel);
    
    commands.entity(center_panel).set_parent(dashboard_root);
    
    // Right Panel - Info and Statistics
    let right_panel = commands.spawn((
        Node {
            width: Val::Px(300.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceEvenly,
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 0.9)),
        BorderColor(Color::srgb(0.2, 0.8, 0.2)),
    )).id();
    
    // Info Panel
    commands.spawn((
        Text::new("SPORT • LAUNCH: READY • G: 1.2G"),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 1.0)),
        Node {
            align_self: AlignSelf::Center,
            ..default()
        },
        BugattiInfoPanel,
    )).set_parent(right_panel);
    
    commands.entity(right_panel).set_parent(dashboard_root);
}

/// System to update the Bugatti telemetry display with real-time data
pub fn update_bugatti_telemetry_system(
    telemetry_state: Res<BugattiTelemetryState>,
    time: Res<Time>,
    supercar_query: Query<(&SuperCar, &VehicleState, &Transform), (With<Car>, With<ActiveEntity>)>,
    mut speedometer_query: Query<&mut Text, (With<BugattiSpeedometer>, Without<BugattiRpmGauge>, Without<BugattiInfoPanel>)>,
    mut rpm_query: Query<&mut Text, (With<BugattiRpmGauge>, Without<BugattiSpeedometer>, Without<BugattiInfoPanel>)>,
    mut info_query: Query<&mut Text, (With<BugattiInfoPanel>, Without<BugattiSpeedometer>, Without<BugattiRpmGauge>)>,
) {
    // Only update if telemetry is visible
    if !telemetry_state.visible {
        return;
    }
    
    // Get SuperCar data
    if let Ok((supercar, vehicle_state, transform)) = supercar_query.get_single() {
        // Update speedometer
        if let Ok(mut speed_text) = speedometer_query.get_single_mut() {
            let speed_mph = vehicle_state.speed * 2.237; // Convert m/s to mph
            speed_text.0 = format!("{:.0}\nMPH", speed_mph);
        }
        
        // Update RPM gauge
        if let Ok(mut rpm_text) = rpm_query.get_single_mut() {
            let rpm = supercar.rpm.min(8500.0); // Redline at 8500 RPM
            rpm_text.0 = format!("{:.0}\nRPM", rpm);
        }
        
        // Update info panel
        if let Ok(mut info_text) = info_query.get_single_mut() {
            let mode = if supercar.launch_control_active { "LAUNCH" } else { "SPORT" };
            let launch = if supercar.launch_control_ready { "READY" } else { "WAIT" };
            let total_g = (vehicle_state.acceleration.length() / 9.81).abs();
            info_text.0 = format!("{} • LAUNCH: {} • G: {:.1}G", mode, launch, total_g);
        }
    }
}
