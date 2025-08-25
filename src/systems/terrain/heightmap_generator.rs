use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use noise::{NoiseFn, Perlin};
use super::asset_based_terrain::TerrainConfig;
use crate::constants::{STATIC_GROUP, VEHICLE_GROUP, CHARACTER_GROUP};
use crate::components::DynamicTerrain;

/// Simple heightmap-based terrain generation system
/// 
/// Design principles:
/// - Simple Perlin noise implementation (not complex FFT/spectral methods)
/// - Single responsibility: Generate terrain heights from config
/// - Minimal coupling: Works with existing terrain service
/// - Performance: 512x512 resolution for balance
/// - Asset-driven: Uses terrain.ron configuration
pub struct HeightmapTerrainMesh {
    pub mesh: Mesh,
    pub heights: Vec<Vec<f32>>,
    pub resolution: u32,
    pub world_size: f32,
}

impl HeightmapTerrainMesh {
    /// Generate heightmap and mesh from terrain configuration
    pub fn from_config(config: &TerrainConfig) -> Self {
        let heights = generate_heightmap(config);
        let mesh = generate_mesh_from_heightmap(&heights, config);
        
        Self {
            mesh,
            heights,
            resolution: config.resolution,
            world_size: config.world_size,
        }
    }
    
    /// Get height at specific grid coordinates (for physics)
    pub fn get_height_at_grid(&self, x: usize, z: usize) -> f32 {
        if x >= self.heights.len() || z >= self.heights[0].len() {
            return 0.0;
        }
        self.heights[x][z]
    }
    
    /// Get height at world position using bilinear interpolation
    pub fn get_height_at_world(&self, x: f32, z: f32) -> f32 {
        // Convert world coordinates to grid coordinates
        let half_size = self.world_size * 0.5;
        let grid_x = (x + half_size) / self.world_size * (self.resolution - 1) as f32;
        let grid_z = (z + half_size) / self.world_size * (self.resolution - 1) as f32;
        
        // Clamp to valid range
        if grid_x < 0.0 || grid_x >= (self.resolution - 1) as f32 || 
           grid_z < 0.0 || grid_z >= (self.resolution - 1) as f32 {
            return 0.0;
        }
        
        // Bilinear interpolation
        let x0 = grid_x.floor() as usize;
        let x1 = (x0 + 1).min(self.resolution as usize - 1);
        let z0 = grid_z.floor() as usize;
        let z1 = (z0 + 1).min(self.resolution as usize - 1);
        
        let fx = grid_x - x0 as f32;
        let fz = grid_z - z0 as f32;
        
        let h00 = self.get_height_at_grid(x0, z0);
        let h10 = self.get_height_at_grid(x1, z0);
        let h01 = self.get_height_at_grid(x0, z1);
        let h11 = self.get_height_at_grid(x1, z1);
        
        // Bilinear interpolation formula
        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;
        h0 * (1.0 - fz) + h1 * fz
    }
}

/// Generate heightmap using simple Perlin noise
#[allow(clippy::needless_range_loop)]
pub fn generate_heightmap(config: &TerrainConfig) -> Vec<Vec<f32>> {
    let perlin = Perlin::new(config.noise_seed);
    let resolution = config.resolution as usize;
    let mut heights = vec![vec![0.0; resolution]; resolution];
    
    info!("Generating {}x{} heightmap with seed {}", resolution, resolution, config.noise_seed);
    
    // Generate base terrain using noise
    for x in 0..resolution {
        for z in 0..resolution {
            // Convert grid coordinates to world coordinates
            let world_x = (x as f32 / (resolution - 1) as f32 - 0.5) * config.world_size;
            let world_z = (z as f32 / (resolution - 1) as f32 - 0.5) * config.world_size;
            
            // Generate height using fractal noise
            let mut height = 0.0;
            let mut amplitude = config.hill_scale;
            let mut frequency = config.generation.noise_frequency;
            
            for _ in 0..config.generation.noise_octaves {
                height += perlin.get([world_x as f64 * frequency as f64, world_z as f64 * frequency as f64]) as f32 * amplitude;
                amplitude *= config.generation.noise_persistence;
                frequency *= config.generation.noise_lacunarity;
            }
            
            // Add base height
            height += config.base_height;
            
            // Apply edge falloff to prevent steep cliffs at world boundaries
            if config.generation.edge_falloff > 0.0 {
                let edge_dist = (resolution as f32 * 0.5 - ((x as f32 - resolution as f32 * 0.5).abs().max(z as f32 - resolution as f32 * 0.5))).max(0.0);
                let falloff_factor = (edge_dist / config.generation.edge_falloff).clamp(0.0, 1.0);
                height = config.base_height + (height - config.base_height) * falloff_factor;
            }
            
            // Apply water basin carving
            height = carve_water_basins_at_point(world_x, world_z, height, config);
            
            heights[x][z] = height;
        }
    }
    
    // Apply terrain smoothing if enabled
    if config.generation.terrain_smoothing {
        heights = smooth_heightmap(heights);
    }
    
    info!("Heightmap generation complete");
    heights
}

