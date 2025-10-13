use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Create a subdivided plane mesh for water surfaces
/// Higher subdivisions = more vertices = better wave detail
pub fn create_subdivided_plane(width: f32, height: f32, subdivisions: u32) -> Mesh {
    let subdivisions = subdivisions.clamp(1, 512); // Safety limits

    let num_vertices = (subdivisions + 1) * (subdivisions + 1);
    let num_indices = subdivisions * subdivisions * 6;

    let mut positions = Vec::with_capacity(num_vertices as usize);
    let mut normals = Vec::with_capacity(num_vertices as usize);
    let mut uvs = Vec::with_capacity(num_vertices as usize);
    let mut indices = Vec::with_capacity(num_indices as usize);

    // Generate vertices
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let x_pos = (x as f32 / subdivisions as f32 - 0.5) * width;
            let z_pos = (z as f32 / subdivisions as f32 - 0.5) * height;

            positions.push([x_pos, 0.0, z_pos]);
            normals.push([0.0, 1.0, 0.0]);
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
