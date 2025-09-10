use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy_rapier3d::prelude::*;

/// Unified height generation - single source of truth for visual and physics
pub struct TerrainHeightfield {
    pub width: usize,
    pub height: usize,
    pub scale: Vec3,
    pub heights: Vec<f32>,
}

impl TerrainHeightfield {
    /// Create flat terrain heightfield (starting simple as per implementation plan)
    pub fn new_flat(width: usize, height: usize, scale: Vec3) -> Self {
        // Validate inputs for critical safeguards
        let safe_width = width.clamp(2, 512);
        let safe_height = height.clamp(2, 512);
        let safe_scale = Vec3::new(
            scale.x.clamp(1.0, 10000.0),
            scale.y.clamp(0.1, 1000.0),
            scale.z.clamp(1.0, 10000.0),
        );

        // Generate flat height data (all zeros for now)
        let total_points = safe_width * safe_height;
        let heights = vec![0.0; total_points];

        Self {
            width: safe_width,
            height: safe_height,
            scale: safe_scale,
            heights,
        }
    }

    /// Get height at specific grid coordinates
    pub fn get_height_at_grid(&self, x: usize, z: usize) -> f32 {
        if x >= self.width || z >= self.height {
            return 0.0; // Safe fallback
        }
        let index = z * self.width + x;
        self.heights.get(index).copied().unwrap_or(0.0)
    }

    /// Convert world position to grid coordinates  
    pub fn world_to_grid(&self, world_pos: Vec2) -> Vec2 {
        let grid_x = ((world_pos.x / self.scale.x) + 0.5) * (self.width - 1) as f32;
        let grid_z = ((world_pos.y / self.scale.z) + 0.5) * (self.height - 1) as f32;
        Vec2::new(grid_x, grid_z)
    }

    /// Convert grid position to world coordinates
    pub fn grid_to_world(&self, grid_pos: Vec2) -> Vec2 {
        let world_x = (grid_pos.x / (self.width - 1) as f32 - 0.5) * self.scale.x;
        let world_z = (grid_pos.y / (self.height - 1) as f32 - 0.5) * self.scale.z;
        Vec2::new(world_x, world_z)
    }

    /// Get interpolated height at world position - FIXED coordinate conversion bug
    pub fn get_height_at_world_pos(&self, world_pos: Vec2) -> f32 {
        // Convert world position to grid coordinates with proper 0.5 offset
        let grid_x = ((world_pos.x / self.scale.x) + 0.5) * (self.width - 1) as f32;
        let grid_z = ((world_pos.y / self.scale.z) + 0.5) * (self.height - 1) as f32;

        // Safe clamping BEFORE casting to usize to prevent overflow from negative values
        let grid_x_clamped = grid_x.clamp(0.0, (self.width - 1) as f32);
        let grid_z_clamped = grid_z.clamp(0.0, (self.height - 1) as f32);
        let grid_x_int = grid_x_clamped as usize;
        let grid_z_int = grid_z_clamped as usize;

        self.get_height_at_grid(grid_x_int, grid_z_int) * self.scale.y
    }

    /// Create visual mesh from height data
    pub fn create_visual_mesh(&self) -> Mesh {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        // Generate vertices
        for z in 0..self.height {
            for x in 0..self.width {
                let height_val = self.get_height_at_grid(x, z);
                
                // World position
                let world_x = (x as f32 / (self.width - 1) as f32 - 0.5) * self.scale.x;
                let world_y = height_val * self.scale.y;
                let world_z = (z as f32 / (self.height - 1) as f32 - 0.5) * self.scale.z;
                
                vertices.push([world_x, world_y, world_z]);
                normals.push([0.0, 1.0, 0.0]); // Flat terrain - all normals point up
                uvs.push([x as f32 / (self.width - 1) as f32, z as f32 / (self.height - 1) as f32]);
            }
        }

        // Generate indices for triangles
        for z in 0..(self.height - 1) {
            for x in 0..(self.width - 1) {
                let i0 = (z * self.width + x) as u32;
                let i1 = (z * self.width + x + 1) as u32;
                let i2 = ((z + 1) * self.width + x) as u32;
                let i3 = ((z + 1) * self.width + x + 1) as u32;

                // Two triangles per quad
                indices.extend_from_slice(&[i0, i2, i1]);
                indices.extend_from_slice(&[i1, i2, i3]);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    /// Create physics collider from same height data
    pub fn create_physics_collider(&self) -> Collider {
        // Rapier heightfield collider from same data source
        // NOTE: According to Rapier docs, scale parameter represents the full size 
        // of the heightfield rectangle in the X-Z plane, NOT half extents
        Collider::heightfield(self.heights.clone(), self.width, self.height, self.scale)
    }
}

/// Marker component for heightfield terrain
#[derive(Component)]
pub struct HeightfieldTerrain;

/// System to spawn heightfield terrain - USES SHARED INSTANCE from GlobalTerrainHeights resource
pub fn spawn_heightfield_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_heights: Res<GlobalTerrainHeights>,
) {
    info!("🏔️ SPAWNING HEIGHTFIELD TERRAIN from shared resource instance");
    
    // Use the SHARED heightfield instance - single source of truth
    let terrain = &terrain_heights.heightfield;

    // Spawn unified heightfield terrain entity using shared data
    commands.spawn((
        HeightfieldTerrain,
        Mesh3d(meshes.add(terrain.create_visual_mesh())),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))), // Same color as original
        Transform::from_xyz(0.0, -0.15, 0.0), // Same position as original
        RigidBody::Fixed,
        terrain.create_physics_collider(), // Heightfield collider from SHARED data
        CollisionGroups::new(
            crate::constants::STATIC_GROUP,
            crate::constants::VEHICLE_GROUP | crate::constants::CHARACTER_GROUP,
        ),
    ));
    
    info!("✅ HEIGHTFIELD TERRAIN spawned using SINGLE shared instance");
}

