use bevy::prelude::*;
use crate::events::cross_plugin_events::*;
use crate::game_state::GameState;
use crate::components::vehicles::*;
use crate::components::control_state::ControlState;
use crate::systems::safety::{world_bounds_safety_system, position_monitor_system};
use crate::systems::vehicles::vehicle_lod_system;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register vehicle-related events
            .add_event::<RequestVehicleMovement>()
            .add_event::<RequestExhaustEffect>()
            .add_event::<RequestJetFlameUpdate>()
            
            .add_systems(Update, (
                // Safety systems (remain internal to vehicle plugin)
                world_bounds_safety_system,
                position_monitor_system,
                
                // LOD system (internal to vehicle plugin)
                vehicle_lod_system,
                
                // Send movement requests based on control state
                send_vehicle_movement_requests.run_if(in_state(GameState::Driving)),
                send_helicopter_movement_requests.run_if(in_state(GameState::Flying)),
                send_f16_movement_requests.run_if(in_state(GameState::Jetting)),
                
                // Send effect requests
                send_exhaust_effect_requests,
                send_jet_flame_requests.run_if(in_state(GameState::Jetting)),
                
                // Internal vehicle systems
                rotate_helicopter_rotors_internal,
            ));
    }
}

/// Send car movement requests to movement plugin
fn send_vehicle_movement_requests(
    mut events: EventWriter<RequestVehicleMovement>,
    query: Query<(Entity, &ControlState), With<Car>>,
) {
    for (entity, control) in query.iter() {
        events.send(RequestVehicleMovement {
            entity,
            vehicle_type: VehicleType::Car,
            throttle: control.throttle,
            steering: control.steering,
            brake: control.brake,
            special: control.turbo,
        });
    }
}

/// Send helicopter movement requests
fn send_helicopter_movement_requests(
    mut events: EventWriter<RequestVehicleMovement>,
    query: Query<(Entity, &ControlState), With<Helicopter>>,
) {
    for (entity, control) in query.iter() {
        events.send(RequestVehicleMovement {
            entity,
            vehicle_type: VehicleType::Helicopter,
            throttle: control.throttle,
            steering: control.yaw,
            brake: control.pitch,
            special: false,
        });
    }
}

/// Send F16 movement requests
fn send_f16_movement_requests(
    mut events: EventWriter<RequestVehicleMovement>,
    query: Query<(Entity, &ControlState), With<F16>>,
) {
    for (entity, control) in query.iter() {
        events.send(RequestVehicleMovement {
            entity,
            vehicle_type: VehicleType::F16,
            throttle: control.throttle,
            steering: control.pitch,
            brake: control.roll,
            special: control.afterburner,
        });
    }
}

/// Send exhaust effect requests to effects plugin
fn send_exhaust_effect_requests(
    mut events: EventWriter<RequestExhaustEffect>,
    query: Query<(Entity, &Transform, &ControlState), Or<(With<Car>, With<Helicopter>)>>,
) {
    for (entity, transform, control) in query.iter() {
        if control.throttle.abs() > 0.1 {
            events.send(RequestExhaustEffect {
                entity,
                intensity: control.throttle.abs(),
                position: transform.translation - transform.forward().as_vec3() * 2.0,
                direction: -transform.forward().as_vec3(),
            });
        }
    }
}

/// Send jet flame update requests
fn send_jet_flame_requests(
    mut events: EventWriter<RequestJetFlameUpdate>,
    query: Query<(Entity, &ControlState), With<F16>>,
) {
    for (entity, control) in query.iter() {
        events.send(RequestJetFlameUpdate {
            entity,
            throttle: control.throttle,
            afterburner: control.afterburner,
        });
    }
}

/// Internal system for rotating helicopter rotors (doesn't cross plugin boundary)
fn rotate_helicopter_rotors_internal(
    mut query: Query<(&mut Transform, &HelicopterRotor)>,
    time: Res<Time>,
) {
    for (mut transform, rotor) in query.iter_mut() {
        let rotation_speed = rotor.speed * time.delta_secs();
        transform.rotate_local_y(rotation_speed);
    }
}

// Vehicle marker components
#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct Helicopter;

#[derive(Component)]
pub struct F16;

#[derive(Component)]
pub struct HelicopterRotor {
    pub speed: f32,
}
