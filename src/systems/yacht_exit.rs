use bevy::prelude::*;
use bevy_rapier3d::geometry::Collider;
use bevy_rapier3d::prelude::Velocity;

use crate::components::{
    ControlState, DeckWalkAnchor, DeckWalkable, DeckWalker, Enterable, ExitPoint, ExitPointKind,
    Helipad, InCar, LandedOnYacht, PendingPhysicsEnable, Player, PlayerControlled,
    VehicleControlType, Yacht,
};
use crate::game_state::GameState;
use crate::systems::safe_active_entity::queue_active_transfer;

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn yacht_exit_system(
    mut commands: Commands,
    yacht_query: Query<(Entity, &ControlState, &Children), (With<Yacht>, With<PlayerControlled>)>,
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
                commands.entity(yacht_entity).remove::<PlayerControlled>();

                *player_visibility = Visibility::Visible;

                commands
                    .entity(player_entity)
                    .insert(PlayerControlled)
                    .insert(PendingPhysicsEnable)
                    .insert(ControlState::default())
                    .remove::<InCar>();

                match exit_point.kind {
                    ExitPointKind::Deck => {
                        if let Some((anchor_entity, anchor_gt)) = children
                            .iter()
                            .find_map(|child| deck_anchor_query.get(child).ok())
                        {
                            let local_pos = anchor_gt
                                .affine()
                                .inverse()
                                .transform_point3(exit_gt.translation());
                            *player_transform = Transform::from_translation(local_pos);

                            commands.entity(player_entity).insert((
                                VehicleControlType::Walking,
                                DeckWalker {
                                    yacht: yacht_entity,
                                },
                                ChildOf(anchor_entity),
                            ));
                        } else {
                            commands.entity(player_entity).remove::<ChildOf>();
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
                        commands.entity(player_entity).remove::<ChildOf>();
                        *player_transform = Transform::from_translation(exit_gt.translation());

                        commands
                            .entity(player_entity)
                            .insert(VehicleControlType::Swimming)
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
    deck_walker_query: Query<(Entity, &DeckWalker), With<Player>>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
    yacht_children_query: Query<&Children, With<Yacht>>,
    deck_volume_query: Query<&Collider, With<DeckWalkable>>,
) {
    for (player_entity, deck_walker) in deck_walker_query.iter() {
        if let Ok(mut player_transform) = player_transform_query.get_mut(player_entity) {
            if let Ok(yacht_children) = yacht_children_query.get(deck_walker.yacht) {
                if let Some(deck_collider) = yacht_children
                    .iter()
                    .filter_map(|child| deck_volume_query.get(child).ok())
                    .next()
                {
                    if let Some(cuboid) = deck_collider.as_cuboid() {
                        let half_extents = cuboid.half_extents();
                        let local_pos = player_transform.translation;
                        let clamped = Vec3::new(
                            local_pos.x.clamp(-half_extents.x, half_extents.x),
                            local_pos.y,
                            local_pos.z.clamp(-half_extents.z, half_extents.z),
                        );
                        player_transform.translation = clamped;
                    }
                }
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
                .remove::<ChildOf>()
                .insert(InCar(yacht_entity));

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
