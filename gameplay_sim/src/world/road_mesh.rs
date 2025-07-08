//! ───────────────────────────────────────────────
//! System:   Road Mesh
//! Purpose:  Manages world state and generation
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::systems::world::road_network::{RoadSpline, RoadType, IntersectionType, RoadIntersection};
use game_core::prelude::MeshCache;

// PROPER ROAD MESH GENERATION (Like GTA) - OPTIMIZED WITH CACHING

pub fn generate_road_mesh_cached(
    road: &RoadSpline, 
    mesh_cache: &mut MeshCache, 
    meshes: &mut Assets<Mesh>
) -> Handle<Mesh> {
    // Create cache key based on road properties
    let cache_key = format!("{:?}_{}_{}_{}", 
        road.road_type, 
        road.points.len(),
        (road.length() * 10.0) as u32, // Discretized length
        road.points.iter()
            .map(|p| format!("{:.1}_{:.1}_{:.1}", p.x, p.y, p.z))
            .collect::<Vec<_>>()
            .join("_")
    );
    
    // Check if mesh already exists in cache
    if let Some(handle) = mesh_cache.road_meshes.get(&cache_key) {
        return handle.clone();
    }
    
    // Generate new mesh if not in cache
    let mesh = generate_road_mesh_internal(road);
    let handle = meshes.add(mesh);
    
    // Store in cache (with size limit to prevent memory leaks)
    if mesh_cache.road_meshes.len() < 500 { // Limit cache size
        mesh_cache.road_meshes.insert(cache_key, handle.clone());
    }
    
    handle
}

#[must_use] pub fn generate_road_mesh(road: &RoadSpline) -> Mesh {
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
        let tangent = calculate_tangent(road, t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize();
        
        // Left and right edge of road
        let left_pos = position + right * width * 0.5;
        let right_pos = position - right * width * 0.5;
        
        // Add vertices (left, right)
        vertices.push([left_pos.x, left_pos.y, left_pos.z]);
        vertices.push([right_pos.x, right_pos.y, right_pos.z]);
        
        // Add normals (pointing up)
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        
        // Add UVs for texture mapping
        let v = t;
        uvs.push([0.0, v]);
        uvs.push([1.0, v]);
        
        // Generate triangles (except for last segment)
        if i < segments {
            let base = (i * 2) as u32;
            
            // First triangle (counter-clockwise for front face)
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            
            // Second triangle (counter-clockwise for front face)
            indices.push(base + 1);
            indices.push(base + 3);
            indices.push(base + 2);
        }
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

#[must_use] pub fn generate_road_markings_mesh(road: &RoadSpline) -> Vec<Mesh> {
    let mut markings = Vec::new();
    
    match road.road_type {
        RoadType::Highway => {
            // Multiple lanes with dashed lines
            markings.push(generate_center_line_mesh(road, true)); // Dashed
            markings.push(generate_lane_markings_mesh(road, 4)); // 4 lanes
        },
        RoadType::MainStreet => {
            // Center line + edge lines
            markings.push(generate_center_line_mesh(road, true)); // Dashed
            markings.push(generate_edge_lines_mesh(road));
        },
        RoadType::SideStreet => {
            // Simple center line
            markings.push(generate_center_line_mesh(road, false)); // Solid
        },
        RoadType::Alley => {
            // No markings for alleys
        }
    }
    
    markings
}

fn calculate_segments(road: &RoadSpline) -> usize {
    // Fewer segments for better performance
    let length = road.length();
    let base_segments = (length / 20.0) as usize; // Segment every 20 units (was 5)
    
    if road.points.len() > 2 {
        // Curved road - reduced segments
        (base_segments).max(4).min(30) // Much fewer segments
    } else {
        // Straight road - minimal segments
        base_segments.max(2).min(8) // Much fewer segments
    }
}

