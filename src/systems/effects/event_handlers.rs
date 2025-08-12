use bevy::prelude::*;
use crate::events::cross_plugin_events::*;

/// Handle exhaust effect requests from vehicle plugin
pub fn handle_exhaust_effect_request_system(
    mut events: EventReader<RequestExhaustEffect>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effect_tracker: ResMut<EffectTracker>,
) {
    for event in events.read() {
        // Check if effect already exists for this entity
        if let Some(effect_entity) = effect_tracker.exhaust_effects.get(&event.entity) {
            // Update existing effect
            if let Ok(mut transform) = commands.get_entity(*effect_entity).and_then(|e| {
                e.get::<Transform>().ok()
            }) {
                transform.translation = event.position;
                transform.look_at(event.position + event.direction, Vec3::Y);
            }
        } else {
            // Create new exhaust effect
            let effect_entity = commands.spawn((
                Mesh3d(meshes.add(Cone::new(0.2, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.5, 0.5, 0.5, 0.7),
                    alpha_mode: AlphaMode::Blend,
                    emissive: Color::srgb(0.3, 0.3, 0.3).into(),
                    ..default()
                })),
                Transform::from_translation(event.position)
                    .looking_at(event.position + event.direction, Vec3::Y),
                ExhaustEffect {
                    owner: event.entity,
                    intensity: event.intensity,
                },
            )).id();
            
            effect_tracker.exhaust_effects.insert(event.entity, effect_entity);
        }
    }
}

/// Handle jet flame update requests
pub fn handle_jet_flame_request_system(
    mut events: EventReader<RequestJetFlameUpdate>,
    mut query: Query<(&mut Transform, &mut JetFlame)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effect_tracker: ResMut<EffectTracker>,
) {
    for event in events.read() {
        if let Some(flame_entity) = effect_tracker.jet_flames.get(&event.entity) {
            // Update existing flame
            if let Ok((mut transform, mut flame)) = query.get_mut(*flame_entity) {
                flame.intensity = event.throttle;
                flame.afterburner = event.afterburner;
                
                // Scale based on throttle and afterburner
                let scale = if event.afterburner { 2.0 } else { 1.0 } * event.throttle;
                transform.scale = Vec3::splat(scale);
            }
        } else if event.throttle > 0.1 {
            // Create new jet flame
            let flame_entity = commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.5, 3.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: if event.afterburner {
                        Color::srgb(0.0, 0.5, 1.0)
                    } else {
                        Color::srgb(1.0, 0.5, 0.0)
                    },
                    emissive: Color::srgb(1.0, 0.3, 0.0).into(),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_scale(Vec3::splat(event.throttle)),
                JetFlame {
                    owner: event.entity,
                    intensity: event.throttle,
                    afterburner: event.afterburner,
                },
            )).id();
            
            effect_tracker.jet_flames.insert(event.entity, flame_entity);
        }
    }
}

/// Handle waypoint update requests from UI plugin
pub fn handle_waypoint_update_request_system(
    mut events: EventReader<RequestWaypointUpdate>,
    mut query: Query<(&mut Transform, &mut Visibility), With<Waypoint>>,
    mut commands: Commands,
    mut effect_tracker: ResMut<EffectTracker>,
) {
    for event in events.read() {
        if let Some(waypoint_entity) = effect_tracker.waypoints.get(&event.entity) {
            // Update existing waypoint
            if let Ok((mut transform, mut visibility)) = query.get_mut(*waypoint_entity) {
                transform.translation = event.position;
                *visibility = if event.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        } else if event.visible {
            // Create new waypoint marker
            let waypoint_entity = commands.spawn((
                // Waypoint visual representation (could be a billboard, mesh, etc.)
                Transform::from_translation(event.position),
                Visibility::Visible,
                Waypoint {
                    owner: event.entity,
                },
            )).id();
            
            effect_tracker.waypoints.insert(event.entity, waypoint_entity);
        }
    }
}

/// Handle beacon visibility requests from debug plugin
pub fn handle_beacon_visibility_request_system(
    mut events: EventReader<RequestBeaconVisibility>,
    mut query: Query<&mut Visibility, With<DebugBeacon>>,
    effect_tracker: Res<EffectTracker>,
) {
    for event in events.read() {
        if let Some(beacon_entity) = effect_tracker.debug_beacons.get(&event.entity) {
            if let Ok(mut visibility) = query.get_mut(*beacon_entity) {
                *visibility = if event.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// Resource to track effect entities
#[derive(Resource, Default)]
pub struct EffectTracker {
    pub exhaust_effects: bevy::utils::HashMap<Entity, Entity>,
    pub jet_flames: bevy::utils::HashMap<Entity, Entity>,
    pub waypoints: bevy::utils::HashMap<Entity, Entity>,
    pub debug_beacons: bevy::utils::HashMap<Entity, Entity>,
}

// Effect marker components
#[derive(Component)]
pub struct ExhaustEffect {
    pub owner: Entity,
    pub intensity: f32,
}

#[derive(Component)]
pub struct JetFlame {
    pub owner: Entity,
    pub intensity: f32,
    pub afterburner: bool,
}

#[derive(Component)]
pub struct Waypoint {
    pub owner: Entity,
}

#[derive(Component)]
pub struct DebugBeacon {
    pub owner: Entity,
}

/// Cleanup effects when their owners are despawned
pub fn cleanup_orphaned_effects_system(
    mut commands: Commands,
    mut effect_tracker: ResMut<EffectTracker>,
    query: Query<Entity>,
) {
    // Clean up effects whose owners no longer exist
    effect_tracker.exhaust_effects.retain(|owner, effect| {
        if query.get(*owner).is_err() {
            if let Some(entity_commands) = commands.get_entity(*effect) {
                entity_commands.despawn_recursive();
            }
            false
        } else {
            true
        }
    });
    
    effect_tracker.jet_flames.retain(|owner, effect| {
        if query.get(*owner).is_err() {
            if let Some(entity_commands) = commands.get_entity(*effect) {
                entity_commands.despawn_recursive();
            }
            false
        } else {
            true
        }
    });
}
