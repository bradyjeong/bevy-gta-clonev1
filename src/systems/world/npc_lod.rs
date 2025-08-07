use bevy::prelude::*;
use crate::components::{
    NPCState, NPCRendering, NPCLOD, NPCAppearance, ActiveEntity,
    NPCHead, NPCTorso, NPCLeftArm, NPCRightArm, NPCLeftLeg, NPCRightLeg, NPCBodyPart,
    NPC_LOD_FULL_DISTANCE, NPC_LOD_MEDIUM_DISTANCE, NPC_LOD_LOW_DISTANCE
};
use crate::factories::{RenderingFactory, StandardRenderingPattern};
use crate::services::timing_service::{TimingService, SystemType, EntityTimerType, ManagedTiming};
use crate::services::distance_cache::{DistanceCache, get_cached_distance};

/// NPC LOD system that follows the vehicle LOD architecture
/// Manages rendering complexity based on distance from active entity
pub fn npc_lod_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut timing_service: ResMut<TimingService>,
    active_query: Query<(Entity, &Transform), With<ActiveEntity>>,
    mut npc_query: Query<(Entity, &mut NPCState, Option<&NPCRendering>, &Transform, Option<&ManagedTiming>)>,
    mut distance_cache: ResMut<DistanceCache>,
) {
    // Use unified timing service instead of manual timing checks
    if !timing_service.should_run_system(SystemType::NPCLOD) {
        return;
    }
    
    let Ok((active_entity, active_transform)) = active_query.single() else { return };
    let player_pos = active_transform.translation;
    
    for (entity, mut npc_state, rendering, transform, managed_timing) in npc_query.iter_mut() {
        // Register entity for timing management if not already managed
        if managed_timing.is_none() {
            timing_service.register_entity(entity, EntityTimerType::NPCLOD, 0.1);
            commands.entity(entity).insert(ManagedTiming::new(EntityTimerType::NPCLOD));
        }
        
        // Check if this specific entity should update (individual entity timing)
        if !timing_service.should_update_entity(entity) {
            continue;
        }
        
        let distance = get_cached_distance(
            active_entity,
            entity,
            player_pos,
            transform.translation,
            &mut distance_cache,
        );
        let new_lod = determine_npc_lod(distance);
        
        if new_lod != npc_state.current_lod {
            // LOD changed - update rendering
            update_npc_lod(
                entity,
                &mut npc_state,
                rendering,
                new_lod,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        }
    }
}

/// Determine appropriate LOD level based on distance
fn determine_npc_lod(distance: f32) -> NPCLOD {
    if distance <= NPC_LOD_FULL_DISTANCE {
        NPCLOD::Full
    } else if distance <= NPC_LOD_MEDIUM_DISTANCE {
        NPCLOD::Medium
    } else if distance <= NPC_LOD_LOW_DISTANCE {
        NPCLOD::Low
    } else {
        NPCLOD::StateOnly
    }
}

/// Update NPC rendering based on new LOD level
fn update_npc_lod(
    entity: Entity,
    npc_state: &mut NPCState,
    current_rendering: Option<&NPCRendering>,
    new_lod: NPCLOD,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Remove existing rendering if present
    if let Some(rendering) = current_rendering {
        cleanup_npc_rendering(entity, rendering, commands);
    }
    
    npc_state.current_lod = new_lod;
    
    // Add new rendering based on LOD level
    match new_lod {
        NPCLOD::Full => {
            spawn_npc_full_lod(entity, &npc_state.appearance, commands, meshes, materials);
        },
        NPCLOD::Medium => {
            spawn_npc_medium_lod(entity, &npc_state.appearance, commands, meshes, materials);
        },
        NPCLOD::Low => {
            spawn_npc_low_lod(entity, &npc_state.appearance, commands, meshes, materials);
        },
        NPCLOD::StateOnly => {
            // No rendering - just ensure component is removed
            commands.entity(entity).remove::<NPCRendering>();
        },
    }
}

/// Clean up existing NPC rendering entities
fn cleanup_npc_rendering(
    _parent_entity: Entity,
    rendering: &NPCRendering,
    commands: &mut Commands,
) {
    for &child_entity in &rendering.body_entities {
        commands.entity(child_entity).despawn();
    }
}

/// Spawn full detail NPC with all body parts and animations
fn spawn_npc_full_lod(
    parent_entity: Entity,
    appearance: &NPCAppearance,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut body_entities = Vec::new();
    
    // Head
    let head_entity = commands.spawn((
        NPCHead,
        NPCBodyPart {
            rest_position: Vec3::new(0.0, appearance.height * 0.85, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        // FACTORY PATTERN: Head creation using rendering factory
        {
            let (mesh, material) = RenderingFactory::create_mesh_and_material(
                meshes, 
                materials, 
                &StandardRenderingPattern::NPCHead { build_factor: appearance.build }
            );
            (Mesh3d(mesh), MeshMaterial3d(material))
        }.0,
        {
            let (mesh, material) = RenderingFactory::create_mesh_and_material(
                meshes, 
                materials, 
                &StandardRenderingPattern::NPCHead { build_factor: appearance.build }
            );
            (Mesh3d(mesh), MeshMaterial3d(material))
        }.1,
        Transform::from_xyz(0.0, appearance.height * 0.85, 0.0),
    )).id();
    body_entities.push(head_entity);
    
    // Torso
    let torso_entity = commands.spawn((
        NPCTorso,
        NPCBodyPart {
            rest_position: Vec3::new(0.0, appearance.height * 0.5, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        Mesh3d(meshes.add(Cuboid::new(0.4 * appearance.build, 0.6 * appearance.height, 0.2 * appearance.build))),
        MeshMaterial3d(materials.add(appearance.shirt_color)),
        Transform::from_xyz(0.0, appearance.height * 0.5, 0.0),
    )).id();
    body_entities.push(torso_entity);
    
    // Arms
    let arm_length = 0.3 * appearance.height;
    let arm_radius = 0.06 * appearance.build;
    
    let left_arm = commands.spawn((
        NPCLeftArm,
        NPCBodyPart {
            rest_position: Vec3::new(-0.25 * appearance.build, appearance.height * 0.65, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        Mesh3d(meshes.add(Capsule3d::new(arm_radius, arm_length))),
        MeshMaterial3d(materials.add(appearance.skin_tone)),
        Transform::from_xyz(-0.25 * appearance.build, appearance.height * 0.65, 0.0),
    )).id();
    body_entities.push(left_arm);
    
    let right_arm = commands.spawn((
        NPCRightArm,
        NPCBodyPart {
            rest_position: Vec3::new(0.25 * appearance.build, appearance.height * 0.65, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        Mesh3d(meshes.add(Capsule3d::new(arm_radius, arm_length))),
        MeshMaterial3d(materials.add(appearance.skin_tone)),
        Transform::from_xyz(0.25 * appearance.build, appearance.height * 0.65, 0.0),
    )).id();
    body_entities.push(right_arm);
    
    // Legs
    let leg_length = 0.4 * appearance.height;
    let leg_radius = 0.08 * appearance.build;
    
    let left_leg = commands.spawn((
        NPCLeftLeg,
        NPCBodyPart {
            rest_position: Vec3::new(-0.1 * appearance.build, appearance.height * 0.2, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        Mesh3d(meshes.add(Capsule3d::new(leg_radius, leg_length))),
        MeshMaterial3d(materials.add(appearance.pants_color)),
        Transform::from_xyz(-0.1 * appearance.build, appearance.height * 0.2, 0.0),
    )).id();
    body_entities.push(left_leg);
    
    let right_leg = commands.spawn((
        NPCRightLeg,
        NPCBodyPart {
            rest_position: Vec3::new(0.1 * appearance.build, appearance.height * 0.2, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        Mesh3d(meshes.add(Capsule3d::new(leg_radius, leg_length))),
        MeshMaterial3d(materials.add(appearance.pants_color)),
        Transform::from_xyz(0.1 * appearance.build, appearance.height * 0.2, 0.0),
    )).id();
    body_entities.push(right_leg);
    
    // Make all body parts children of the main NPC entity
    for &child in &body_entities {
        commands.entity(parent_entity).add_child(child);
    }
    
    // Add rendering component
    commands.entity(parent_entity).insert(NPCRendering {
        lod_level: NPCLOD::Full,
        body_entities,
    });
}

/// Spawn medium detail NPC with simplified 3-part mesh
fn spawn_npc_medium_lod(
    parent_entity: Entity,
    appearance: &NPCAppearance,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut body_entities = Vec::new();
    
    // Single simplified body mesh
    let body_entity = commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.3 * appearance.build, appearance.height * 0.8))),
        MeshMaterial3d(materials.add(appearance.shirt_color)),
        Transform::from_xyz(0.0, appearance.height * 0.4, 0.0),
    )).id();
    body_entities.push(body_entity);
    
    // Simple head
    let head_entity = commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.1 * appearance.build))),
        MeshMaterial3d(materials.add(appearance.skin_tone)),
        Transform::from_xyz(0.0, appearance.height * 0.85, 0.0),
    )).id();
    body_entities.push(head_entity);
    
    // Make body parts children of the main NPC entity
    for &child in &body_entities {
        commands.entity(parent_entity).add_child(child);
    }
    
    commands.entity(parent_entity).insert(NPCRendering {
        lod_level: NPCLOD::Medium,
        body_entities,
    });
}

/// Spawn low detail NPC with single human silhouette
fn spawn_npc_low_lod(
    parent_entity: Entity,
    appearance: &NPCAppearance,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut body_entities = Vec::new();
    
    // Single silhouette mesh
    let silhouette_entity = commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.25 * appearance.build, appearance.height))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, appearance.height * 0.5, 0.0),
    )).id();
    body_entities.push(silhouette_entity);
    
    commands.entity(parent_entity).add_child(silhouette_entity);
    
    commands.entity(parent_entity).insert(NPCRendering {
        lod_level: NPCLOD::Low,
        body_entities,
    });
}
