#![allow(clippy::type_complexity)]
use crate::components::{Boat, Lake, WaterBody, Yacht};
use crate::factories::{
    MaterialFactory, RenderingBundleType, RenderingFactory, StandardRenderingPattern,
};
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_rapier3d::prelude::*;

/// Phase 4: Setup water surfaces for carved terrain basins
/// 
/// Creates water surface entities positioned naturally in terrain-carved basins.
/// Water basins are now carved into the terrain heightmap, so we only need surface rendering.
pub fn setup_water_surfaces(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_config: Res<crate::systems::terrain::asset_based_terrain::LoadedTerrainConfig>,
) {
    let Some(ref config) = terrain_config.config else {
        warn!("Cannot setup water surfaces: terrain config not loaded");
        return;
    };

    info!("Setting up {} water surfaces for carved terrain basins", config.water_areas.len());

    // Create water surface for each water area in terrain config
    for (index, water_area) in config.water_areas.iter().enumerate() {
        let water_position = Vec3::new(
            water_area.center.0,
            water_area.depth + 0.1, // Float slightly above carved basin floor
            water_area.center.1,
        );

        // Create water surface mesh scaled to match carved basin
        let water_entity = RenderingFactory::create_rendering_entity(
            &mut commands,
            &mut meshes,
            &mut materials,
            StandardRenderingPattern::WaterSurface {
                size: water_area.radius * 2.0, // Diameter matches carved basin
                color: Color::srgba(0.1, 0.4, 0.8, 0.7),
            },
            water_position,
            RenderingBundleType::Standalone,
            None,
        );

        commands.entity(water_entity).insert((
            Lake {
                size: water_area.radius * 2.0,
                depth: water_area.depth.abs(), // Positive depth value
                wave_height: 0.5,
                wave_speed: 1.0,
                position: water_position,
            },
            WaterBody,
            RigidBody::Fixed,
            Collider::cylinder(0.05, water_area.radius), // Thin cylinder for water detection
            Sensor,
            Name::new(format!("Water Surface {}: {}", index + 1, water_area.description)),
        ));

        info!("Created water surface at ({:.1}, {:.1}) radius={:.1} depth={:.1}",
              water_area.center.0, water_area.center.1, water_area.radius, water_area.depth);
    }
}

pub fn setup_yacht(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_config: Res<crate::systems::terrain::asset_based_terrain::LoadedTerrainConfig>,
) {
    let Some(ref config) = terrain_config.config else {
        warn!("Cannot setup yacht: terrain config not loaded");
        return;
    };

    // Position yacht in the first (largest) water area
    let yacht_position = if let Some(first_water) = config.water_areas.first() {
        Vec3::new(
            first_water.center.0,
            first_water.depth + 1.0, // Float above water surface
            first_water.center.1,
        )
    } else {
        warn!("No water areas defined, placing yacht at origin");
        Vec3::new(0.0, 1.0, 0.0)
    };

    // Yacht hull - FACTORY PATTERN
    let yacht_id = RenderingFactory::create_rendering_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        StandardRenderingPattern::VehicleBody {
            vehicle_type: crate::factories::VehicleBodyType::Boat,
            color: Color::srgb(0.9, 0.9, 0.9),
        },
        yacht_position,
        RenderingBundleType::Parent,
        None,
    );

    commands.entity(yacht_id).insert((
        RigidBody::Dynamic,
        Collider::cuboid(4.0, 1.0, 10.0),
        Yacht {
            speed: 0.0,
            max_speed: 25.0,
            turning_speed: 2.0,
            buoyancy: 15.0,
            wake_enabled: true,
        },
        Boat,
        VisibilityRange::abrupt(0.0, 300.0),
        Name::new("Yacht"),
    ));

    // Yacht cabin
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(6.0, 3.0, 8.0))),
            MeshMaterial3d(MaterialFactory::create_metallic_material(
                &mut materials,
                Color::srgb(0.8, 0.8, 0.9),
                0.3,
                0.4,
            )),
            Transform::from_xyz(0.0, 3.5, -2.0),
            Name::new("Yacht Cabin"),
        ))
        .insert(ChildOf(yacht_id));

    // Yacht mast
    commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(0.2, 15.0))),
            MeshMaterial3d(MaterialFactory::create_metallic_material(
                &mut materials,
                Color::srgb(0.6, 0.4, 0.2),
                0.1,
                0.8,
            )),
            Transform::from_xyz(0.0, 9.5, 2.0),
            Name::new("Yacht Mast"),
        ))
        .insert(ChildOf(yacht_id));
}

