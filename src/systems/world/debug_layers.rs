#![allow(clippy::type_complexity)]
use bevy::{
    prelude::*,
    render::view::{RenderLayers, visibility::VisibilityRange},
};

/// Debug rendering layers for selective visualization
pub const DEBUG_LAYER: usize = 1;
pub const UI_LAYER: usize = 2;
pub const WORLD_LAYER: usize = 0; // Default layer

/// Setup debug camera that only renders debug layer
pub fn setup_debug_camera(mut _commands: Commands) {
    #[cfg(feature = "debug-ui")]
    {
        _commands.spawn((
            Camera3d::default(),
            RenderLayers::layer(DEBUG_LAYER),
            Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("DebugCamera"),
        ));
    }
}

/// Add debug visualization to existing entities - TEMPORARILY DISABLED
pub fn add_debug_visualization(
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    _entity_query: Query<(Entity, &Transform), (With<VisibilityRange>, Without<Camera>)>,
) {
    // TEMPORARILY DISABLED
    #[cfg(never)] // was: "debug-ui" 
    {
        let debug_material = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.0, 0.0, 0.3),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        for (entity, transform) in entity_query.iter().take(10) {
            // Limit to first 10 for demo
            // Add debug wireframe sphere to show visibility range
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Sphere::new(2.0))),
                    MeshMaterial3d(debug_material.clone()),
                    Transform::IDENTITY,
                    RenderLayers::layer(DEBUG_LAYER),
                    Name::new("DebugBounds"),
                ));
            });
        }
    }
}

/// Toggle debug layer visibility on camera
pub fn toggle_debug_layer_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut RenderLayers, With<Camera>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        for mut render_layers in camera_query.iter_mut() {
            if render_layers.intersects(&RenderLayers::layer(DEBUG_LAYER)) {
                // Remove debug layer
                *render_layers = render_layers.clone().without(DEBUG_LAYER);
                info!("Debug layer disabled");
            } else {
                // Add debug layer
                *render_layers = render_layers.clone().with(DEBUG_LAYER);
                info!("Debug layer enabled");
            }
        }
    }
}
