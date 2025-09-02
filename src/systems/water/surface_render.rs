use crate::components::water_new::WaterRegion;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;

/// Water surface rendering system - creates visual water planes
pub fn surface_render_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    water_regions: Query<(Entity, &WaterRegion), Added<WaterRegion>>,
) {
    for (entity, region) in water_regions.iter() {
        // Calculate water plane dimensions
        let width = region.bounds.2 - region.bounds.0;  // max_x - min_x
        let depth = region.bounds.3 - region.bounds.1;  // max_z - min_z
        let center_x = (region.bounds.0 + region.bounds.2) / 2.0;
        let center_z = (region.bounds.1 + region.bounds.3) / 2.0;

        // Create water surface mesh
        let water_mesh = meshes.add(Plane3d::default().mesh().size(width, depth));
        
        // Create semi-transparent water material
        let water_material = materials.add(StandardMaterial {
            base_color: Color::srgba(
                region.surface_color.0,
                region.surface_color.1,
                region.surface_color.2,
                region.surface_color.3,
            ),
            metallic: 0.1,
            perceptual_roughness: 0.1,
            reflectance: 0.8,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        // Spawn water surface entity as child of water region
        let surface_entity = commands.spawn((
            Mesh3d(water_mesh),
            MeshMaterial3d(water_material),
            Transform::from_xyz(center_x, region.current_level, center_z),
            VisibilityRange::abrupt(0.0, 2000.0), // Visible up to 2km
            Name::new(format!("{} Surface", region.name)),
        )).id();

        // Attach surface as child of water region
        commands.entity(entity).add_children(&[surface_entity]);

        info!(
            "Created water surface for {} at ({:.1}, {:.1}) size {:.1}x{:.1}",
            region.name,
            center_x,
            center_z,
            width,
            depth
        );
    }
}

/// Update water surface positions based on tide changes
pub fn update_water_surface_system(
    time: Res<Time>,
    mut water_regions: Query<&mut WaterRegion>,
    mut surface_transforms: Query<(&mut Transform, &Name), Without<WaterRegion>>,
) {
    let current_time = time.elapsed_secs();

    for mut region in water_regions.iter_mut() {
        let new_level = region.get_water_level(current_time);
        
        // Only update if level changed significantly
        if (new_level - region.current_level).abs() > 0.01 {
            region.current_level = new_level;

            // Update all water surface transforms that match this region
            for (mut transform, name) in surface_transforms.iter_mut() {
                if name.as_str().contains(&region.name) && name.as_str().contains("Surface") {
                    transform.translation.y = new_level;
                }
            }
        }
    }
}
