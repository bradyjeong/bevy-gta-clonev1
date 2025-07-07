//! ───────────────────────────────────────────────
//! System:   Road Mesh Generation
//! Purpose:  Generates road meshes from splines
//! Schedule: On demand
//! Reads:    RoadSpline, RoadType
//! Writes:   Mesh assets
//! Owner:    @rendering-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::collections::HashMap;
use crate::systems::world::road_network::RoadSpline;
use game_core::prelude::*;

#[derive(Resource, Default)]
pub struct RoadMeshCache {
    pub road_meshes: HashMap<String, Handle<Mesh>>,
}

pub fn generate_road_mesh_cached(
    road: &RoadSpline,
    meshes: &mut ResMut<Assets<Mesh>>,
    mut mesh_cache: ResMut<RoadMeshCache>,
) -> Handle<Mesh> {
    let cache_key = format!("road_{}_{}", road.id, road.control_points.len());
    
    if let Some(handle) = mesh_cache.road_meshes.get(&cache_key) {
        return handle.clone();
    }
    
    let mesh = generate_road_mesh(road);
    let handle = meshes.add(mesh);
    
    // Store in cache (with size limit to prevent memory leaks)
    if mesh_cache.road_meshes.len() < 500 { // Limit cache size
        mesh_cache.road_meshes.insert(cache_key, handle.clone());
    }
    
    handle
}

pub fn generate_road_mesh(road: &RoadSpline) -> Mesh {
    generate_road_mesh_internal(road)
}

fn generate_road_mesh_internal(road: &RoadSpline) -> Mesh {
    let width = road.road_type.width();
    let segments = calculate_segments(road);
    
    // Pre-allocate with exact capacity to avoid reallocations
    let vertex_count = (segments + 1) * 2;
    let index_count = segments * 6;
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(index_count);
    
    // Generate vertices along the spline
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let position = road.evaluate(t);
        let direction = road.direction_at(t);
        
        // Calculate perpendicular vector for road width
        let up = Vec3::Y;
        let right = direction.cross(up).normalize();
        
        // Create left and right vertices
        let left = position - right * width * 0.5;
        let right = position + right * width * 0.5;
        
        vertices.push([left.x, left.y, left.z]);
        vertices.push([right.x, right.y, right.z]);
        
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        
        // UV coordinates
        let u_left = 0.0;
        let u_right = 1.0;
        let v = t;
        
        uvs.push([u_left, v]);
        uvs.push([u_right, v]);
    }
    
    // Generate indices for triangles
    for i in 0..segments {
        let base = i * 2;
        
        // First triangle
        indices.push(base as u32);
        indices.push((base + 1) as u32);
        indices.push((base + 2) as u32);
        
        // Second triangle
        indices.push((base + 1) as u32);
        indices.push((base + 3) as u32);
        indices.push((base + 2) as u32);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    
    mesh
}

fn calculate_segments(road: &RoadSpline) -> usize {
    let length = road.length();
    let base_segments = (length / 5.0).ceil() as usize;
    base_segments.clamp(10, 100)
}

pub fn generate_road_markings_mesh(road: &RoadSpline) -> Vec<Mesh> {
    let mut markings = Vec::new();
    
    match road.road_type {
        RoadType::Highway => {
            markings.push(generate_center_line_mesh(road));
            markings.push(generate_lane_divider_mesh(road));
        }
        RoadType::MainStreet => {
            markings.push(generate_center_line_mesh(road));
        }
        RoadType::SideStreet => {
            markings.push(generate_dashed_center_line_mesh(road));
        }
        RoadType::Alley => {
            // No markings for alleys
        }
    }
    
    markings
}

