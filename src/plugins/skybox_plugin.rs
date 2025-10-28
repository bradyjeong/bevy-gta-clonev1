use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;

pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_skybox);
    }
}

#[derive(Component)]
struct Skybox;

fn setup_skybox(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let skybox_mesh = meshes.add(Sphere::new(9500.0).mesh().ico(5).expect(
        "Failed to create skybox icosphere mesh (subdivision level 5, radius 9500.0).\n\
                 This is a Bevy mesh generation error, not an asset loading issue.\n\
                 Troubleshooting:\n\
                 1. Check system memory (large icosphere requires ~500MB RAM)\n\
                 2. If running on low-end hardware, reduce subdivision in skybox_plugin.rs\n\
                 3. Try lower .ico() value (3-4 instead of 5) for fewer triangles\n\
                 4. Check Bevy version compatibility (requires 0.16+)",
    ));

    let sky_color = Color::srgb(0.4, 0.7, 1.0);
    let skybox_material = materials.add(StandardMaterial {
        base_color: sky_color,
        emissive: LinearRgba::from(sky_color) * 1.5,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Skybox,
        Mesh3d(skybox_mesh),
        MeshMaterial3d(skybox_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        NotShadowCaster,
        Name::new("Skybox Sphere"),
    ));
}
