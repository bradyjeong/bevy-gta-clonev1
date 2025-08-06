# Vegetation Optimization Guide for Bevy Games

## Overview
This guide provides comprehensive strategies for optimizing detailed vegetation in Bevy games while maintaining 60+ FPS performance. Focuses on practical techniques for grass, trees, and foliage that work with Bevy 0.16.1.

## 1. Instancing Techniques for Repeated Geometry

### GPU Instancing for Vegetation
```rust
// Example: Grass blade instancing
#[derive(Component)]
pub struct GrassInstanceData {
    pub transforms: Vec<Mat4>,
    pub variations: Vec<u32>, // texture/color variations
}

impl GrassInstanceData {
    pub fn new_batch(positions: &[Vec3], variations: usize) -> Self {
        let transforms = positions.iter().map(|pos| {
            let rotation = Quat::from_rotation_y(fastrand::f32() * 2.0 * PI);
            let scale = Vec3::splat(0.8 + fastrand::f32() * 0.4);
            Mat4::from_scale_rotation_translation(scale, rotation, *pos)
        }).collect();
        
        let variations = (0..positions.len())
            .map(|_| fastrand::u32(0..variations as u32))
            .collect();
            
        Self { transforms, variations }
    }
}
```

### Mesh Instance Optimization
- **Target**: 1000-5000 instances per draw call
- **Memory**: Pre-allocate instance buffers, reuse when possible
- **Culling**: Frustum cull entire instance batches, not individual instances

## 2. Level of Detail (LOD) Systems

### Distance-Based LOD Strategy
```rust
#[derive(Component)]
pub struct VegetationLOD {
    pub meshes: Vec<Handle<Mesh>>, // LOD0 (high) to LOD3 (low)
    pub distances: [f32; 4],       // Switch distances: [50, 100, 200, 300]
    pub current_lod: usize,
}

impl VegetationLOD {
    pub fn update_lod(&mut self, distance_to_camera: f32) {
        self.current_lod = self.distances.iter()
            .position(|&d| distance_to_camera < d)
            .unwrap_or(self.meshes.len() - 1);
    }
}
```

### LOD Transition Distances
| Vegetation Type | LOD0 | LOD1 | LOD2 | LOD3 | Cull Distance |
|----------------|------|------|------|------|---------------|
| Grass Clumps   | 0-25m| 25-50m| 50-100m| 100-150m| 150m |
| Bushes/Shrubs  | 0-40m| 40-80m| 80-150m| 150-250m| 250m |
| Trees (Small)  | 0-50m| 50-100m| 100-200m| 200-300m| 300m |
| Trees (Large)  | 0-75m| 75-150m| 150-300m| 300-500m| 500m |

## 3. Billboarding for Distant Vegetation

### Impostor Billboards
```rust
#[derive(Component)]
pub struct VegetationBillboard {
    pub atlas_texture: Handle<Image>,
    pub frame_count: u32,
    pub current_frame: u32,
    pub facing_mode: BillboardMode,
}

pub enum BillboardMode {
    Screen,           // Always face camera
    Cylindrical,      // Rotate around Y-axis only
    Spherical,        // Full 3D rotation
}

// Billboard update system
pub fn update_vegetation_billboards(
    mut billboards: Query<(&mut Transform, &VegetationBillboard)>,
    camera_query: Query<&Transform, (With<Camera>, Without<VegetationBillboard>)>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        for (mut transform, billboard) in billboards.iter_mut() {
            let direction = camera_transform.translation - transform.translation;
            match billboard.facing_mode {
                BillboardMode::Screen => {
                    transform.look_at(camera_transform.translation, Vec3::Y);
                },
                BillboardMode::Cylindrical => {
                    let dir = Vec3::new(direction.x, 0.0, direction.z).normalize();
                    transform.look_at(transform.translation + dir, Vec3::Y);
                },
                BillboardMode::Spherical => {
                    transform.look_at(camera_transform.translation, Vec3::Y);
                }
            }
        }
    }
}
```

### Billboard Transition Strategy
- **Near to Mid**: Fade from 3D mesh to billboard at LOD2 distance
- **Alpha Blending**: Use dithering for smooth transitions
- **Atlas Optimization**: Pack multiple views in texture atlas (8x8 grid common)

## 4. Mesh Merging vs Instancing Trade-offs

### When to Use Mesh Merging
- **Static vegetation** that never moves
- **Dense clusters** with < 100 unique positions
- **Uniform materials** across vegetation
- **Memory-constrained** scenarios

### When to Use Instancing
- **Dynamic vegetation** (wind animation, growth)
- **Large quantities** (1000+ instances)
- **Varied materials** per instance
- **GPU-heavy** scenarios

### Hybrid Approach
```rust
pub struct VegetationBatchManager {
    // Small static clusters use merged meshes
    pub merged_batches: Vec<Handle<Mesh>>,
    // Large dynamic areas use instancing
    pub instance_batches: Vec<GrassInstanceData>,
    // Threshold for switching strategies
    pub merge_threshold: usize, // Typically 50-100
}
```

## 5. Bevy-Specific Optimization Patterns

