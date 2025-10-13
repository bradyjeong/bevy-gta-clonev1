use bevy::prelude::*;
use crate::components::{VehicleType, PlayerControlled, ControlState, VehicleControlType};
use crate::bundles::PlayerPhysicsBundle;

/// Events for vehicle entry/exit - replaces giant match blocks
#[derive(Event, Debug)]
pub struct EnterVehicleEvent {
    pub player: Entity,
    pub vehicle: Entity,
    pub vehicle_type: VehicleType,
}

#[derive(Event, Debug)]
pub struct ExitVehicleEvent {
    pub player: Entity,
    pub vehicle: Entity,
    pub vehicle_type: VehicleType,
}

/// Generic vehicle transfer helper - eliminates duplication across Car/Helicopter/F16
pub fn transfer_player_to_vehicle(
    commands: &mut Commands,
    player: Entity,
    vehicle: Entity,
    vehicle_type: VehicleType,
    control_state: Option<&ControlState>,
    has_player_controlled: bool,
) {
    // Remove control from player and hide them
    commands.entity(player)
        .remove::<crate::components::ActiveEntity>()
        .remove::<PlayerControlled>()
        .remove::<ControlState>()
        .insert(Visibility::Hidden)
        .insert(crate::components::ChildOf(vehicle));

    // Add control to vehicle
    let mut vehicle_commands = commands.entity(vehicle);
    vehicle_commands.insert(crate::components::ActiveEntity);
    
    if let Some(control_state) = control_state {
        vehicle_commands.insert(control_state.clone());
    } else {
        vehicle_commands.insert(ControlState::default());
    }
    
    if has_player_controlled {
        vehicle_commands.insert(PlayerControlled);
    }
    
    // Set appropriate vehicle control type
    let control_type = match vehicle_type {
        VehicleType::Car => VehicleControlType::Car,
        VehicleType::Helicopter => VehicleControlType::Helicopter,
        VehicleType::F16 => VehicleControlType::F16,
        VehicleType::Yacht => VehicleControlType::Yacht,
    };
    vehicle_commands.insert(control_type);
    
    // Store vehicle relationship
    commands.entity(player).insert(crate::components::InCar(vehicle));
}

/// Enhanced vehicle exit helper with physics restoration
pub fn transfer_player_from_vehicle_with_physics(
    commands: &mut Commands,
    player: Entity,
    vehicle: Entity,
    exit_position: Vec3,
    exit_rotation: Quat,
) {
    // Remove control from vehicle
    commands.entity(vehicle)
        .remove::<crate::components::ActiveEntity>()
        .remove::<PlayerControlled>()
        .remove::<ControlState>()
        .remove::<VehicleControlType>();

    // Restore player control, physics, and visibility
    commands.entity(player)
        .remove::<crate::components::InCar>()
        .remove::<crate::components::ChildOf>()
        .insert(crate::components::ActiveEntity)
        .insert(PlayerControlled)
        .insert(ControlState::default())
        .insert(VehicleControlType::Walking)
        .insert(Visibility::Visible)
        .insert(Transform::from_translation(exit_position).with_rotation(exit_rotation))
        .insert(PlayerPhysicsBundle::default());
}

/// Legacy helper for backward compatibility
pub fn transfer_player_from_vehicle(
    commands: &mut Commands,
    player: Entity,
    vehicle: Entity,
    exit_offset: Vec3,
    vehicle_transform: &Transform,
) {
    let exit_position = vehicle_transform.translation + exit_offset;
    // Preserve vehicle's Y rotation
    let exit_rotation = Quat::from_rotation_y(vehicle_transform.rotation.to_euler(EulerRot::YXZ).0);
    transfer_player_from_vehicle_with_physics(
        commands,
        player,
        vehicle,
        exit_position,
        exit_rotation,
    );
}