fn calculate_tangent(road: &RoadSpline, t: f32) -> Vec3 {
    let epsilon = 0.01;
    let t1 = (t - epsilon).max(0.0);
    let t2 = (t + epsilon).min(1.0);
    
    let p1 = road.evaluate(t1);
    let p2 = road.evaluate(t2);
    
    (p2 - p1).normalize()
}

fn generate_center_line_mesh(road: &RoadSpline, dashed: bool) -> Mesh {
    let segments = calculate_segments(road);
    let line_width = 0.3;
    
    // Pre-allocate for dashed lines (estimate half capacity for dashed)
    let estimated_segments = if dashed { segments / 2 } else { segments };
    let vertex_count = (estimated_segments + 1) * 2;
    let index_count = estimated_segments * 6;
    
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(index_count);
    
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        
        // Skip segments for dashed line
        if dashed && (i / 5) % 2 == 1 {
            continue;
        }
        
        let position = road.evaluate(t);
        let tangent = calculate_tangent(road, t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize();
        
        // Center line vertices
        let left_pos = position + right * line_width * 0.5;
        let right_pos = position - right * line_width * 0.5;
        
        let base_idx = vertices.len() as u32;
        
        vertices.push([left_pos.x, left_pos.y, left_pos.z]); // At road surface level
        vertices.push([right_pos.x, right_pos.y, right_pos.z]);
        
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        
        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
        
        // Generate triangles for line segments
        if vertices.len() >= 4 && base_idx >= 2 {
            indices.push(base_idx - 2);
            indices.push(base_idx);
            indices.push(base_idx - 1);
            
            indices.push(base_idx - 1);
            indices.push(base_idx);
            indices.push(base_idx + 1);
        }
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

fn generate_lane_markings_mesh(road: &RoadSpline, lanes: u32) -> Mesh {
    // Generate lane divider lines for multi-lane roads
    let width = road.road_type.width();
    let lane_width = width / lanes as f32;
    let segments = calculate_segments(road);
    
    // Pre-allocate for lane markings (lanes-1 dividers, segments/2 for dashed)
    let lane_dividers = (lanes - 1) as usize;
    let estimated_segments = segments / 2; // Dashed lines
    let vertex_count = lane_dividers * estimated_segments * 2;
    let index_count = lane_dividers * estimated_segments * 6;
    
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(index_count);
    
    // Generate markings between lanes (skip center and edges)
    for lane in 1..lanes {
        let lane_offset = (lane as f32 - lanes as f32 * 0.5) * lane_width;
        
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            
            // Dashed lines between lanes
            if (i / 3) % 2 == 1 {
                continue;
            }
            
            let position = road.evaluate(t);
            let tangent = calculate_tangent(road, t);
            let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize();
            
            let line_pos = position + right * lane_offset;
            let line_width = 0.2;
            
            let left_pos = line_pos + right * line_width * 0.5;
            let right_pos = line_pos - right * line_width * 0.5;
            
            let base_idx = vertices.len() as u32;
            
            vertices.push([left_pos.x, left_pos.y, left_pos.z]);
            vertices.push([right_pos.x, right_pos.y, right_pos.z]);
            
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            
            uvs.push([0.0, t]);
            uvs.push([1.0, t]);
            
            if vertices.len() >= 4 && base_idx >= 2 {
                indices.push(base_idx - 2);
                indices.push(base_idx);
                indices.push(base_idx - 1);
                
                indices.push(base_idx - 1);
                indices.push(base_idx);
                indices.push(base_idx + 1);
            }
        }
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

fn generate_edge_lines_mesh(road: &RoadSpline) -> Mesh {
    // Solid white lines at road edges
    let width = road.road_type.width();
    let segments = calculate_segments(road);
    let line_width = 0.15;
    
    // Pre-allocate for edge lines (2 edges, solid lines)
    let vertex_count = (segments + 1) * 4; // 4 vertices per segment (2 edges)
    let index_count = segments * 12; // 12 indices per segment (2 edges * 6 indices each)
    
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(index_count);
    
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let position = road.evaluate(t);
        let tangent = calculate_tangent(road, t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize();
        
        // Left edge line
        let left_center = position + right * (width * 0.5 - 0.5);
        let left_inner = left_center - right * line_width * 0.5;
        let left_outer = left_center + right * line_width * 0.5;
        
        // Right edge line  
        let right_center = position - right * (width * 0.5 - 0.5);
        let right_inner = right_center + right * line_width * 0.5;
        let right_outer = right_center - right * line_width * 0.5;
        
        let base_idx = vertices.len() as u32;
        
        // Add vertices for both edge lines
        vertices.push([left_inner.x, left_inner.y, left_inner.z]);
        vertices.push([left_outer.x, left_outer.y, left_outer.z]);
        vertices.push([right_inner.x, right_inner.y, right_inner.z]);
        vertices.push([right_outer.x, right_outer.y, right_outer.z]);
        
        for _ in 0..4 {
            normals.push([0.0, 1.0, 0.0]);
        }
        
        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
        
        if i > 0 {
            // Left edge line triangles
            indices.push(base_idx - 4);
            indices.push(base_idx);
            indices.push(base_idx - 3);
            
            indices.push(base_idx - 3);
            indices.push(base_idx);
            indices.push(base_idx + 1);
            
            // Right edge line triangles
            indices.push(base_idx - 2);
            indices.push(base_idx + 2);
            indices.push(base_idx - 1);
            
            indices.push(base_idx - 1);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

#[must_use] pub fn generate_intersection_mesh(intersection: &RoadIntersection, connected_roads: &[&RoadSpline]) -> Mesh {
    match intersection.intersection_type {
        IntersectionType::Cross => generate_cross_intersection_mesh(intersection, connected_roads),
        IntersectionType::TJunction => generate_t_intersection_mesh(intersection, connected_roads),
        IntersectionType::Curve => generate_curved_intersection_mesh(intersection, connected_roads),
        IntersectionType::HighwayOnramp => generate_onramp_mesh(intersection, connected_roads),
        IntersectionType::CrossRoads => generate_cross_intersection_mesh(intersection, connected_roads),
        IntersectionType::Roundabout => generate_curved_intersection_mesh(intersection, connected_roads),
    }
}

fn generate_cross_intersection_mesh(intersection: &RoadIntersection, _connected_roads: &[&RoadSpline]) -> Mesh {
    let radius = intersection.radius;
    let segments = 16; // Circle segments
    
    // Pre-allocate for intersection mesh (center + circle vertices)
    let vertex_count = segments + 1; // Center + circle vertices
    let index_count = segments * 3; // Triangle fan from center
    
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(index_count);
    
    // Center vertex
    vertices.push([intersection.position.x, intersection.position.y, intersection.position.z]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    
    // Circle vertices
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = intersection.position.x + radius * angle.cos();
        let z = intersection.position.z + radius * angle.sin();
        
        vertices.push([x, intersection.position.y, z]);
        normals.push([0.0, 1.0, 0.0]);
        
        let u = 0.5 + 0.5 * angle.cos();
        let v = 0.5 + 0.5 * angle.sin();
        uvs.push([u, v]);
        
        // Triangle from center to edge
        let next_i = (i + 1) % segments;
        indices.push(0); // Center
        indices.push((i + 1) as u32);
        indices.push((next_i + 1) as u32);
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

fn generate_t_intersection_mesh(intersection: &RoadIntersection, _connected_roads: &[&RoadSpline]) -> Mesh {
    // Simplified T-intersection - could be more sophisticated
    generate_cross_intersection_mesh(intersection, _connected_roads)
}

fn generate_curved_intersection_mesh(intersection: &RoadIntersection, _connected_roads: &[&RoadSpline]) -> Mesh {
    // Curved connection between two roads
    generate_cross_intersection_mesh(intersection, _connected_roads)
}

fn generate_onramp_mesh(intersection: &RoadIntersection, _connected_roads: &[&RoadSpline]) -> Mesh {
    // Highway onramp with proper merging geometry
    generate_cross_intersection_mesh(intersection, _connected_roads)
}
