//! Shared physics and collision setup utilities
//! 
//! Stateless helper functions for consistent physics configuration

use bevy_rapier3d::prelude::*;
use crate::GameConfig;

/// Common physics configuration utilities
pub struct PhysicsSetup;

impl PhysicsSetup {
    /// Create standard collision groups for entity type
    pub fn collision_groups_for_entity(entity_type: EntityPhysicsType, config: &GameConfig) -> CollisionGroups {
        match entity_type {
            EntityPhysicsType::StaticBuilding => CollisionGroups::new(
                config.physics.static_group,
                Group::ALL
            ),
            EntityPhysicsType::DynamicVehicle => CollisionGroups::new(
                config.physics.vehicle_group,
                config.physics.static_group | config.physics.vehicle_group | config.physics.character_group
            ),
            EntityPhysicsType::Character => CollisionGroups::new(
                config.physics.character_group,
                Group::ALL
            ),
            EntityPhysicsType::StaticVegetation => CollisionGroups::new(
                config.physics.static_group,
                config.physics.vehicle_group | config.physics.character_group
            ),
        }
    }
    
    /// Create standard damping for entity type
    pub fn damping_for_entity(entity_type: EntityPhysicsType) -> Damping {
        match entity_type {
            EntityPhysicsType::DynamicVehicle => Damping {
                linear_damping: 1.0,
                angular_damping: 5.0,
            },
            EntityPhysicsType::Character => Damping {
                linear_damping: 2.0,
                angular_damping: 10.0,
            },
            _ => Damping::default(),
        }
    }
    
    /// Create standard locked axes for entity type
    pub fn locked_axes_for_entity(entity_type: EntityPhysicsType) -> Option<LockedAxes> {
        match entity_type {
            EntityPhysicsType::DynamicVehicle => Some(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
            EntityPhysicsType::Character => Some(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
            _ => None,
        }
    }
}

/// Entity physics type for consistent configuration
#[derive(Clone, Copy, Debug)]
pub enum EntityPhysicsType {
    StaticBuilding,
    DynamicVehicle,
    Character,
    StaticVegetation,
}
