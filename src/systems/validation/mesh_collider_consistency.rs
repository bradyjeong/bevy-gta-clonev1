use crate::config::{GameConfig, VehicleTypeConfig};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Mesh-Collider Consistency System
/// Ensures visual meshes properly align with physics colliders
/// Following AGENT.md simplicity principles with automated validation
///
/// Configuration for mesh-collider relationship
#[derive(Debug, Clone)]
pub struct MeshColliderConfig {
    pub collider_size: Vec3,
    pub visual_scale: f32, // Multiplier for visual size (typically 1.5-2.0)
    pub collider_type: ColliderType,
}

#[derive(Debug, Clone)]
pub enum ColliderType {
    Cuboid,
    Capsule {
        radius_scale: f32,
        height_scale: f32,
    },
}

impl MeshColliderConfig {
    pub fn new_cuboid(collider_size: Vec3, visual_scale: f32) -> Self {
        Self {
            collider_size,
            visual_scale,
            collider_type: ColliderType::Cuboid,
        }
    }

    pub fn new_capsule(collider_size: Vec3, visual_scale: f32) -> Self {
        Self {
            collider_size,
            visual_scale,
            collider_type: ColliderType::Capsule {
                radius_scale: 1.0,
                height_scale: 1.0,
            },
        }
    }

    /// Create matching mesh and collider pair
    pub fn create_pair(&self, meshes: &mut ResMut<Assets<Mesh>>) -> (Handle<Mesh>, Collider) {
        let visual_size = self.collider_size * self.visual_scale;

        match self.collider_type {
            ColliderType::Cuboid => {
                let mesh = meshes.add(Cuboid::new(visual_size.x, visual_size.y, visual_size.z));
                let collider = Collider::cuboid(
                    self.collider_size.x / 2.0,
                    self.collider_size.y / 2.0,
                    self.collider_size.z / 2.0,
                );
                (mesh, collider)
            }
            ColliderType::Capsule {
                radius_scale,
                height_scale,
            } => {
                let radius = self.collider_size.x * radius_scale;
                let height = self.collider_size.z * height_scale;
                let mesh = meshes.add(Capsule3d::new(
                    radius * self.visual_scale,
                    height * self.visual_scale,
                ));
                let collider = Collider::capsule_z(radius, height / 2.0);
                (mesh, collider)
            }
        }
    }

    /// Validate mesh-collider consistency
    pub fn validate(&self, config: &GameConfig) -> Result<(), String> {
        // Check size bounds
        if self.collider_size.min_element() < config.physics.min_collider_size {
            return Err(format!("Collider too small: {:?}", self.collider_size));
        }

        if self.collider_size.max_element() > config.physics.max_collider_size {
            return Err(format!("Collider too large: {:?}", self.collider_size));
        }

        // Check visual scale reasonableness
        if self.visual_scale < 0.5 || self.visual_scale > 3.0 {
            return Err(format!("Visual scale unrealistic: {}", self.visual_scale));
        }

        Ok(())
    }
}