/// Phase 4: Water basin carving for natural water basins
/// 
/// Creates smooth circular depressions in terrain for natural water areas.
/// Uses linear falloff from edge to center for realistic shoreline slopes.
/// 
/// Design principles:
/// - Simple linear falloff algorithm for predictable results
/// - Walkable shoreline slopes (not too steep)
/// - Natural integration with existing terrain noise
/// - Multiple overlapping water areas supported
pub fn carve_water_basins_at_point(world_x: f32, world_z: f32, base_height: f32, config: &TerrainConfig) -> f32 {
    let mut final_height = base_height;
    
    // Apply carving for each water area (supports overlapping basins)
    for water in &config.water_areas {
        let distance = ((world_x - water.center.0).powi(2) + (world_z - water.center.1).powi(2)).sqrt();
        
        if distance <= water.radius {
            // Linear falloff from edge to center for natural slopes
            let edge_factor = (water.radius - distance) / water.radius;
            let edge_factor_smoothed = edge_factor.clamp(0.0, 1.0);
            
            // Target depth at this point (linear interpolation)
            let target_depth = water.depth * edge_factor_smoothed;
            
            // Create smooth transition to water depth
            // Ensure we don't raise terrain above original height
            let carved_height = final_height + (target_depth - final_height) * edge_factor_smoothed;
            final_height = carved_height.min(final_height);
        }
    }
    
    final_height
}

/// Simple heightmap smoothing using 3x3 kernel
#[allow(clippy::needless_range_loop)]
fn smooth_heightmap(heights: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let resolution = heights.len();
    let mut smoothed = heights.clone();
    
    for x in 1..(resolution - 1) {
        for z in 1..(resolution - 1) {
            let mut sum = 0.0;
            let mut count = 0;
            
            // 3x3 smoothing kernel
            for dx in -1..=1 {
                for dz in -1..=1 {
                    let nx = (x as i32 + dx) as usize;
                    let nz = (z as i32 + dz) as usize;
                    if nx < resolution && nz < resolution {
                        sum += heights[nx][nz];
                        count += 1;
                    }
                }
            }
            
            smoothed[x][z] = sum / count as f32;
        }
    }
    
    smoothed
}

/// Generate triangulated mesh from heightmap
#[allow(clippy::needless_range_loop)]
pub fn generate_mesh_from_heightmap(heights: &[Vec<f32>], config: &TerrainConfig) -> Mesh {
    let resolution = heights.len();
    let world_size = config.world_size;
    let _half_size = world_size * 0.5;
    
    info!("Generating mesh from heightmap: {} vertices", resolution * resolution);
    
    // Generate vertices
    let mut vertices = Vec::with_capacity(resolution * resolution);
    let mut uvs = Vec::with_capacity(resolution * resolution);
    
    for x in 0..resolution {
        for z in 0..resolution {
            // World position
            let world_x = (x as f32 / (resolution - 1) as f32 - 0.5) * world_size;
            let world_z = (z as f32 / (resolution - 1) as f32 - 0.5) * world_size;
            let world_y = heights[x][z];
            
            vertices.push([world_x, world_y, world_z]);
            
            // UV coordinates
            uvs.push([x as f32 / (resolution - 1) as f32, z as f32 / (resolution - 1) as f32]);
        }
    }
    
    // Generate triangular indices
    let mut indices = Vec::new();
    for x in 0..(resolution - 1) {
        for z in 0..(resolution - 1) {
            let i0 = x * resolution + z;
            let i1 = (x + 1) * resolution + z;
            let i2 = x * resolution + (z + 1);
            let i3 = (x + 1) * resolution + (z + 1);
            
            // Two triangles per quad
            indices.extend_from_slice(&[i0 as u32, i1 as u32, i2 as u32]);
            indices.extend_from_slice(&[i1 as u32, i3 as u32, i2 as u32]);
        }
    }
    
    // Calculate normals
    let normals = calculate_normals(&vertices, &indices);
    
    info!("Generated mesh: {} vertices, {} triangles", vertices.len(), indices.len() / 3);
    
    // Create Bevy mesh
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    mesh
}

/// Calculate smooth normals for terrain mesh
fn calculate_normals(vertices: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; vertices.len()];
    
    // Calculate face normals and accumulate at vertices
    for triangle in indices.chunks(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;
        
        let v0 = Vec3::from_array(vertices[i0]);
        let v1 = Vec3::from_array(vertices[i1]);
        let v2 = Vec3::from_array(vertices[i2]);
        
        let normal = (v1 - v0).cross(v2 - v0).normalize();
        
        // Accumulate normal at each vertex
        for &idx in &[i0, i1, i2] {
            normals[idx][0] += normal.x;
            normals[idx][1] += normal.y;
            normals[idx][2] += normal.z;
        }
    }
    
    // Normalize accumulated normals
    for normal in &mut normals {
        let length = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if length > 0.0 {
            normal[0] /= length;
            normal[1] /= length;
            normal[2] /= length;
        } else {
            normal[1] = 1.0; // Default to up vector
        }
    }
    
    normals
}

