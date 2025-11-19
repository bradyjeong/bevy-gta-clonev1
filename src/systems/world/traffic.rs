use crate::components::ActiveEntity;
use crate::components::traffic::{TrafficAI, TrafficState, TrafficVehicle};
use crate::config::GameConfig;
use crate::systems::world::road_network::RoadNetwork;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

#[derive(Resource, Default)]
pub struct TrafficManager {
    pub spawn_timer: f32,
}

const MAX_TRAFFIC_CARS: usize = 50;
const SPAWN_DISTANCE_MIN: f32 = 100.0;
const SPAWN_DISTANCE_MAX: f32 = 200.0;
const DESPAWN_DISTANCE: f32 = 250.0;
const TRAFFIC_SPEED: f32 = 15.0; // m/s

#[allow(clippy::too_many_arguments)]
pub fn spawn_traffic_system(
    mut commands: Commands,
    time: Res<Time>,
    mut manager: ResMut<TrafficManager>,
    road_network: Res<RoadNetwork>,
    player_query: Query<&Transform, With<ActiveEntity>>,
    traffic_query: Query<Entity, With<TrafficVehicle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
) {
    manager.spawn_timer += time.delta_secs();
    if manager.spawn_timer < 0.1 {
        return;
    }
    manager.spawn_timer = 0.0;

    #[allow(deprecated)]
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    let current_count = traffic_query.iter().len();
    if current_count >= MAX_TRAFFIC_CARS {
        return;
    }

    // Try to spawn a car
    let mut rng = rand::thread_rng();

    // Pick a random road
    if road_network.roads.is_empty() {
        return;
    }

    // Inefficient but works for now: convert keys to vector or just iterate randomly
    // Optimization: RoadNetwork should ideally have a spatial index, but for now we just pick random IDs
    // if we have a known range of IDs. But IDs are sparse.
    // Let's just pick a few random attempts.

    let road_ids: Vec<u64> = road_network.roads.keys().cloned().collect();
    if road_ids.is_empty() {
        return;
    }

    for _ in 0..5 {
        let road_id = road_ids[rng.gen_range(0..road_ids.len())];
        let road = &road_network.roads[&road_id];

        // Check distance
        let center = road.evaluate(0.5);
        let dist = center.distance(player_pos);

        if (SPAWN_DISTANCE_MIN..=SPAWN_DISTANCE_MAX).contains(&dist) {
            // Spawn here
            let t = rng.gen_range(0.0..1.0);
            let lane = if rng.gen_bool(0.5) { 0 } else { -1 }; // 0 = forward, -1 = backward

            let pos = road.get_lane_position(t, lane);

            spawn_traffic_car(
                &mut commands,
                &mut meshes,
                &mut materials,
                pos,
                road_id,
                lane,
                t,
                &config,
            );
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_traffic_car(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    road_id: u64,
    lane: i32,
    t: f32,
    config: &GameConfig,
) {
    let mut rng = rand::thread_rng();
    let color = Color::srgb(rng.r#gen(), rng.r#gen(), rng.r#gen());

    let car_entity = commands
        .spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(0.9, 0.6, 2.1),
            CollisionGroups::new(
                config.physics.vehicle_group,
                config.physics.static_group
                    | config.physics.vehicle_group
                    | config.physics.character_group,
            ),
            TrafficVehicle,
            TrafficAI {
                current_road_id: road_id,
                current_lane: lane,
                spline_t: t,
                speed: TRAFFIC_SPEED,
                state: TrafficState::Moving,
                target_speed: TRAFFIC_SPEED,
            },
        ))
        .id();

    // Simple visual mesh (box)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 1.2, 4.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.6, 0.0),
        ChildOf(car_entity),
    ));
}

pub fn despawn_traffic_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<ActiveEntity>>,
    traffic_query: Query<(Entity, &Transform), With<TrafficVehicle>>,
) {
    #[allow(deprecated)]
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, transform) in traffic_query.iter() {
        if transform.translation.distance(player_pos) > DESPAWN_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}

