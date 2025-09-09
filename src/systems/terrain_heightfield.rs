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

    /// Get interpolated height at world position
    pub fn get_height_at_world_pos(&self, world_pos: Vec2) -> f32 {
        // Convert world position to grid coordinates
        let grid_x = (world_pos.x / self.scale.x) * (self.width - 1) as f32;
        let grid_z = (world_pos.y / self.scale.z) * (self.height - 1) as f32;

        // Simple nearest-neighbor sampling for now (can add bilinear later)
        let grid_x_int = grid_x.round() as usize;
        let grid_z_int = grid_z.round() as usize;

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
        Collider::heightfield(self.heights.clone(), self.width, self.height, self.scale)
    }
}

/// Marker component for heightfield terrain
#[derive(Component)]
pub struct HeightfieldTerrain;

/// System to spawn heightfield terrain
pub fn spawn_heightfield_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create 4km x 4km heightfield terrain (matching current system)
    let terrain = TerrainHeightfield::new_flat(
        64,  // Grid resolution 64x64
        64,
        Vec3::new(4096.0, 10.0, 4096.0), // 4km x 4km, 10m max height
    );

    // Spawn unified heightfield terrain entity
    commands.spawn((
        HeightfieldTerrain,
        Mesh3d(meshes.add(terrain.create_visual_mesh())),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))), // Same color as original
        Transform::from_xyz(0.0, -0.15, 0.0), // Same position as original
        RigidBody::Fixed,
        terrain.create_physics_collider(), // Heightfield collider
        CollisionGroups::new(
            crate::constants::STATIC_GROUP,
            crate::constants::VEHICLE_GROUP | crate::constants::CHARACTER_GROUP,
        ),
    ));
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