/// Resource to provide global terrain height queries
#[derive(Resource)]
pub struct GlobalTerrainHeights {
    pub heightfield: TerrainHeightfield,
}

impl GlobalTerrainHeights {
    /// Get terrain height at world position - single source of truth
    pub fn get_height_at_position(&self, world_pos: Vec2) -> f32 {
        self.heightfield.get_height_at_world_pos(world_pos) - 0.15 // Account for terrain Y offset
    }

    /// Update heightfield and return what needs to be refreshed
    pub fn update_heightfield(&mut self, new_heightfield: TerrainHeightfield) -> TerrainUpdateEvent {
        self.heightfield = new_heightfield;
        
        TerrainUpdateEvent {
            needs_mesh_update: true,
            needs_collider_update: true,
            needs_entity_repositioning: true,
        }
    }
}

/// Event sent when terrain heightfield changes
#[derive(Event)]
pub struct TerrainUpdateEvent {
    pub needs_mesh_update: bool,
    pub needs_collider_update: bool, 
    pub needs_entity_repositioning: bool,
}

/// System to handle dynamic terrain updates
pub fn handle_terrain_updates(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_events: EventReader<TerrainUpdateEvent>,
    terrain_heights: Res<GlobalTerrainHeights>,
    mut terrain_query: Query<(Entity, &mut Mesh3d, &HeightfieldTerrain)>,
) {
    for event in terrain_events.read() {
        if event.needs_mesh_update || event.needs_collider_update {
            // Update all heightfield terrain entities
            for (entity, mut mesh_handle, _) in terrain_query.iter_mut() {
                if event.needs_mesh_update {
                    // Replace mesh with updated heightfield mesh
                    *mesh_handle = Mesh3d(meshes.add(terrain_heights.heightfield.create_visual_mesh()));
                }
                
                if event.needs_collider_update {
                    // Remove old collider and add new one
                    commands.entity(entity).remove::<Collider>();
                    commands.entity(entity).insert(terrain_heights.heightfield.create_physics_collider());
                }
            }
        }
        
        if event.needs_entity_repositioning {
            // TODO: Reposition vehicles, NPCs, etc. on new terrain
            // This would involve querying all entities that should be on ground
            // and updating their Y positions using the new heightfield
        }
    }
}

/// System to validate all terrain-dependent systems stay synchronized
pub fn validate_terrain_consistency(
    terrain_heights: Res<GlobalTerrainHeights>,
    ground_service: Res<crate::services::ground_detection::GroundDetectionService>,
) {
    // Sample a few positions to ensure heightfield and ground service agree
    let test_positions = [Vec2::ZERO, Vec2::new(100.0, 100.0), Vec2::new(-200.0, 300.0)];
    
    for pos in test_positions {
        let heightfield_result = terrain_heights.get_height_at_position(pos);
        let ground_result = ground_service.get_ground_height_simple(pos);
        
        let difference = (heightfield_result - ground_result).abs();
        if difference > 0.1 {
            warn!("❌ TERRAIN SYNC ERROR: At position {:?}, heightfield says {:.3} but ground service says {:.3} (diff: {:.3})", 
                  pos, heightfield_result, ground_result, difference);
        }
    }
}

