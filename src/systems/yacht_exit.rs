use bevy::prelude::*;
use bevy_rapier3d::geometry::Collider;
use bevy_rapier3d::prelude::{Damping, GravityScale, Velocity};

use crate::components::unified_water::WaterBodyId;
use crate::components::{
    ControlState, DeckWalkAnchor, DeckWalker, Enterable, ExitPoint, ExitPointKind, Helipad, InCar,
    LandedOnYacht, PendingPhysicsEnable, Player, PlayerControlled, VehicleControlType, Yacht,
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

pub fn heli_landing_detection_system(
    mut commands: Commands,
    yacht_query: Query<(Entity, &Children), With<Yacht>>,
    helipad_query: Query<&GlobalTransform, With<Helipad>>,
    helicopter_query: Query<(Entity, &GlobalTransform, &Velocity), With<Enterable>>,
) {
    for (yacht_entity, yacht_children) in yacht_query.iter() {
        if let Some(helipad_gt) = yacht_children
            .iter()
            .find_map(|child| helipad_query.get(child).ok())
        {
            for (heli_entity, heli_gt, heli_velocity) in helicopter_query.iter() {
                let distance = helipad_gt.translation().distance(heli_gt.translation());
                let is_overlapping = distance < 5.0;

                let is_slow =
                    heli_velocity.linvel.length() < 2.0 && heli_velocity.angvel.length() < 0.5;

                if is_overlapping && is_slow {
                    commands.entity(heli_entity).insert(LandedOnYacht {
                        yacht: yacht_entity,
                    });
                } else {
                    commands.entity(heli_entity).remove::<LandedOnYacht>();
                }
            }
        }
    }
}
