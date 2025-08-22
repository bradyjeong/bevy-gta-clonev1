use crate::components::{AircraftFlight, F16, VehicleState};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

/// Ultra-simplified F-16 persistence - minimal data only
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableAircraftFlight {
    pub throttle: f32,
    pub airspeed: f32,
    pub afterburner_active: bool,
}

impl From<&AircraftFlight> for SerializableAircraftFlight {
    fn from(flight: &AircraftFlight) -> Self {
        Self {
            throttle: flight.throttle,
            airspeed: flight.airspeed,
            afterburner_active: flight.afterburner_active,
        }
    }
}

impl From<SerializableAircraftFlight> for AircraftFlight {
    fn from(data: SerializableAircraftFlight) -> Self {
        Self {
            throttle: data.throttle,
            airspeed: data.airspeed,
            afterburner_active: data.afterburner_active,
        }
    }
}

/// Simplified F-16 save data - only what we actually need
#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableF16 {
    pub entity_id: u64,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub velocity: (f32, f32, f32),
    pub angular_velocity: (f32, f32, f32),
    pub flight_data: SerializableAircraftFlight,
    pub vehicle_state: SerializableAircraftVehicleState,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableAircraftVehicleState {
    pub damage: f32,
    pub fuel: f32,
    pub max_speed: f32,
    pub acceleration: f32,
}

impl From<&VehicleState> for SerializableAircraftVehicleState {
    fn from(state: &VehicleState) -> Self {
        Self {
            damage: state.damage,
            fuel: state.fuel,
            max_speed: state.max_speed,
            acceleration: state.acceleration,
        }
    }
}

/// Collect F-16 entities for saving
pub fn collect_f16_data(
    f16_query: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &VehicleState,
            &AircraftFlight,
        ),
        With<F16>,
    >,
) -> Vec<SerializableF16> {
    f16_query
        .iter()
        .map(
            |(entity, transform, velocity, vehicle_state, aircraft_flight)| SerializableF16 {
                entity_id: entity.index() as u64,
                position: (
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                ),
                rotation: (
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z,
                    transform.rotation.w,
                ),
                velocity: (velocity.linvel.x, velocity.linvel.y, velocity.linvel.z),
                angular_velocity: (velocity.angvel.x, velocity.angvel.y, velocity.angvel.z),
                flight_data: aircraft_flight.into(),
                vehicle_state: vehicle_state.into(),
            },
        )
        .collect()
}
