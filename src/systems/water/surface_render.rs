use crate::components::unified_water::{UnifiedWaterBody, WaterSurface};
use crate::components::water_material::WaterMaterial;
use crate::factories::create_subdivided_plane;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;

/// Water surface rendering system - creates visual water planes with animated waves
pub fn surface_render_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    water_regions: Query<(Entity, &UnifiedWaterBody), Added<UnifiedWaterBody>>,
) {
    for (entity, region) in water_regions.iter() {
        // Calculate water plane dimensions
        let width = region.bounds.2 - region.bounds.0; // max_x - min_x
        let depth = region.bounds.3 - region.bounds.1; // max_z - min_z
        let center_x = (region.bounds.0 + region.bounds.2) / 2.0;
        let center_z = (region.bounds.1 + region.bounds.3) / 2.0;

        // Calculate subdivisions based on region size
        // Target: ~20-30m per vertex (balanced detail vs performance)
        let max_dimension = width.max(depth);
        let subdivisions = if max_dimension > 1000.0 {
            256 // Large ocean: ~23m per vertex
        } else if max_dimension > 500.0 {
            128 // Medium water: ~4-8m per vertex
        } else {
            64 // Small lake: ~3m per vertex
        };

        // Create subdivided water surface mesh for wave detail
        let water_mesh = meshes.add(create_subdivided_plane(width, depth, subdivisions));

        // Map wave_params from config to material if present
        let mut water_material = WaterMaterial {
            base_color: LinearRgba::new(
                region.surface_color.0,
                region.surface_color.1,
                region.surface_color.2,
                region.surface_color.3,
            ),
            ..Default::default()
        };

        // Apply wave parameters from config if available
        if let Some(wave_params) = &region.wave_params {
            // Scale default wave amplitudes by config amplitude ratio
            let amplitude_scale = (wave_params.amplitude / 0.25).clamp(0.5, 2.0);
            for i in 0..4 {
                water_material.wave_data0[i].z *= amplitude_scale;
            }

            // Override wave speeds if config specifies
            if wave_params.speed > 0.0 {
                for i in 0..4 {
                    water_material.wave_data1[i].x = wave_params.speed;
                }
            }
        }

        let water_material_handle = materials.add(water_material);

        // Spawn water surface entity with link to parent region
        let surface_entity = commands
            .spawn((
                Mesh3d(water_mesh),
                MeshMaterial3d(water_material_handle),
                Transform::from_xyz(center_x, region.get_water_surface_level(0.0), center_z),
                VisibilityRange::abrupt(0.0, 2000.0), // Visible up to 2km
                NotShadowCaster,                      // Water should not cast shadows
                WaterSurface {
                    region_entity: entity,
                }, // Direct link for O(1) updates
                Name::new(format!("{} Surface", region.name)),
            ))
            .id();

        // Attach surface as child of water region
        commands.entity(entity).add_children(&[surface_entity]);

        info!(
            "Created water surface for {} at ({:.1}, {:.1}) size {:.1}x{:.1} with {}x{} subdivisions (Gerstner waves)",
            region.name, center_x, center_z, width, depth, subdivisions, subdivisions
        );
    }
}

/// Update water surface positions based on tide changes (O(N) with direct entity links)
pub fn update_water_surface_system(
    time: Res<Time>,
    water_regions: Query<(Entity, &UnifiedWaterBody)>,
    mut surface_query: Query<(&WaterSurface, &mut Transform), With<MeshMaterial3d<WaterMaterial>>>,
) {
    let current_time = time.elapsed_secs();

    // O(N) update: iterate surfaces once, lookup region by entity
    for (water_surface, mut transform) in surface_query.iter_mut() {
        if let Ok((_, region)) = water_regions.get(water_surface.region_entity) {
            let new_level = region.get_water_surface_level(current_time);

            // Only update if level changed significantly (avoid unnecessary writes)
            if (new_level - transform.translation.y).abs() > 0.01 {
                transform.translation.y = new_level;
            }
        }
    }
}