pub fn move_traffic_system(
    mut traffic_query: Query<(&mut Transform, &mut TrafficAI)>,
    road_network: Res<RoadNetwork>,
    time: Res<Time>,
    // For raycast
    rapier_context: ReadRapierContext,
) {
    let Ok(context) = rapier_context.single() else {
        return;
    };
    let dt = time.delta_secs();

    for (mut transform, mut ai) in traffic_query.iter_mut() {
        // 1. Obstacle Avoidance (Raycast)
        let forward = transform.forward();
        let ray_pos = transform.translation + Vec3::new(0.0, 0.5, 0.0) + (forward * 2.5);
        let ray_dir = *forward;
        let max_toi = 15.0;
        let solid = true;
        let filter = QueryFilter::default().exclude_sensors(); // Filter out sensors

        let mut _obstacle_detected = false;

        if let Some((_entity, toi)) = context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter) {
            // Obstacle ahead
            _obstacle_detected = true;
            // Simple braking logic
            let dist = toi;
            if dist < 5.0 {
                ai.target_speed = 0.0;
            } else {
                ai.target_speed = TRAFFIC_SPEED * (dist / 15.0);
            }
        } else {
            ai.target_speed = TRAFFIC_SPEED;
        }

        // 2. Update Speed
        ai.speed = ai.speed.lerp(ai.target_speed, 5.0 * dt);

        if ai.speed < 0.1 && ai.target_speed < 0.1 {
            continue; // Stopped
        }

        // 3. Move along spline
        if let Some(road) = road_network.roads.get(&ai.current_road_id) {
            let length = road.length();
            if length < 1.0 {
                continue;
            } // Safety

            let distance_traveled = ai.speed * dt;
            let delta_t = distance_traveled / length;

            // Depending on lane direction, we increase or decrease t?
            // Assuming roads are one-way for now, or t always increases.
            // But `lane_index` logic suggests bidirectional.
            // If lane < 0, we are moving against the spline?
            // Standard GTA roads usually handle lanes by offset.
            // Let's assume all movement is +t for now, but if lane < 0, we should probably move -t?
            // Actually, `road_network.rs` doesn't seem to have directionality explicitly encoded other than start/end.
            // Usually, lane < 0 implies opposite direction.

            let moving_forward = ai.current_lane >= 0;

            if moving_forward {
                ai.spline_t += delta_t;
            } else {
                ai.spline_t -= delta_t;
            }

            // Handle Road End/Start
            if (moving_forward && ai.spline_t >= 1.0) || (!moving_forward && ai.spline_t <= 0.0) {
                // Teleport to start/end or find connection
                // For now: Despawn/Respawn loop or just loop around if connected.
                // Since we don't have full graph nav implemented yet, let's just wrap around for testing
                // or reverse direction?
                // Reversing direction is easy.
                if moving_forward {
                    ai.spline_t = 1.0;
                    // Attempt to find connection
                    if !road.connections.is_empty() {
                        let _next_road_id = road.connections
                            [rand::thread_rng().gen_range(0..road.connections.len())];
                        // We need to know if we enter at start or end of next road.
                        // This requires complex graph logic.
                        // Fallback: Just wrap to start of same road
                        ai.spline_t = 0.0;
                    } else {
                        ai.spline_t = 0.0;
                    }
                } else {
                    ai.spline_t = 0.0;
                    if !road.connections.is_empty() {
                        // Fallback wrap
                        ai.spline_t = 1.0;
                    } else {
                        ai.spline_t = 1.0;
                    }
                }
            }

            // 4. Update Transform
            let pos = road.get_lane_position(ai.spline_t, ai.current_lane);

            // Look at target
            let look_t = if moving_forward {
                (ai.spline_t + 0.01).min(1.0)
            } else {
                (ai.spline_t - 0.01).max(0.0)
            };

            let target_pos = road.get_lane_position(look_t, ai.current_lane);

            transform.translation = pos;

            if pos.distance_squared(target_pos) > 0.0001 {
                transform.look_at(target_pos, Vec3::Y);
            }
        }
    }
}

// Placeholder for traffic lights
pub fn traffic_light_system() {
    // Implement light cycling logic
}