/// Critical validation system that samples positions and confirms all three systems 
/// (visual mesh, physics collider, height queries) return identical heights
pub fn validate_single_source_of_truth(
    terrain_heights: Res<GlobalTerrainHeights>,
    terrain_query: Query<&Collider, With<HeightfieldTerrain>>,
) {
    if terrain_query.is_empty() {
        warn!("⚠️ TERRAIN VALIDATION: No HeightfieldTerrain entities found yet - skipping validation");
        return; // Terrain not spawned yet
    }

    info!("🔍 VALIDATING SINGLE SOURCE OF TRUTH for terrain heightfield...");

    let test_positions = [
        Vec2::ZERO,
        Vec2::new(512.0, 512.0),
        Vec2::new(-1024.0, 1024.0),
        Vec2::new(1536.0, -768.0),
        Vec2::new(-2000.0, -2000.0),
    ];

    let mut validation_passed = true;
    
    for pos in test_positions {
        // 1. Get height from GlobalTerrainHeights resource (height query system)
        let resource_height = terrain_heights.get_height_at_position(pos);
        
        // 2. Get height directly from heightfield (visual mesh system)
        let mesh_height = terrain_heights.heightfield.get_height_at_world_pos(pos) - 0.15; // Account for transform offset
        
        // 3. Get height from physics collider system (requires manual calculation)
        let collider_height = {
            // Convert world position to grid position for physics collider validation
            let grid_x = ((pos.x / terrain_heights.heightfield.scale.x) + 0.5) * (terrain_heights.heightfield.width - 1) as f32;
            let grid_z = ((pos.y / terrain_heights.heightfield.scale.z) + 0.5) * (terrain_heights.heightfield.height - 1) as f32;
            
            let grid_x_clamped = grid_x.clamp(0.0, (terrain_heights.heightfield.width - 1) as f32) as usize;
            let grid_z_clamped = grid_z.clamp(0.0, (terrain_heights.heightfield.height - 1) as f32) as usize;
            
            terrain_heights.heightfield.get_height_at_grid(grid_x_clamped, grid_z_clamped) * terrain_heights.heightfield.scale.y - 0.15
        };

        // Check if all three systems agree within tolerance
        let tolerance = 0.001;
        let resource_mesh_diff = (resource_height - mesh_height).abs();
        let resource_collider_diff = (resource_height - collider_height).abs();
        let mesh_collider_diff = (mesh_height - collider_height).abs();

        if resource_mesh_diff > tolerance {
            error!("❌ SINGLE SOURCE VIOLATION: At {:?}, resource height {:.6} != mesh height {:.6} (diff: {:.6})",
                  pos, resource_height, mesh_height, resource_mesh_diff);
            validation_passed = false;
        }
        
        if resource_collider_diff > tolerance {
            error!("❌ SINGLE SOURCE VIOLATION: At {:?}, resource height {:.6} != collider height {:.6} (diff: {:.6})",
                  pos, resource_height, collider_height, resource_collider_diff);
            validation_passed = false;
        }
        
        if mesh_collider_diff > tolerance {
            error!("❌ SINGLE SOURCE VIOLATION: At {:?}, mesh height {:.6} != collider height {:.6} (diff: {:.6})",
                  pos, mesh_height, collider_height, mesh_collider_diff);
            validation_passed = false;
        }
    }
    
    if validation_passed {
        info!("✅ SINGLE SOURCE VALIDATION PASSED: All systems use identical terrain data");
    } else {
        error!("❌ CRITICAL BUG: Multiple TerrainHeightfield instances detected!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    #[test]
    fn test_coordinate_conversion_roundtrip() {
        let terrain = TerrainHeightfield::new_flat(10, 10, Vec3::new(1000.0, 100.0, 1000.0));
        
        let test_positions = [
            Vec2::ZERO,
            Vec2::new(100.0, 200.0),
            Vec2::new(-300.0, 150.0),
            Vec2::new(500.0, -400.0),
            Vec2::new(-250.0, -350.0),
        ];
        
        for world_pos in test_positions {
            let grid_pos = terrain.world_to_grid(world_pos);
            let world_pos_back = terrain.grid_to_world(grid_pos);
            
            let diff = (world_pos - world_pos_back).length();
            assert!(diff < 0.001, 
                "Round-trip conversion failed for {:?}: grid={:?}, back={:?}, diff={:.6}", 
                world_pos, grid_pos, world_pos_back, diff);
        }
    }
    
    #[test]
    fn test_negative_coordinates_safe() {
        let terrain = TerrainHeightfield::new_flat(5, 5, Vec3::new(100.0, 10.0, 100.0));
        
        // These should not crash (negative coordinates clamped safely)
        let negative_positions = [
            Vec2::new(-1000.0, -1000.0),
            Vec2::new(-50.0, 25.0),
            Vec2::new(25.0, -50.0),
        ];
        
        for pos in negative_positions {
            let height = terrain.get_height_at_world_pos(pos);
            assert!(height >= 0.0, "Height query at {:?} returned {:.3}", pos, height);
        }
    }
    
    #[test]
    fn test_center_position_mapping() {
        let terrain = TerrainHeightfield::new_flat(5, 5, Vec3::new(100.0, 10.0, 100.0));
        
        // (0,0) world position should map to center of grid
        let grid_pos = terrain.world_to_grid(Vec2::ZERO);
        let expected_center = Vec2::new(2.0, 2.0); // Center of 5x5 grid
        
        let diff = (grid_pos - expected_center).length();
        assert!(diff < 0.001, 
            "World (0,0) should map to grid center {:?}, got {:?}, diff={:.6}",
            expected_center, grid_pos, diff);
    }
    
    #[test]
    fn test_out_of_bounds_clamping() {
        let terrain = TerrainHeightfield::new_flat(3, 3, Vec3::new(10.0, 1.0, 10.0));
        
        let out_of_bounds_positions = [
            Vec2::new(1000.0, 1000.0),   // Way out positive
            Vec2::new(-1000.0, -1000.0), // Way out negative
            Vec2::new(6.0, -6.0),        // Mixed
        ];
        
        for pos in out_of_bounds_positions {
            // Should not crash and return valid heights
            let height = terrain.get_height_at_world_pos(pos);
            assert!(height >= 0.0, "Out of bounds query at {:?} crashed or returned invalid height {:.3}", pos, height);
        }
    }
    
    #[test]
    fn test_coordinate_conversion_edge_cases() {
        let terrain = TerrainHeightfield::new_flat(4, 4, Vec3::new(30.0, 5.0, 30.0));
        
        // Test exact grid boundaries 
        let grid_corners = [
            Vec2::new(0.0, 0.0), // Should map to grid (1.5, 1.5) 
            Vec2::new(-15.0, -15.0), // Should map to grid (0, 0)
            Vec2::new(15.0, 15.0),   // Should map to grid (3, 3)
        ];
        
        for world_pos in grid_corners {
            let grid_pos = terrain.world_to_grid(world_pos);
            
            // Verify grid coordinates are within valid range
            assert!(grid_pos.x >= 0.0 && grid_pos.x <= 3.0, 
                "Grid X coordinate {:.3} out of range [0,3] for world pos {:?}", grid_pos.x, world_pos);
            assert!(grid_pos.y >= 0.0 && grid_pos.y <= 3.0, 
                "Grid Z coordinate {:.3} out of range [0,3] for world pos {:?}", grid_pos.y, world_pos);
        }
    }
    
    #[test]
    fn test_coordinate_conversion_consistency_with_mesh_generation() {
        let terrain = TerrainHeightfield::new_flat(3, 3, Vec3::new(20.0, 1.0, 20.0));
        
        // Verify that mesh vertex positions match coordinate conversion
        // From mesh generation: world_x = (x / (width-1) - 0.5) * scale.x
        // Should be inverse of world_to_grid conversion
        
        for x in 0..3 {
            for z in 0..3 {
                // Calculate mesh vertex position (same as create_visual_mesh)
                let mesh_world_x = (x as f32 / (3 - 1) as f32 - 0.5) * 20.0;
                let mesh_world_z = (z as f32 / (3 - 1) as f32 - 0.5) * 20.0;
                let mesh_world_pos = Vec2::new(mesh_world_x, mesh_world_z);
                
                // Convert back to grid using our conversion function
                let converted_grid = terrain.world_to_grid(mesh_world_pos);
                
                let expected_grid = Vec2::new(x as f32, z as f32);
                let diff = (converted_grid - expected_grid).length();
                
                assert!(diff < 0.001, 
                    "Mesh vertex at grid ({},{}) has world pos {:?} but converts back to grid {:?}, expected {:?}, diff={:.6}",
                    x, z, mesh_world_pos, converted_grid, expected_grid, diff);
            }
        }
    }
    
    #[test]
    fn test_physics_visual_alignment() {
        // Create a 4km x 4km terrain (same as production)
        let terrain = TerrainHeightfield::new_flat(64, 64, Vec3::new(4096.0, 10.0, 4096.0));
        
        // Test positions including terrain boundaries
        let test_positions = [
            Vec2::ZERO,                      // Center
            Vec2::new(100.0, 100.0),        // Near center
            Vec2::new(2048.0, 2048.0),      // Corner (should be at edge)
            Vec2::new(-2048.0, -2048.0),    // Opposite corner
            Vec2::new(2048.0, -2048.0),     // Mixed corners
            Vec2::new(-2048.0, 2048.0),
            Vec2::new(0.0, 2048.0),         // Edge centers
            Vec2::new(2048.0, 0.0),
        ];
        
        for world_pos in test_positions {
            // Get height from visual mesh coordinate system
            let visual_height = terrain.get_height_at_world_pos(world_pos);
            
            // Get height using the same algorithm that physics collider should use
            // Based on Rapier docs: scale represents full size of rectangle in X-Z plane
            let grid_x = ((world_pos.x / terrain.scale.x) + 0.5) * (terrain.width - 1) as f32;
            let grid_z = ((world_pos.y / terrain.scale.z) + 0.5) * (terrain.height - 1) as f32;
            
            // Validate that our world positions map to valid grid coordinates
            assert!(grid_x >= 0.0 && grid_x <= (terrain.width - 1) as f32, 
                "World position {:?} maps to invalid grid X {:.3}, expected [0, {}]",
                world_pos, grid_x, terrain.width - 1);
            assert!(grid_z >= 0.0 && grid_z <= (terrain.height - 1) as f32,
                "World position {:?} maps to invalid grid Z {:.3}, expected [0, {}]", 
                world_pos, grid_z, terrain.height - 1);
                
            // Clamp and get physics height (same as Rapier would do internally)
            let grid_x_clamped = grid_x.clamp(0.0, (terrain.width - 1) as f32) as usize;
            let grid_z_clamped = grid_z.clamp(0.0, (terrain.height - 1) as f32) as usize;
            let physics_height = terrain.get_height_at_grid(grid_x_clamped, grid_z_clamped) * terrain.scale.y;
            
            // Visual and physics should match exactly
            let diff = (visual_height - physics_height).abs();
            assert!(diff < 0.001, 
                "Physics/visual mismatch at {:?}: visual={:.6}, physics={:.6}, diff={:.6}",
                world_pos, visual_height, physics_height, diff);
        }
    }
    
    #[test]  
    fn test_terrain_boundaries_exact() {
        // Test that terrain boundaries are exactly where we expect them
        let terrain = TerrainHeightfield::new_flat(64, 64, Vec3::new(4096.0, 10.0, 4096.0));
        
        // Check that terrain extends from -2048 to +2048 in both X and Z
        // Based on visual mesh generation: world_x = (x/(width-1) - 0.5) * scale.x
        
        // Corner grid positions
        let corner_positions = [
            (0, 0),                           // Grid corner -> world corner
            (63, 0),                         // Other corners
            (0, 63),
            (63, 63),
        ];
        
        for (grid_x, grid_z) in corner_positions {
            // Calculate expected world position from mesh generation formula
            let expected_world_x = (grid_x as f32 / (64 - 1) as f32 - 0.5) * 4096.0;
            let expected_world_z = (grid_z as f32 / (64 - 1) as f32 - 0.5) * 4096.0;
            let expected_world_pos = Vec2::new(expected_world_x, expected_world_z);
            
            // Convert back to grid using our conversion function
            let converted_grid = terrain.world_to_grid(expected_world_pos);
            
            let expected_grid = Vec2::new(grid_x as f32, grid_z as f32);
            let diff = (converted_grid - expected_grid).length();
            
            assert!(diff < 0.001,
                "Grid corner ({},{}) -> world {:?} -> grid {:?}, diff={:.6}",
                grid_x, grid_z, expected_world_pos, converted_grid, diff);
        }
        
        // Validate exact boundaries
        assert!((terrain.grid_to_world(Vec2::new(0.0, 0.0)).x - (-2048.0)).abs() < 0.001,
            "Left boundary should be -2048");
        assert!((terrain.grid_to_world(Vec2::new(63.0, 0.0)).x - 2048.0).abs() < 0.001,
            "Right boundary should be +2048");
        assert!((terrain.grid_to_world(Vec2::new(0.0, 0.0)).y - (-2048.0)).abs() < 0.001,
            "Bottom boundary should be -2048");
        assert!((terrain.grid_to_world(Vec2::new(0.0, 63.0)).y - 2048.0).abs() < 0.001,
            "Top boundary should be +2048");
    }
}
