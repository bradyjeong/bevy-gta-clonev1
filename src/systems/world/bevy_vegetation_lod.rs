use crate::components::*;
use bevy::{prelude::*, render::view::visibility::VisibilityRange};

/// New vegetation LOD system using Bevy's built-in VisibilityRange
/// This replaces the old manual mesh-swapping system with multiple child entities

#[derive(Component)]
pub struct VegetationParent {
    pub vegetation_type: VegetationType,
}

#[derive(Component)]
pub struct VegetationLODLevel {
    pub level: u8, // 0=Full, 1=Medium, 2=Billboard
}

#[derive(Debug, Clone, Copy)]
pub enum VegetationType {
    Tree,
    Bush,
    Grass,
    Flower,
}

/// Spawn vegetation with multiple LOD levels using VisibilityRange
pub fn spawn_vegetation_with_lod(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    vegetation_type: VegetationType,
) -> Entity {
    // Create parent entity
    let parent = commands
        .spawn((
            Transform::from_translation(position),
            Visibility::Visible,
            VegetationParent { vegetation_type },
            Name::new("VegetationLOD"),
        ))
        .id();

    // Create LOD level 0: Full detail (0-50m)
    let full_mesh = create_full_vegetation_mesh(vegetation_type);
    let full_material = create_vegetation_material(vegetation_type, 1.0);
    commands.spawn((
        Mesh3d(meshes.add(full_mesh)),
        MeshMaterial3d(materials.add(full_material)),
        Transform::IDENTITY,
        VisibilityRange::abrupt(0.0, 50.0), // Visible from 0-50m
        VegetationLODLevel { level: 0 },
        ChildOf(parent),
        Name::new("VegetationFull"),
    ));

    // Create LOD level 1: Medium detail (45-150m)
    let medium_mesh = create_medium_vegetation_mesh(vegetation_type);
    let medium_material = create_vegetation_material(vegetation_type, 0.8);
    commands.spawn((
        Mesh3d(meshes.add(medium_mesh)),
        MeshMaterial3d(materials.add(medium_material)),
        Transform::IDENTITY,
        VisibilityRange::abrupt(50.0, 150.0), // Visible from 50-150m
        VegetationLODLevel { level: 1 },
        ChildOf(parent),
        Name::new("VegetationMedium"),
    ));

    // Create LOD level 2: Billboard (145-300m)
    let billboard_mesh = create_billboard_mesh();
    let billboard_material = create_vegetation_material(vegetation_type, 0.6);
    commands.spawn((
        Mesh3d(meshes.add(billboard_mesh)),
        MeshMaterial3d(materials.add(billboard_material)),
        Transform::IDENTITY,
        VisibilityRange::abrupt(150.0, 300.0), // Visible from 150-300m
        VegetationLODLevel { level: 2 },
        VegetationBillboard {
            original_scale: Vec3::ONE,
            billboard_size: Vec2::new(2.0, 3.0),
        },
        ChildOf(parent),
        Name::new("VegetationBillboard"),
    ));

    parent
}

// Helper functions for mesh creation
fn create_full_vegetation_mesh(vegetation_type: VegetationType) -> Mesh {
    match vegetation_type {
        VegetationType::Tree => Mesh::from(Cuboid::new(0.5, 3.0, 0.5)),
        VegetationType::Bush => Mesh::from(Sphere::new(0.8)),
        VegetationType::Grass => Mesh::from(Plane3d::default().mesh().size(0.3, 0.3)),
        VegetationType::Flower => Mesh::from(Sphere::new(0.2)),
    }
}

fn create_medium_vegetation_mesh(vegetation_type: VegetationType) -> Mesh {
    match vegetation_type {
        VegetationType::Tree => Mesh::from(Cuboid::new(0.3, 2.0, 0.3)),
        VegetationType::Bush => Mesh::from(Sphere::new(0.5)),
        VegetationType::Grass => Mesh::from(Plane3d::default().mesh().size(0.2, 0.2)),
        VegetationType::Flower => Mesh::from(Sphere::new(0.1)),
    }
}

fn create_billboard_mesh() -> Mesh {
    Mesh::from(Plane3d::default().mesh().size(2.0, 3.0))
}

fn create_vegetation_material(vegetation_type: VegetationType, alpha: f32) -> StandardMaterial {
    let base_color = match vegetation_type {
        VegetationType::Tree => Color::srgb(0.2, 0.6, 0.1),
        VegetationType::Bush => Color::srgb(0.3, 0.7, 0.2),
        VegetationType::Grass => Color::srgb(0.4, 0.8, 0.3),
        VegetationType::Flower => Color::srgb(0.8, 0.4, 0.6),
    };

    StandardMaterial {
        base_color: Color::srgba(
            base_color.to_srgba().red,
            base_color.to_srgba().green,
            base_color.to_srgba().blue,
            alpha,
        ),
        alpha_mode: if alpha < 1.0 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        },
        ..default()
    }
}

/// System to make billboard vegetation face the camera (runs only for billboard LOD)
pub fn vegetation_billboard_facing_system(
    camera_query: Query<&Transform, (With<Camera>, Without<VegetationLODLevel>)>,
    mut billboard_query: Query<
        &mut Transform,
        (With<VegetationLODLevel>, With<VegetationBillboard>),
    >,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_transform.translation;

    for mut transform in billboard_query.iter_mut() {
        let direction = (camera_pos - transform.translation).normalize();
        let look_rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
        transform.rotation = look_rotation;
    }
}
