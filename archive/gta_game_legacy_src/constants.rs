use bevy_rapier3d::prelude::*;

// Collision groups for proper physics separation
pub const STATIC_GROUP: Group = Group::GROUP_1;    // Buildings, terrain, trees
pub const VEHICLE_GROUP: Group = Group::GROUP_2;   // Cars, helicopters, jets
pub const CHARACTER_GROUP: Group = Group::GROUP_3; // Player, NPCs