pub fn yacht_movement_system(
    time: Res<Time>,
    mut yacht_query: Query<
        (
            &mut Transform,
            &mut Yacht,
            &mut Velocity,
            &crate::components::ControlState,
        ),
        (With<Boat>, With<crate::components::ActiveEntity>),
    >,
) {
    for (mut transform, yacht, mut velocity, control_state) in yacht_query.iter_mut() {
        let mut acceleration = Vec3::ZERO;
        let mut angular_velocity = 0.0;

        // Forward/backward movement using ControlState
        if control_state.is_accelerating() {
            acceleration += transform.forward() * yacht.max_speed * control_state.throttle;
        }
        if control_state.is_braking() {
            acceleration -= transform.forward() * yacht.max_speed * control_state.brake * 0.5;
        }

        // Turning using ControlState steering
        if control_state.steering.abs() > 0.1 {
            angular_velocity = yacht.turning_speed * control_state.steering;
        }

        // Boost functionality
        let boost_multiplier = if control_state.is_boosting() {
            2.0
        } else {
            1.0
        };
        acceleration *= boost_multiplier;

        // Apply rotation
        transform.rotate_y(angular_velocity * time.delta_secs());

        // Apply movement with water resistance while preserving gravity
        let drag = 0.95;
        let new_velocity = velocity.linvel * drag + acceleration * time.delta_secs() * 0.1;

        // Preserve gravity in Y-axis (yachts can fall if lifted out of water)
        velocity.linvel = Vec3::new(
            new_velocity.x,
            velocity.linvel.y, // Preserve gravity
            new_velocity.z,
        );

        // Keep yacht on water surface (simple buoyancy)
        if transform.translation.y < 0.5 {
            velocity.linvel.y += yacht.buoyancy * time.delta_secs();
        }
    }
}

pub fn water_wave_system(
    time: Res<Time>,
    mut lake_query: Query<(&mut Transform, &Lake), With<WaterBody>>,
) {
    for (mut transform, lake) in lake_query.iter_mut() {
        let wave_offset = (time.elapsed_secs() * lake.wave_speed).sin() * lake.wave_height * 0.1;
        transform.translation.y = wave_offset;
    }
}

pub fn yacht_buoyancy_system(
    mut yacht_query: Query<(&Transform, &mut Velocity, &Yacht), With<Boat>>,
    terrain_config: Res<crate::systems::terrain::asset_based_terrain::LoadedTerrainConfig>,
) {
    let Some(ref config) = terrain_config.config else {
        return; // Skip if terrain config not loaded
    };

    for (yacht_transform, mut velocity, yacht) in yacht_query.iter_mut() {
        let yacht_position = yacht_transform.translation;
        let yacht_bottom = yacht_position.y - 1.0;

        // Check if yacht is in any water area using terrain config
        if let Some(water_area) = config.get_water_area_at(yacht_position.x, yacht_position.z) {
            let distance_from_center = ((yacht_position.x - water_area.center.0).powi(2) + 
                                      (yacht_position.z - water_area.center.1).powi(2)).sqrt();

            // Keep yacht within water area boundaries
            if distance_from_center > water_area.radius * 0.9 { // 10% buffer from edge
                // Push yacht back toward water center
                let direction_to_center = Vec2::new(
                    water_area.center.0 - yacht_position.x,
                    water_area.center.1 - yacht_position.z,
                ).normalize();

                velocity.linvel.x += direction_to_center.x * 5.0;
                velocity.linvel.z += direction_to_center.y * 5.0;
            }

            // Apply buoyancy if yacht is below water surface
            let water_surface_level = water_area.depth + 0.1; // Water surface height
            if yacht_bottom < water_surface_level {
                let submersion = water_surface_level - yacht_bottom;
                let buoyancy_force = submersion * yacht.buoyancy;
                velocity.linvel.y += buoyancy_force * 0.1;

                // Water resistance
                velocity.linvel *= 0.98;
            }
        } else {
            // Yacht is outside water areas - apply gravity and try to get back to water
            if let Some(nearest_water) = config.water_areas.first() {
                let direction_to_water = Vec2::new(
                    nearest_water.center.0 - yacht_position.x,
                    nearest_water.center.1 - yacht_position.z,
                ).normalize();

                // Gentle push toward nearest water area
                velocity.linvel.x += direction_to_water.x * 2.0;
                velocity.linvel.z += direction_to_water.y * 2.0;
            }
        }
    }
}

pub fn yacht_water_constraint_system(
    mut yacht_query: Query<(&mut Transform, &mut Velocity), With<Yacht>>,
    terrain_config: Res<crate::systems::terrain::asset_based_terrain::LoadedTerrainConfig>,
) {
    let Some(ref config) = terrain_config.config else {
        return; // Skip if terrain config not loaded
    };

    for (mut transform, mut velocity) in yacht_query.iter_mut() {
        let yacht_position = transform.translation;

        // Ensure yacht stays above minimum depth
        let min_depth = config.water_areas.iter()
            .map(|w| w.depth)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(-4.0);

        if yacht_position.y < min_depth - 2.0 {
            // Reset yacht to nearest water surface
            if let Some(nearest_water) = config.water_areas.first() {
                transform.translation.x = nearest_water.center.0;
                transform.translation.z = nearest_water.center.1;
                transform.translation.y = nearest_water.depth + 1.0;
                velocity.linvel = Vec3::ZERO;
                info!("Yacht reset to water surface at ({:.1}, {:.1})", 
                      nearest_water.center.0, nearest_water.center.1);
            }
        }

        // Check if yacht has strayed too far from all water areas
        let min_distance_to_water = config.water_areas.iter()
            .map(|water| {
                ((yacht_position.x - water.center.0).powi(2) + 
                 (yacht_position.z - water.center.1).powi(2)).sqrt() - water.radius
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(f32::MAX);

        // If yacht is more than 200m from any water area, teleport back
        if min_distance_to_water > 200.0 {
            if let Some(nearest_water) = config.water_areas.first() {
                transform.translation.x = nearest_water.center.0;
                transform.translation.z = nearest_water.center.1;
                transform.translation.y = nearest_water.depth + 1.0;
                velocity.linvel = Vec3::ZERO;
                info!("Yacht teleported back to water - was too far from water areas");
            }
        }
    }
}