fn generate_center_line_mesh(road: &RoadSpline) -> Mesh {
    let segments = calculate_segments(road);
    let line_width = 0.1;
    
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let position = road.evaluate(t);
        let direction = road.direction_at(t);
        
        let up = Vec3::Y;
        let right = direction.cross(up).normalize();
        
        let left = position - right * line_width * 0.5;
        let right = position + right * line_width * 0.5;
        
        vertices.push([left.x, left.y + 0.01, left.z]);
        vertices.push([right.x, right.y + 0.01, right.z]);
        
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        
        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
    }
    
    for i in 0..segments {
        let base = i * 2;
        
        indices.push(base as u32);
        indices.push((base + 1) as u32);
        indices.push((base + 2) as u32);
        
        indices.push((base + 1) as u32);
        indices.push((base + 3) as u32);
        indices.push((base + 2) as u32);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    
    mesh
}

fn generate_lane_divider_mesh(road: &RoadSpline) -> Mesh {
    let segments = calculate_segments(road);
    let line_width = 0.05;
    let lane_offset = road.road_type.width() * 0.25;
    
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let position = road.evaluate(t);
        let direction = road.direction_at(t);
        
        let up = Vec3::Y;
        let right = direction.cross(up).normalize();
        
        // Left lane divider
        let left_center = position - right * lane_offset;
        let left_left = left_center - right * line_width * 0.5;
        let left_right = left_center + right * line_width * 0.5;
        
        vertices.push([left_left.x, left_left.y + 0.01, left_left.z]);
        vertices.push([left_right.x, left_right.y + 0.01, left_right.z]);
        
        // Right lane divider
        let right_center = position + right * lane_offset;
        let right_left = right_center - right * line_width * 0.5;
        let right_right = right_center + right * line_width * 0.5;
        
        vertices.push([right_left.x, right_left.y + 0.01, right_left.z]);
        vertices.push([right_right.x, right_right.y + 0.01, right_right.z]);
        
        for _ in 0..4 {
            normals.push([0.0, 1.0, 0.0]);
        }
        
        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
    }
    
    for i in 0..segments {
        let base = i * 4;
        
        // Left lane divider
        indices.push(base as u32);
        indices.push((base + 1) as u32);
        indices.push((base + 4) as u32);
        
        indices.push((base + 1) as u32);
        indices.push((base + 5) as u32);
        indices.push((base + 4) as u32);
        
        // Right lane divider
        indices.push((base + 2) as u32);
        indices.push((base + 3) as u32);
        indices.push((base + 6) as u32);
        
        indices.push((base + 3) as u32);
        indices.push((base + 7) as u32);
        indices.push((base + 6) as u32);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    
    mesh
}

fn generate_dashed_center_line_mesh(road: &RoadSpline) -> Mesh {
    let segments = calculate_segments(road);
    let line_width = 0.1;
    let dash_length = 3.0;
    let gap_length = 2.0;
    let pattern_length = dash_length + gap_length;
    
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    let road_length = road.length();
    let mut current_distance = 0.0;
    let mut vertex_index = 0;
    
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let position = road.evaluate(t);
        let direction = road.direction_at(t);
        
        let segment_distance = if i > 0 {
            let prev_t = (i - 1) as f32 / segments as f32;
            let prev_position = road.evaluate(prev_t);
            position.distance(prev_position)
        } else {
            0.0
        };
        
        current_distance += segment_distance;
        
        let pattern_position = current_distance % pattern_length;
        let in_dash = pattern_position < dash_length;
        
        if in_dash {
            let up = Vec3::Y;
            let right = direction.cross(up).normalize();
            
            let left = position - right * line_width * 0.5;
            let right = position + right * line_width * 0.5;
            
            vertices.push([left.x, left.y + 0.01, left.z]);
            vertices.push([right.x, right.y + 0.01, right.z]);
            
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            
            uvs.push([0.0, t]);
            uvs.push([1.0, t]);
            
            if vertex_index > 0 {
                let base = vertex_index - 2;
                
                indices.push(base as u32);
                indices.push((base + 1) as u32);
                indices.push((base + 2) as u32);
                
                indices.push((base + 1) as u32);
                indices.push((base + 3) as u32);
                indices.push((base + 2) as u32);
            }
            
            vertex_index += 2;
        }
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    if !vertices.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
    }
    
    mesh
}
