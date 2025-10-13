use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Create sloped beach terrain that transitions from land to water
/// Professional games use gradual slopes to create realistic coastlines
pub fn create_beach_slope(
    width: f32,
    depth: f32,
    start_height: f32,
    end_height: f32,
    subdivisions: u32,
) -> Mesh {
    let subdivisions = subdivisions.clamp(2, 256);

    let num_vertices = (subdivisions + 1) * (subdivisions + 1);
    let num_indices = subdivisions * subdivisions * 6;

    let mut positions = Vec::with_capacity(num_vertices as usize);
    let mut normals = Vec::with_capacity(num_vertices as usize);
    let mut uvs = Vec::with_capacity(num_vertices as usize);
    let mut indices = Vec::with_capacity(num_indices as usize);

    // Generate vertices with gradual height transition
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let x_pos = (x as f32 / subdivisions as f32 - 0.5) * width;
            let z_pos = (z as f32 / subdivisions as f32 - 0.5) * depth;

            // Calculate height transition along X axis (land to water)
            let t = x as f32 / subdivisions as f32; // 0 = back (land), 1 = front (water)

            // Use smoothstep for natural looking slope
            let smooth_t = smoothstep(t);
            let height = start_height + (end_height - start_height) * smooth_t;

            positions.push([x_pos, height, z_pos]);

            // Calculate normal based on slope (along X axis)
            let slope_angle = (end_height - start_height) / width;
            let normal = Vec3::new(-slope_angle, 1.0, 0.0).normalize();
            normals.push([normal.x, normal.y, normal.z]);

            uvs.push([
                x as f32 / subdivisions as f32,
                z as f32 / subdivisions as f32,
            ]);
        }
    }

    // Generate indices (two triangles per quad)
    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let i0 = z * (subdivisions + 1) + x;
            let i1 = i0 + 1;
            let i2 = i0 + subdivisions + 1;
            let i3 = i2 + 1;

            // First triangle
            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            // Second triangle
            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Smoothstep function for natural terrain transitions
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Create circular beach around a lake
pub fn create_circular_beach_ring(
    inner_radius: f32,
    outer_radius: f32,
    center: Vec3,
    land_height: f32,
    water_height: f32,
    radial_segments: u32,
    height_segments: u32,
) -> Mesh {
    let radial_segments = radial_segments.clamp(8, 64);
    let height_segments = height_segments.clamp(2, 32);

    let num_vertices = (radial_segments + 1) * (height_segments + 1);
    let num_indices = radial_segments * height_segments * 6;

    let mut positions = Vec::with_capacity(num_vertices as usize);
    let mut normals = Vec::with_capacity(num_vertices as usize);
    let mut uvs = Vec::with_capacity(num_vertices as usize);
    let mut indices = Vec::with_capacity(num_indices as usize);

    // Generate ring vertices
    for h in 0..=height_segments {
        let t = h as f32 / height_segments as f32;
        let radius = inner_radius + (outer_radius - inner_radius) * t;

        // Smoothstep for height transition
        let smooth_t = smoothstep(t);
        let height = water_height + (land_height - water_height) * smooth_t;

        for r in 0..=radial_segments {
            let angle = (r as f32 / radial_segments as f32) * std::f32::consts::TAU;

            let x = center.x + radius * angle.cos();
            let z = center.z + radius * angle.sin();

            positions.push([x, height, z]);

            // Normal points outward and slightly up
            let normal = Vec3::new(0.0, 0.8, 0.0).normalize();
            normals.push([normal.x, normal.y, normal.z]);

            uvs.push([r as f32 / radial_segments as f32, t]);
        }
    }

    // Generate indices
    for h in 0..height_segments {
        for r in 0..radial_segments {
            let i0 = h * (radial_segments + 1) + r;
            let i1 = i0 + 1;
            let i2 = i0 + radial_segments + 1;
            let i3 = i2 + 1;

            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}
