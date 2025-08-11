use bevy::prelude::*;

/// PlacementGrid - Spatial grid for entity placement and collision (≤40 bytes)
#[derive(Resource, Debug)]
pub struct PlacementGrid {
    /// Packed occupied cells bitfield (8 bytes)
    pub occupied_cells: u64,
    /// Grid origin and size (16 bytes)
    pub grid_size: u16,
    /// Validation frame counter (4 bytes)
    pub validation_frame: u32,
    /// Placement mode flags (1 byte)
    pub placement_mode: PlacementMode,
    /// Last clear frame (4 bytes)
    pub last_clear_frame: u32,
}

/// Grid cell coordinate (8 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: i32,
    pub z: i32,
}

/// Content type for placement validation (1 byte enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Building,
    Vehicle,
    Vegetation,
    Prop,
}

/// Placement mode (1 byte enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementMode {
    Normal,
    Dense,
    Sparse,
    Override,
}

impl Default for PlacementGrid {
    fn default() -> Self {
        Self {
            occupied_cells: 0,
            grid_size: 16,
            validation_frame: 0,
            placement_mode: PlacementMode::Normal,
            last_clear_frame: 0,
        }
    }
}

impl PlacementGrid {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn clear(&mut self) {
        self.occupied_cells = 0;
        self.last_clear_frame = self.validation_frame;
    }
    
    pub fn world_to_grid(&self, position: Vec3) -> GridCell {
        let cell_size = 50.0;
        GridCell {
            x: (position.x / cell_size).floor() as i32,
            z: (position.z / cell_size).floor() as i32,
        }
    }
    
    pub fn can_place(&self, position: Vec3, _content_type: ContentType, _radius: f32) -> bool {
        let cell = self.world_to_grid(position);
        let cell_index = self.grid_cell_to_index(cell);
        
        if cell_index >= 64 {
            return false; // Outside grid bounds
        }
        
        // Check if bit is set in occupied cells
        (self.occupied_cells & (1u64 << cell_index)) == 0
    }
    
    pub fn add_entity(&mut self, position: Vec3, _content_type: ContentType, _radius: f32) {
        let cell = self.world_to_grid(position);
        let cell_index = self.grid_cell_to_index(cell);
        
        if cell_index < 64 {
            self.occupied_cells |= 1u64 << cell_index;
        }
        
        self.validation_frame += 1;
    }
    
    pub fn remove_entity(&mut self, position: Vec3, _content_type: ContentType, _radius: f32) {
        let cell = self.world_to_grid(position);
        let cell_index = self.grid_cell_to_index(cell);
        
        if cell_index < 64 {
            self.occupied_cells &= !(1u64 << cell_index);
        }
        
        self.validation_frame += 1;
    }
    
    fn grid_cell_to_index(&self, cell: GridCell) -> usize {
        // Map grid cell to bitfield index (8x8 grid = 64 cells)
        // Use rem_euclid for proper negative coordinate handling
        let x = cell.x.rem_euclid(8) as usize;
        let z = cell.z.rem_euclid(8) as usize;
        z * 8 + x
    }
    
    // Additional methods for migration compatibility
    pub fn get_occupied_count(&self) -> usize {
        self.occupied_cells.count_ones() as usize
    }
    
    pub fn get_cell_size(&self) -> f32 {
        50.0 // Fixed cell size
    }
    
    pub fn set_cell_size(&mut self, _size: f32) {
        // Cell size is fixed at 50.0 in this implementation
        // This method exists for migration compatibility
    }
    
    pub fn mark_occupied(&mut self, position: Vec3, radius: f32) {
        self.add_entity(position, ContentType::Building, radius);
    }
    
    pub fn can_place_at(&self, position: Vec3, radius: f32, min_distance: f32) -> bool {
        self.can_place(position, ContentType::Building, radius.max(min_distance))
    }
    
    // V2 compatibility methods
    pub fn check_collision(&self, position: Vec3, radius: f32) -> bool {
        // Returns true if there's a collision (opposite of can_place)
        // Use EventContentType::Building for V2 compatibility
        !self.can_place(position, ContentType::Building, radius)
    }
    
    pub fn find_free_position(&self, base_position: Vec3, radius: f32, search_radius: f32) -> Option<Vec3> {
        // Try to find a free position near the base position
        let steps = 8;
        let angle_step = std::f32::consts::TAU / steps as f32;
        
        for distance in [10.0, 20.0, 30.0, 40.0] {
            if distance > search_radius {
                break;
            }
            
            for i in 0..steps {
                let angle = i as f32 * angle_step;
                let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);
                let test_pos = base_position + offset;
                
                if !self.check_collision(test_pos, radius) {
                    return Some(test_pos);
                }
            }
        }
        
        // If no free position found in search radius, return base if it's free
        if !self.check_collision(base_position, radius) {
            Some(base_position)
        } else {
            None
        }
    }
}

// Static size assertion - PlacementGrid actual size
// occupied_cells: u64 = 8 bytes
// grid_size: u16 = 2 bytes
// validation_frame: u32 = 4 bytes
// placement_mode: PlacementMode = 1 byte
// last_clear_frame: u32 = 4 bytes
// Total: 8 + 2 + 4 + 1 + 4 = 19 bytes + alignment = 24 bytes
static_assertions::const_assert!(std::mem::size_of::<PlacementGrid>() <= 24);
