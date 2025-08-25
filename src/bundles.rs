use crate::components::{
    Building, Car, DynamicContent, MovementController, NPCAppearance, NPCBehaviorComponent,
    NPCState, VehicleState, VehicleType,
};
use crate::services::distance_cache::MovementTracker;
use crate::systems::world::unified_world::UnifiedChunkEntity;
use bevy::prelude::*;
use bevy::render::view::VisibilityRange;
use bevy_rapier3d::prelude::*;

/// Bundle for entities that need to be visible and inherit visibility from parents
#[derive(Bundle)]
pub struct VisibleBundle {
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl Default for VisibleBundle {
    fn default() -> Self {
        Self {
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Bundle for child entities that inherit visibility from parents
#[derive(Bundle)]
pub struct VisibleChildBundle {
    pub inherited_visibility: InheritedVisibility,
}

impl Default for VisibleChildBundle {
    fn default() -> Self {
        Self {
            inherited_visibility: InheritedVisibility::VISIBLE,
        }
    }
}

/// Bundle for vehicle parent entities
#[derive(Bundle)]
pub struct VehicleVisibilityBundle {
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl Default for VehicleVisibilityBundle {
    fn default() -> Self {
        Self {
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Complete vehicle bundle with physics and state
#[derive(Bundle)]
pub struct VehicleBundle {
    pub vehicle_type: VehicleType,
    pub vehicle_state: VehicleState,
    pub transform: Transform,
    pub visibility: Visibility,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub additional_mass: AdditionalMassProperties,
    pub velocity: Velocity,
    pub damping: Damping,
    pub visibility_range: VisibilityRange,
}

/// Complete NPC bundle with physics and state  
#[derive(Bundle)]
pub struct NPCBundle {
    pub npc_marker: NPCState,
    pub npc_behavior: NPCBehaviorComponent,
    pub npc_appearance: NPCAppearance,
    pub movement_controller: MovementController,
    pub transform: Transform,
    pub visibility: Visibility,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub additional_mass: AdditionalMassProperties,
    pub velocity: Velocity,
    pub visibility_range: VisibilityRange,
    pub movement_tracker: MovementTracker,
}

/// Complete building bundle with physics and state
#[derive(Bundle)]
pub struct BuildingBundle {
    pub building_marker: Building,
    pub transform: Transform,
    pub visibility: Visibility,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub visibility_range: VisibilityRange,
}

/// Generic physics bundle for any physics object
#[derive(Bundle)]
pub struct PhysicsBundle {
    pub transform: Transform,
    pub visibility: Visibility,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub additional_mass: AdditionalMassProperties,
    pub velocity: Velocity,
    pub damping: Damping,
    pub friction: Friction,
    pub restitution: Restitution,
}

/// Bundle for dynamic content entities in the world
#[derive(Bundle)]
pub struct DynamicContentBundle {
    pub dynamic_content: DynamicContent,
    pub transform: Transform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub visibility_range: VisibilityRange,
}

/// Bundle for dynamic content with physics
#[derive(Bundle)]
pub struct DynamicPhysicsBundle {
    pub dynamic_content: DynamicContent,
    pub transform: Transform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub velocity: Velocity,
    pub visibility_range: VisibilityRange,
}

/// Bundle for vehicle entities with complete setup
#[derive(Bundle)]
pub struct DynamicVehicleBundle {
    pub dynamic_content: DynamicContent,
    pub car: Car,
    pub transform: Transform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub velocity: Velocity,
    pub damping: Damping,
    pub locked_axes: LockedAxes,
    pub visibility_range: VisibilityRange,
}

/// Bundle for trees and vegetation
#[derive(Bundle)]
pub struct VegetationBundle {
    pub dynamic_content: DynamicContent,
    pub transform: Transform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub visibility_range: VisibilityRange,
}

/// Bundle for simple static physics objects
#[derive(Bundle)]
pub struct StaticPhysicsBundle {
    pub transform: Transform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
}

/// Bundle for unified chunk entities
#[derive(Bundle)]
pub struct UnifiedChunkBundle {
    pub chunk_entity: UnifiedChunkEntity,
    pub dynamic_content: DynamicContent,
    pub transform: Transform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub visibility_range: VisibilityRange,
}