### Efficient Component Queries
```rust
// Batch process vegetation updates
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct VegetationSystems;

pub fn vegetation_update_system(
    // Use With<> filters for efficient queries
    mut vegetation: Query<(&mut Transform, &VegetationLOD), 
                         (With<Vegetation>, Without<Camera>)>,
    camera: Query<&Transform, With<Camera>>,
    time: Res<Time>,
) {
    // Process in chunks to avoid frame drops
    const CHUNK_SIZE: usize = 100;
    
    if let Ok(camera_transform) = camera.get_single() {
        for chunk in vegetation.iter_mut().chunks(CHUNK_SIZE) {
            for (mut transform, lod) in chunk {
                // Process vegetation update
            }
        }
    }
}
```

### Memory Pool Management
```rust
#[derive(Resource)]
pub struct VegetationMemoryPool {
    pub instance_buffers: Vec<Vec<Mat4>>,
    pub mesh_cache: HashMap<String, Handle<Mesh>>,
    pub material_cache: HashMap<String, Handle<StandardMaterial>>,
}

impl VegetationMemoryPool {
    pub fn get_instance_buffer(&mut self, capacity: usize) -> Vec<Mat4> {
        self.instance_buffers.pop()
            .unwrap_or_else(|| Vec::with_capacity(capacity))
    }
    
    pub fn return_instance_buffer(&mut self, mut buffer: Vec<Mat4>) {
        buffer.clear();
        if buffer.capacity() > 0 {
            self.instance_buffers.push(buffer);
        }
    }
}
```

### Spatial Partitioning
```rust
#[derive(Resource)]
pub struct VegetationSpatialGrid {
    pub cell_size: f32,
    pub cells: HashMap<IVec2, Vec<Entity>>,
}

impl VegetationSpatialGrid {
    pub fn get_nearby_vegetation(&self, position: Vec3, radius: f32) -> Vec<Entity> {
        let cell_min = self.world_to_cell(position - Vec3::splat(radius));
        let cell_max = self.world_to_cell(position + Vec3::splat(radius));
        
        let mut entities = Vec::new();
        for x in cell_min.x..=cell_max.x {
            for y in cell_min.y..=cell_max.y {
                if let Some(cell_entities) = self.cells.get(&IVec2::new(x, y)) {
                    entities.extend(cell_entities);
                }
            }
        }
        entities
    }
}
```

## 6. Performance Targets for 60+ FPS

### Recommended Limits
| Category | Target Count | Max Draw Calls | Memory Budget |
|----------|-------------|----------------|---------------|
| Grass Instances | 10,000-50,000 | 5-10 | 50-100MB |
| Tree Instances | 1,000-5,000 | 10-20 | 100-200MB |
| Bushes/Shrubs | 2,000-8,000 | 5-15 | 50-150MB |
| Billboards | 20,000-100,000 | 5-10 | 20-50MB |

### System Performance Intervals
```rust
// Vegetation update frequencies
pub const VEGETATION_LOD_UPDATE_INTERVAL: f32 = 0.2;  // 5 FPS
pub const VEGETATION_CULLING_INTERVAL: f32 = 0.1;     // 10 FPS  
pub const VEGETATION_BILLBOARD_INTERVAL: f32 = 0.05;  // 20 FPS
pub const VEGETATION_WIND_INTERVAL: f32 = 0.016;      // 60 FPS
```

### GPU Budget Allocation
- **Vegetation Rendering**: 3-5ms per frame
- **Culling Operations**: 0.5-1ms per frame
- **LOD Updates**: 0.1-0.3ms per frame
- **Total Vegetation**: < 6ms per frame (10% of 16.67ms budget)

## 7. Implementation Checklist

### Phase 1: Basic Optimization
- [ ] Implement distance-based culling (150-300m)
- [ ] Add basic LOD system (2-3 levels)
- [ ] Use GPU instancing for grass/small vegetation
- [ ] Implement spatial partitioning grid

### Phase 2: Advanced Techniques
- [ ] Add billboard imposters for distant vegetation
- [ ] Implement mesh merging for static clusters
- [ ] Add wind animation using vertex shaders
- [ ] Optimize material usage (shared materials)

### Phase 3: Performance Tuning
- [ ] Profile and optimize critical paths
- [ ] Implement temporal LOD updates
- [ ] Add memory pooling for dynamic buffers
- [ ] Fine-tune distance thresholds per scene

## 8. Bevy-Specific Considerations

### Asset Pipeline
- Use Bevy's asset preprocessing for LOD generation
- Compress vegetation textures using DXT/BC formats
- Consider using Bevy's virtual geometry (meshlets) for complex vegetation

### Rendering Pipeline
- Leverage Bevy's batching system for vegetation
- Use custom materials for vegetation-specific shaders
- Consider deferred rendering for dense vegetation scenes

### Memory Management
- Use Bevy's asset loading for streaming vegetation
- Implement proper cleanup for despawned vegetation
- Monitor memory usage with Bevy's diagnostics

## Conclusion

Achieving 60+ FPS with detailed vegetation requires a combination of these techniques:

1. **Aggressive culling** at multiple levels (frustum, distance, occlusion)
2. **Smart LOD systems** with appropriate transition distances
3. **Efficient instancing** for repeated geometry
4. **Billboard impostors** for distant vegetation
5. **Spatial optimization** using grids or octrees
6. **Memory management** with pooling and streaming

The key is to implement these systems incrementally, profile performance at each step, and tune parameters based on your specific use case and target hardware.