/// Startup system to validate all vehicle mesh-collider consistency
pub fn validate_vehicle_consistency(config: Res<GameConfig>) {
    let vehicles = [
        ("SuperCar", &config.vehicles.super_car),
        ("Helicopter", &config.vehicles.helicopter),
        ("F16", &config.vehicles.f16),
        ("Yacht", &config.vehicles.yacht),
    ];

    for (name, vehicle_config) in vehicles {
        let collider_size = vehicle_config.collider_size;
        let body_size = vehicle_config.body_size;

        // Check if collider is reasonable fraction of mesh (GTA-style: collider < mesh)
        let ratio = collider_size / body_size;

        if ratio.min_element() < 0.5 {
            warn!(
                "Vehicle {}: Collider too small (ratio < 0.5).\n\
                 Visual Mesh: {:.2} x {:.2} x {:.2}\n\
                 Collider:    {:.2} x {:.2} x {:.2}\n\
                 Ratio:       {:.2} x {:.2} x {:.2}\n\
                 Fix: In config.rs, increase collider_size to ~0.8x of body_size.\n\
                 Example: If body_size is Vec3(2.0, 1.0, 4.0), set collider_size to Vec3(1.6, 0.8, 3.2)",
                name,
                body_size.x,
                body_size.y,
                body_size.z,
                collider_size.x,
                collider_size.y,
                collider_size.z,
                ratio.x,
                ratio.y,
                ratio.z
            );
        }

        if ratio.max_element() > 1.0 {
            warn!(
                "Vehicle {}: Collider larger than visual mesh (physics bigger than visuals).\n\
                 Visual Mesh: {:.2} x {:.2} x {:.2}\n\
                 Collider:    {:.2} x {:.2} x {:.2}\n\
                 Ratio:       {:.2} x {:.2} x {:.2}\n\
                 Fix: In config.rs, reduce collider_size to be smaller than body_size.\n\
                 GTA-style: collider should be 0.7-0.9x of mesh for forgiving collision.",
                name,
                body_size.x,
                body_size.y,
                body_size.z,
                collider_size.x,
                collider_size.y,
                collider_size.z,
                ratio.x,
                ratio.y,
                ratio.z
            );
        }

        // Check for GTA-style forgiving collision (0.7-0.9x)
        let avg_ratio = (ratio.x + ratio.y + ratio.z) / 3.0;
        if (0.7..=0.9).contains(&avg_ratio) {
            info!(
                "Vehicle {}: GTA-style forgiving collision OK ({:.2}x ratio)",
                name, avg_ratio
            );
        } else {
            warn!(
                "Vehicle {}: Collision ratio {:.2}x outside GTA-style range [0.7-0.9].\n\
                 Current sizes:\n\
                 - Visual Mesh: {:.2} x {:.2} x {:.2}\n\
                 - Collider:    {:.2} x {:.2} x {:.2}\n\
                 Target: Collider should be 70-90% of mesh size for arcade-style forgiving collision.\n\
                 Fix: In config.rs, adjust collider_size = body_size * 0.8 (recommended).\n\
                 See AGENTS.md section 'Mesh-Collider Consistency' for details.",
                name,
                avg_ratio,
                body_size.x,
                body_size.y,
                body_size.z,
                collider_size.x,
                collider_size.y,
                collider_size.z
            );
        }
    }
}

/// Debug visualization system for colliders vs meshes
#[cfg(feature = "debug-ui")]
pub fn debug_render_colliders(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Collider), With<crate::components::Player>>,
    vehicle_query: Query<
        (&Transform, &Collider),
        (
            With<crate::components::VehicleBody>,
            Without<crate::components::Player>,
        ),
    >,
) {
    // Draw player collider in green
    for (transform, collider) in query.iter() {
        draw_collider_gizmo(&mut gizmos, transform, collider, Color::srgb(0.0, 1.0, 0.0));
    }

    // Draw vehicle colliders in red
    for (transform, collider) in vehicle_query.iter() {
        draw_collider_gizmo(&mut gizmos, transform, collider, Color::srgb(1.0, 0.0, 0.0));
    }
}

#[cfg(feature = "debug-ui")]
fn draw_collider_gizmo(
    gizmos: &mut Gizmos,
    transform: &Transform,
    collider: &Collider,
    color: Color,
) {
    match collider.shape() {
        bevy_rapier3d::prelude::ColliderShape::Cuboid(cuboid) => {
            let size = Vec3::new(
                cuboid.half_extents.x * 2.0,
                cuboid.half_extents.y * 2.0,
                cuboid.half_extents.z * 2.0,
            );
            gizmos.cuboid(*transform, color);
        }
        bevy_rapier3d::prelude::ColliderShape::Capsule(capsule) => {
            // Draw capsule as cylinder wireframe
            gizmos.cylinder(
                transform.translation,
                transform.rotation,
                capsule.half_height * 2.0,
                capsule.radius,
                color,
            );
        }
        _ => {
            // Draw generic sphere for other shapes
            gizmos.sphere(transform.translation, transform.rotation, 1.0, color);
        }
    }
}

/// Factory methods using consistent mesh-collider creation
pub struct ConsistentVehicleFactory;

impl ConsistentVehicleFactory {
    pub fn create_super_car(
        config: &VehicleTypeConfig,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> (Handle<Mesh>, Collider) {
        let mesh_config = MeshColliderConfig::new_cuboid(config.collider_size * 2.0, 1.0);
        mesh_config.create_pair(meshes)
    }

    pub fn create_helicopter(
        config: &VehicleTypeConfig,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> (Handle<Mesh>, Collider) {
        let mesh_config = MeshColliderConfig::new_cuboid(config.collider_size * 2.0, 1.0);
        mesh_config.create_pair(meshes)
    }

    pub fn create_f16(
        config: &VehicleTypeConfig,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> (Handle<Mesh>, Collider) {
        let mesh_config = MeshColliderConfig::new_capsule(config.collider_size * 2.0, 1.0);
        mesh_config.create_pair(meshes)
    }
}