/// Create Rapier heightfield collider from heightmap
pub fn create_heightfield_collider(heightmap_mesh: &HeightmapTerrainMesh) -> Collider {
    let resolution = heightmap_mesh.resolution as usize;
    let world_size = heightmap_mesh.world_size;
    
    // Convert 2D heights to 1D array for Rapier
    let mut heights = Vec::with_capacity(resolution * resolution);
    for z in 0..resolution {
        for x in 0..resolution {
            heights.push(heightmap_mesh.heights[x][z]);
        }
    }
    
    // Create heightfield with proper scale
    let scale = world_size / (resolution - 1) as f32;
    let heightfield = Collider::heightfield(
        heights,
        resolution,
        resolution,
        Vec3::new(scale, 1.0, scale),
    );
    
    info!("Created heightfield collider: {}x{} resolution, {:.2}m scale", 
          resolution, resolution, scale);
    
    heightfield
}

/// Spawn heightmap-based terrain entity to replace flat plane
pub fn spawn_heightmap_terrain(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &TerrainConfig,
) -> Entity {
    info!("Spawning heightmap-generated terrain");
    
    // Generate heightmap and mesh
    let heightmap_mesh = HeightmapTerrainMesh::from_config(config);
    
    // Create heightfield collider
    let collider = create_heightfield_collider(&heightmap_mesh);
    
    // Spawn terrain entity
    let terrain_entity = commands.spawn((
        DynamicTerrain,
        Mesh3d(meshes.add(heightmap_mesh.mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))), // Sandy terrain color
        Transform::from_xyz(0.0, 0.0, 0.0), // Terrain at world origin
        RigidBody::Fixed,
        collider,
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
    )).id();
    
    info!("Heightmap terrain spawned successfully");
    terrain_entity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heightmap_generation() {
        let config = TerrainConfig {
            resolution: 64,
            world_size: 1000.0,
            hill_scale: 10.0,
            noise_seed: 12345,
            ..Default::default()
        };
        
        let heights = generate_heightmap(&config);
        assert_eq!(heights.len(), 64);
        assert_eq!(heights[0].len(), 64);
        
        // Check that heights are reasonable
        for row in &heights {
            for &height in row {
                assert!(height > -100.0 && height < 100.0, "Height {} is unreasonable", height);
            }
        }
    }
    
    #[test]
    fn test_mesh_generation() {
        let config = TerrainConfig {
            resolution: 32,
            world_size: 500.0,
            ..Default::default()
        };
        
        let heights = generate_heightmap(&config);
        let mesh = generate_mesh_from_heightmap(&heights, &config);
        
        // Verify mesh has correct structure
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        assert!(mesh.indices().is_some());
    }
    
    #[test]
    fn test_height_interpolation() {
        let config = TerrainConfig {
            resolution: 32,
            world_size: 100.0,
            base_height: -0.15,
            ..Default::default()
        };
        
        let heightmap_mesh = HeightmapTerrainMesh::from_config(&config);
        
        // Test height query at origin
        let height = heightmap_mesh.get_height_at_world(0.0, 0.0);
        assert!(height > -50.0 && height < 50.0, "Height interpolation failed");
        
        // Test boundary conditions
        let boundary_height = heightmap_mesh.get_height_at_world(60.0, 60.0);
        assert_eq!(boundary_height, 0.0); // Should return 0 for out of bounds
    }
    
    #[test]
    fn test_water_basin_carving() {
        use super::super::asset_based_terrain::{TerrainConfig, WaterArea};
        
        let config = TerrainConfig {
            resolution: 64,
            world_size: 200.0,
            base_height: 0.0,
            water_areas: vec![
                WaterArea {
                    center: (0.0, 0.0),
                    radius: 20.0,
                    depth: -5.0,
                    description: "Test lake".to_string(),
                }
            ],
            ..Default::default()
        };
        
        // Test carving at water center
        let center_height = carve_water_basins_at_point(0.0, 0.0, 0.0, &config);
        assert!(center_height < 0.0, "Water center should be carved below base height");
        
        // Test carving at water edge
        let edge_height = carve_water_basins_at_point(20.0, 0.0, 0.0, &config);
        assert!(edge_height >= 0.0, "Water edge should remain at or above base height");
        
        // Test no carving outside water area
        let outside_height = carve_water_basins_at_point(30.0, 0.0, 0.0, &config);
        assert_eq!(outside_height, 0.0, "Outside water area should not be carved");
        
        // Test smooth transition - point halfway from center to edge
        let mid_height = carve_water_basins_at_point(10.0, 0.0, 0.0, &config);
        assert!(mid_height < 0.0 && mid_height > center_height, "Midpoint should show smooth transition");
    }
}
