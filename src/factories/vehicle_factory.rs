use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::components::realistic_vehicle::{RealisticVehicle, RealisticVehicleType};
use crate::factories::MeshFactory;

/// Factory for creating vehicle entities with simplified, clear interface
pub struct VehicleFactory;

impl VehicleFactory {
    /// Spawn a single vehicle entity
    pub fn spawn(
        commands: &mut Commands,
        _mesh_factory: &MeshFactory,
        position: Vec3,
        vehicle_type: RealisticVehicleType,
    ) -> Entity {
        let entity = commands.spawn((
            Name::new("Vehicle"),
            Transform::from_translation(position),
            Visibility::default(),
        )).id();

        // Add vehicle-specific components
        commands.entity(entity).insert((
            Car,
            VehicleState::new(vehicles::VehicleType::BasicCar),
            RealisticVehicle {
                physics_enabled: true,
                vehicle_type,
                ..Default::default()
            },
            // Basic physics
            RigidBody::Dynamic,
            Collider::cuboid(1.0, 0.5, 2.0),
            CollisionGroups::new(Group::GROUP_1, Group::ALL),
            Velocity::default(),
        ));

        entity
    }
}
