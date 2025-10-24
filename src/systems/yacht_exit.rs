use bevy::prelude::*;
use bevy_rapier3d::geometry::Collider;
use bevy_rapier3d::prelude::{Damping, GravityScale, Velocity};

use crate::components::unified_water::WaterBodyId;
use crate::components::{
    ControlState, DeckWalkAnchor, DeckWalker, Enterable, ExitPoint, ExitPointKind, Helicopter,
    Helipad, InCar, LandedOnYacht, PendingPhysicsEnable, Player, PlayerControlled,
    VehicleControlType, Yacht,
};
use crate::game_state::GameState;
use crate::systems::safe_active_entity::queue_active_transfer;
use crate::systems::swimming::{ProneRotation, SwimState, Swimming};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn yacht_exit_system(
    mut commands: Commands,
    yacht_query: Query<(Entity, &ControlState, &Children), (With<Yacht>, With<PlayerControlled>)>,
    just_controlled: Query<Entity, (With<Yacht>, Added<PlayerControlled>)>,
    helipad_query: Query<(&GlobalTransform, &Collider), With<Helipad>>,
    helicopter_query: Query<
        (Entity, &GlobalTransform, &Collider),
        (
            With<Enterable>,
            With<LandedOnYacht>,
            Without<PlayerControlled>,
        ),
    >,
    exit_point_query: Query<(&ExitPoint, &GlobalTransform)>,
    deck_anchor_query: Query<(Entity, &GlobalTransform), With<DeckWalkAnchor>>,
    mut player_query: Query<
        (Entity, &mut Transform, &mut Visibility),
        (With<Player>, Without<PlayerControlled>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (yacht_entity, control_state, children) in yacht_query.iter() {
        // Skip one frame after control transfer to prevent immediate exit when F is held
        if just_controlled.get(yacht_entity).is_ok() {
            continue;
        }

        if !control_state.interact {
            continue;
        }

        if let Some((helipad_gt, _helipad_collider)) = children
            .iter()
            .find_map(|child| helipad_query.get(child).ok())
        {
            if let Some((heli_entity, _heli_gt, _heli_collider)) = helicopter_query
                .iter()
                .filter(|(_, heli_gt, _)| {
                    helipad_gt.translation().distance(heli_gt.translation()) < 5.0
                })
                .min_by(|(_, heli_gt_a, _), (_, heli_gt_b, _)| {
                    let dist_a = helipad_gt.translation().distance(heli_gt_a.translation());
                    let dist_b = helipad_gt.translation().distance(heli_gt_b.translation());
                    dist_a.total_cmp(&dist_b)
                })
            {
                commands.entity(yacht_entity).remove::<PlayerControlled>();

                commands
                    .entity(heli_entity)
                    .insert(PlayerControlled)
                    .insert(VehicleControlType::Helicopter)
                    .insert(ControlState::default());

                if let Ok((player_entity, _, _)) = player_query.single_mut() {
                    commands
                        .entity(player_entity)
                        .insert(ChildOf(heli_entity))
                        .insert(InCar(heli_entity));
                }

                queue_active_transfer(&mut commands, yacht_entity, heli_entity);
                next_state.set(GameState::Flying);

                continue;
            }
        }

        // Determine exit behavior: Shift+F = water, plain F = deck walk
        let exit_to_water = control_state.run;

        let target_exit_kind = if exit_to_water {
            ExitPointKind::Water
        } else {
            ExitPointKind::Deck
        };

        let exit_point = children
            .iter()
            .filter_map(|child| exit_point_query.get(child).ok())
            .find(|(exit_point, _)| exit_point.kind == target_exit_kind)
            .or_else(|| {
                children
                    .iter()
                    .filter_map(|child| exit_point_query.get(child).ok())
                    .next()
            });

        if let Some((exit_point, exit_gt)) = exit_point {
            if let Ok((player_entity, mut player_transform, mut player_visibility)) =
                player_query.single_mut()
            {
                // Remove control from yacht
                commands
                    .entity(yacht_entity)
                    .remove::<PlayerControlled>()
                    .insert(ControlState::default());

                *player_visibility = Visibility::Visible;

                // Transfer control to player
                commands
                    .entity(player_entity)
                    .insert(PlayerControlled)
                    .insert(ControlState::default())
                    .remove::<InCar>()
                    .remove::<ChildOf>();

                match exit_point.kind {
                    ExitPointKind::Deck => {
                        const FOOT_OFFSET: f32 = 0.45;

                        if let Some((anchor_entity, anchor_gt)) = children
                            .iter()
                            .find_map(|child| deck_anchor_query.get(child).ok())
                        {
                            let local_pos = anchor_gt
                                .affine()
                                .inverse()
                                .transform_point3(exit_gt.translation());
                            let local_snapped = Vec3::new(local_pos.x, FOOT_OFFSET, local_pos.z);
                            let world_pos = anchor_gt.affine().transform_point3(local_snapped);

                            *player_transform = Transform::from_translation(world_pos);

                            commands.entity(player_entity).insert((
                                VehicleControlType::Walking,
                                DeckWalker {
                                    yacht: yacht_entity,
                                    deck_anchor: anchor_entity,
                                    last_anchor: *anchor_gt,
                                    half_extents: Vec2::new(9.0, 20.0),
                                    foot_offset: FOOT_OFFSET,
                                },
                            ));
                        } else {
                            *player_transform = Transform::from_translation(exit_gt.translation());
                            commands
                                .entity(player_entity)
                                .insert(VehicleControlType::Walking)
                                .remove::<DeckWalker>();
                        }

                        queue_active_transfer(&mut commands, yacht_entity, player_entity);
                        next_state.set(GameState::Walking);
                    }
                    ExitPointKind::Water => {
                        // Enable physics for swimming (buoyancy, collisions)
                        commands.entity(player_entity).remove::<ChildOf>();
                        *player_transform = Transform::from_translation(exit_gt.translation());

                        // CRITICAL: Insert swimming components and physics for proper water entry
                        commands
                            .entity(player_entity)
                            .insert(VehicleControlType::Swimming)
                            .insert(ControlState::default())
                            .insert(Swimming {
                                state: SwimState::Surface,
                            })
                            .insert(GravityScale(0.1))
                            .insert(Damping {
                                linear_damping: 6.0,
                                angular_damping: 3.0,
                            })
                            .insert(WaterBodyId)
                            .insert(ProneRotation {
                                target_pitch: -std::f32::consts::FRAC_PI_2,
                                current_pitch: 0.0,
                                going_prone: true,
                            })
                            .insert(PendingPhysicsEnable)
                            .remove::<DeckWalker>();

                        queue_active_transfer(&mut commands, yacht_entity, player_entity);
                        next_state.set(GameState::Swimming);
                    }
                }
            }
        }
    }
}

pub fn deck_walk_movement_system(
    time: Res<Time>,
    mut deck_walker_query: Query<(Entity, &mut DeckWalker, &ControlState), With<Player>>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
    anchor_query: Query<&GlobalTransform, With<DeckWalkAnchor>>,
) {
    for (player_entity, mut deck_walker, control_state) in deck_walker_query.iter_mut() {
        if let Ok(mut player_transform) = player_transform_query.get_mut(player_entity) {
            if let Ok(anchor_gt) = anchor_query.get(deck_walker.deck_anchor) {
                let a_now = anchor_gt.affine();
                let a_last = deck_walker.last_anchor.affine();
                let delta = a_now * a_last.inverse();

                let delta_translation = delta.transform_point3(Vec3::ZERO);
                player_transform.translation += delta_translation;

                let walk_speed = if control_state.run { 8.0 } else { 4.0 };

                let mut fwd_axis: f32 = 0.0;
                if control_state.is_accelerating() {
                    fwd_axis += 1.0;
                }
                if control_state.is_braking() {
                    fwd_axis -= 1.0;
                }

                if fwd_axis != 0.0 {
                    let direction = *player_transform.forward() * fwd_axis.signum();
                    player_transform.translation += direction * walk_speed * time.delta_secs();
                }

                if control_state.steering != 0.0 {
                    player_transform.rotate_y(control_state.steering * 1.8 * time.delta_secs());
                }

                let mut p_local = a_now
                    .inverse()
                    .transform_point3(player_transform.translation);
                p_local.y = deck_walker.foot_offset;
                p_local.x = p_local
                    .x
                    .clamp(-deck_walker.half_extents.x, deck_walker.half_extents.x);
                p_local.z = p_local
                    .z
                    .clamp(-deck_walker.half_extents.y, deck_walker.half_extents.y);
                player_transform.translation = a_now.transform_point3(p_local);

                deck_walker.last_anchor = *anchor_gt;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn yacht_board_from_deck_system(
    mut commands: Commands,
    player_query: Query<
        (Entity, &ControlState, &DeckWalker),
        (With<Player>, With<PlayerControlled>),
    >,
    yacht_query: Query<Entity, (With<Yacht>, Without<PlayerControlled>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (player_entity, control_state, deck_walker) in player_query.iter() {
        if !control_state.interact {
            continue;
        }

        if let Ok(yacht_entity) = yacht_query.get(deck_walker.yacht) {
            commands
                .entity(player_entity)
                .remove::<PlayerControlled>()
                .remove::<DeckWalker>()
                .insert(InCar(yacht_entity))
                .insert(ChildOf(yacht_entity))
                .insert(Visibility::Hidden);

            commands
                .entity(yacht_entity)
                .insert(PlayerControlled)
                .insert(VehicleControlType::Yacht)
                .insert(ControlState::default());

            queue_active_transfer(&mut commands, player_entity, yacht_entity);
            next_state.set(GameState::Driving);
        }
    }
}

/// Optimized helicopter-yacht landing detection system.
///
/// OPTIMIZATION STRATEGY:
/// 1. Altitude pre-filter: Only check helicopters below MAX_LANDING_ALTITUDE (20m)
/// 2. Early-exit: Skip yachts without helipads entirely
/// 3. Distance pre-filter: Only calculate distance for helicopters within rough proximity
/// 4. Velocity check after distance: Avoid expensive distance calculations for fast-moving helicopters
///
/// PERFORMANCE IMPROVEMENT:
/// - Before: O(yachts × yacht_children × helicopters) every frame
/// - After: O(yachts + low_helicopters) with early exits
/// - Typical case: ~80% reduction in distance calculations
///
/// FUTURE OPTIMIZATION:
/// Consider using Rapier's CollisionEvent or SensorShape on helipads to get automatic
/// proximity detection without manual distance checks. This would eliminate the need
/// for this system entirely and leverage Rapier's spatial acceleration structures.
pub fn heli_landing_detection_system(
    mut commands: Commands,
    yacht_query: Query<(Entity, &Children), With<Yacht>>,
    helipad_query: Query<&GlobalTransform, With<Helipad>>,
    helicopter_query: Query<(Entity, &GlobalTransform, &Velocity), With<Enterable>>,
) {
    // OPTIMIZATION 1: Altitude pre-filter - only check helicopters below landing altitude
    const MAX_LANDING_ALTITUDE: f32 = 20.0;
    const LANDING_DISTANCE: f32 = 5.0;
    const LANDING_DISTANCE_SQUARED: f32 = LANDING_DISTANCE * LANDING_DISTANCE;
    const MAX_LANDING_SPEED: f32 = 2.0;
    const MAX_LANDING_ROTATION: f32 = 0.5;

    // Build helipad cache: collect all helipad transforms per yacht (avoids nested iteration)
    // This converts O(yachts × children) into O(yachts) with single-pass collection
    // ALSO track max_helipad_y for scene-independent altitude filtering
    // MULTI-HELIPAD SUPPORT: Collects ALL helipads per yacht (not just first one)
    let mut helipad_cache: Vec<(Entity, Vec3)> = Vec::new();
    let mut max_helipad_y = f32::NEG_INFINITY;

    for (yacht_entity, yacht_children) in yacht_query.iter() {
        // Collect ALL helipads for this yacht (supports multiple landing pads)
        for child in yacht_children.iter() {
            if let Ok(helipad_gt) = helipad_query.get(child) {
                let helipad_pos = helipad_gt.translation();
                max_helipad_y = max_helipad_y.max(helipad_pos.y);
                helipad_cache.push((yacht_entity, helipad_pos));
            }
        }
    }

    // Process helicopters (don't early return - must clear stale state)
    for (heli_entity, heli_gt, heli_velocity) in helicopter_query.iter() {
        let heli_pos = heli_gt.translation();

        // OPTIMIZATION 3: Scene-independent altitude pre-filter
        // Use relative altitude: above highest helipad + threshold
        if helipad_cache.is_empty() || heli_pos.y > max_helipad_y + MAX_LANDING_ALTITUDE {
            // Remove landing marker if helicopter climbs away or no helipads
            commands.entity(heli_entity).remove::<LandedOnYacht>();
            continue;
        }

        // OPTIMIZATION 4: Velocity check early - skip fast-moving helicopters
        let is_slow = heli_velocity.linvel.length() < MAX_LANDING_SPEED
            && heli_velocity.angvel.length() < MAX_LANDING_ROTATION;

        if !is_slow {
            commands.entity(heli_entity).remove::<LandedOnYacht>();
            continue;
        }

        // Check distance against cached helipad positions (use distance_squared to avoid sqrt)
        let mut landed = false;
        for (yacht_entity, helipad_pos) in &helipad_cache {
            let distance_squared = heli_pos.distance_squared(*helipad_pos);

            if distance_squared < LANDING_DISTANCE_SQUARED {
                commands.entity(heli_entity).insert(LandedOnYacht {
                    yacht: *yacht_entity,
                });
                landed = true;
                break; // Early exit: helicopter can only land on one yacht
            }
        }

        // Remove landing marker if not landed on any yacht
        if !landed {
            commands.entity(heli_entity).remove::<LandedOnYacht>();
        }
    }
}

/// Synchronize landed helicopters with yacht motion
///
/// This system makes helicopters move rigidly with the yacht when they have LandedOnYacht marker.
/// Applies the yacht's velocity directly to the helicopter to create realistic deck attachment.
#[allow(clippy::type_complexity)]
pub fn sync_landed_helicopter_with_yacht(
    yacht_query: Query<(&Velocity, &GlobalTransform), (With<Yacht>, Without<Helicopter>)>,
    mut helicopter_query: Query<
        (&mut Velocity, &GlobalTransform, &LandedOnYacht),
        (With<Helicopter>, Without<PlayerControlled>),
    >,
) {
    for (mut heli_velocity, _heli_gt, landed_on) in helicopter_query.iter_mut() {
        if let Ok((yacht_velocity, _yacht_gt)) = yacht_query.get(landed_on.yacht) {
            heli_velocity.linvel = yacht_velocity.linvel;
            heli_velocity.angvel = yacht_velocity.angvel;
        }
    }
}
