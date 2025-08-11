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
        // Convert u32 group numbers to Group constants
        let static_group = Self::u32_to_group(config.gameplay.physics.static_group);
        let vehicle_group = Self::u32_to_group(config.gameplay.physics.vehicle_group);
        let character_group = Self::u32_to_group(config.gameplay.physics.character_group);
        
        match entity_type {
            EntityPhysicsType::StaticBuilding => CollisionGroups::new(
                static_group,
                Group::ALL
            ),
            EntityPhysicsType::DynamicVehicle => CollisionGroups::new(
                vehicle_group,
                static_group | vehicle_group | character_group
            ),
            EntityPhysicsType::Character => CollisionGroups::new(
                character_group,
                Group::ALL
            ),
            EntityPhysicsType::StaticVegetation => CollisionGroups::new(
                static_group,
                vehicle_group | character_group
            ),
        }
    }
    
    /// Convert u32 group number to Group constant
    fn u32_to_group(group_num: u32) -> Group {
        match group_num {
            1 => Group::GROUP_1,
            2 => Group::GROUP_2,
            3 => Group::GROUP_3,
            4 => Group::GROUP_4,
            5 => Group::GROUP_5,
            6 => Group::GROUP_6,
            7 => Group::GROUP_7,
            8 => Group::GROUP_8,
            9 => Group::GROUP_9,
            10 => Group::GROUP_10,
            _ => Group::GROUP_1, // Default to GROUP_1
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
