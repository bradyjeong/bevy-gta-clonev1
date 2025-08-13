use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::bundles::{VehicleVisibilityBundle, VisibleChildBundle};
use crate::factories::MaterialFactory;
use crate::systems::world::map_system::BuildingType;

/// Unified entity factory that eliminates duplicate entity spawning patterns
/// CRITICAL: This replaces 60+ duplicate entity creation patterns across the codebase
pub struct EntityFactory;

impl EntityFactory {
    /// Create a vehicle entity with standardized components
    /// SAFETY: This creates entities with identical patterns to existing code
    pub fn spawn_vehicle(
        commands: &mut Commands,
        position: Vec3,
        vehicle_type: VehicleType,
        color: Color,
    ) -> Entity {
        let mut vehicle_commands = commands.spawn((
            Transform::from_translation(position),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Velocity::default(),
            CollisionGroups::new(
                crate::constants::VEHICLE_GROUP,
                crate::constants::STATIC_GROUP | crate::constants::VEHICLE_GROUP | crate::constants::CHARACTER_GROUP
            ),
            Damping { linear_damping: 1.0, angular_damping: 5.0 },
            VehicleVisibilityBundle::default(),
            ActiveEntity,
            VehicleState::new(vehicle_type),
        ));

        // Add specific vehicle component based on type
        match vehicle_type {
            VehicleType::SuperCar => {
                vehicle_commands.insert(SuperCar {
                    max_speed: 120.0,
                    acceleration: 40.0,
                    turbo_boost: false,
                    exhaust_timer: 0.0,
                });
            },
            VehicleType::BasicCar => {
                vehicle_commands.insert(Car);
            },
            VehicleType::Helicopter => {
                vehicle_commands.insert(Helicopter);
            },
            VehicleType::F16 => {
                vehicle_commands.insert(F16);
            },
        }

        let vehicle_entity = vehicle_commands.id();

        // Add appropriate physics collider
        let collider = match vehicle_type {
            VehicleType::SuperCar => Collider::cuboid(0.9, 0.5, 2.1),
            VehicleType::BasicCar => Collider::cuboid(0.9, 0.6, 1.8),
            VehicleType::Helicopter => Collider::cuboid(1.25, 0.75, 2.5),
            VehicleType::F16 => Collider::cuboid(8.0, 1.0, 1.5),
        };
        
        commands.entity(vehicle_entity).insert(collider);

        vehicle_entity
    }

    /// Create a visual child entity with standardized components
    /// SAFETY: This creates the exact component bundle pattern used throughout the codebase
    pub fn spawn_visual_child(
        commands: &mut Commands,
        parent: Entity,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
    ) -> Entity {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            ChildOf(parent),
            VisibleChildBundle::default(),
        )).id()
    }

    /// Create a visual child entity with a name (for debugging)
    pub fn spawn_named_visual_child(
        commands: &mut Commands,
        parent: Entity,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
        name: &str,
    ) -> Entity {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            ChildOf(parent),
            VisibleChildBundle::default(),
            Name::new(name.to_string()),
        )).id()
    }

    /// Create an NPC entity with standardized components
    pub fn spawn_npc(
        commands: &mut Commands,
        position: Vec3,
        npc_type: NPCType,
    ) -> Entity {
        commands.spawn((
            Transform::from_translation(position),
            RigidBody::Dynamic,
            Collider::capsule_y(0.8, 0.4),
            Velocity::default(),
            LockedAxes::ROTATION_LOCKED,
            NPC {
                target_position: Vec3::ZERO,
                speed: 1.5,
                last_update: 0.0,
                update_interval: 1.0,
            },
            ActiveEntity,
            Cullable { max_distance: 300.0, is_culled: false },
            DynamicContent {
                content_type: ContentType::NPC,
            },
        )).id()
    }

    /// Create a building entity with standardized components
    pub fn spawn_building(
        commands: &mut Commands,
        position: Vec3,
        size: Vec3,
        building_type: crate::components::world::BuildingType,
        color: Color,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Entity {
        let building_entity = commands.spawn((
            Transform::from_translation(position),
            RigidBody::Fixed,
            Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
            CollisionGroups::new(
                crate::constants::STATIC_GROUP,
                crate::constants::VEHICLE_GROUP | crate::constants::CHARACTER_GROUP
            ),
            Building {
                building_type,
                height: size.y,
                scale: size,
            },
            Cullable { max_distance: 600.0, is_culled: false },
            DynamicContent { content_type: ContentType::Building },
        )).id();

        // Add visual representation
        Self::spawn_visual_child(
            commands,
            building_entity,
            meshes.add(Cuboid::new(size.x, size.y, size.z)),
            MaterialFactory::create_building_material(materials, color),
            Transform::default(),
        );

        building_entity
    }

    /// Create a road entity with standardized components
    pub fn spawn_road_entity(
        commands: &mut Commands,
        position: Vec3,
        size: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Entity {
        let road_entity = commands.spawn((
            Transform::from_translation(position),
            RigidBody::Fixed,
            Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
            CollisionGroups::new(
                crate::constants::STATIC_GROUP,
                crate::constants::VEHICLE_GROUP | crate::constants::CHARACTER_GROUP
            ),
            RoadEntity { road_id: 0 },
            Name::new("Road Segment"),
        )).id();

        // Add visual representation
        Self::spawn_visual_child(
            commands,
            road_entity,
            meshes.add(Cuboid::new(size.x, size.y, size.z)),
            MaterialFactory::create_water_bottom_material(materials, Color::srgb(0.3, 0.3, 0.3)),
            Transform::default(),
        );

        road_entity
    }
}

/// Standardized entity bundle patterns
/// These replace repeated component combinations across the codebase
#[derive(Bundle)]
pub struct StandardVehicleBundle {
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub velocity: Velocity,
    pub collision_groups: CollisionGroups,
    pub damping: Damping,
    pub visibility_bundle: VehicleVisibilityBundle,
    pub active_entity: ActiveEntity,
}

impl Default for StandardVehicleBundle {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::default(),
            collision_groups: CollisionGroups::new(
                crate::constants::VEHICLE_GROUP,
                crate::constants::STATIC_GROUP | crate::constants::VEHICLE_GROUP | crate::constants::CHARACTER_GROUP
            ),
            damping: Damping { linear_damping: 1.0, angular_damping: 5.0 },
            visibility_bundle: VehicleVisibilityBundle::default(),
            active_entity: ActiveEntity,
        }
    }
}

#[derive(Bundle)]
pub struct VisualChildBundle {
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub child_of: ChildOf,
    pub visible_child: VisibleChildBundle,
}

impl VisualChildBundle {
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
        parent: Entity,
    ) -> Self {
        Self {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(material),
            transform,
            child_of: ChildOf(parent),
            visible_child: VisibleChildBundle::default(),
        }
    }
}
