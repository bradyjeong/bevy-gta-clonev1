use bevy::prelude::*;

/// RoadNetwork - Manages road topology and pathfinding
/// Cache resource: accessed for pathfinding queries (<100 times/frame)
/// Size optimized for cache-friendliness despite infrequent access
#[derive(Resource, Debug)]
pub struct RoadNetwork {
    /// Road nodes array (16 bytes = 4 nodes * 4 bytes each)
    pub nodes: [(u16, u16); 4],
    /// Network connections bitfield (8 bytes)
    pub connections: u64,
    /// Active node count (1 byte)
    pub active_nodes: u8,
    /// Network flags (1 byte)
    pub network_flags: u8,
    /// Generation seed (4 bytes)
    pub generation_seed: u32,
}

/// Road node in the network (16 bytes)
#[derive(Debug, Clone, Copy)]
pub struct RoadNode {
    pub position: Vec3,  // 12 bytes
    pub road_type: RoadType,  // 1 byte
    pub _padding: [u8; 3],  // Padding for alignment
}

/// Road edge connecting two nodes (8 bytes)
#[derive(Debug, Clone, Copy)]
pub struct RoadEdge {
    pub from: u16,  // Node index
    pub to: u16,    // Node index
    pub weight: f32,  // Edge weight for pathfinding
}

/// Road type classification (1 byte)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoadType {
    Highway,
    MainStreet,
    SideStreet,
    Alley,
}

impl RoadType {
    pub fn width(&self) -> f32 {
        match self {
            RoadType::Highway => 16.0,
            RoadType::MainStreet => 12.0,
            RoadType::SideStreet => 8.0,
            RoadType::Alley => 4.0,
        }
    }
    
    pub fn priority(&self) -> i32 {
        match self {
            RoadType::Highway => 4,
            RoadType::MainStreet => 3,
            RoadType::SideStreet => 2,
            RoadType::Alley => 1,
        }
    }
}

/// Network generation settings (4 bytes)
#[derive(Debug, Clone, Copy)]
pub struct NetworkSettings {
    pub density: f32,  // Road density factor
}

impl Default for RoadNetwork {
    fn default() -> Self {
        Self {
            nodes: [(0, 0); 4],
            connections: 0,
            active_nodes: 0,
            network_flags: 0,
            generation_seed: 12345,
        }
    }
}

impl RoadNetwork {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_node(&mut self, x: u16, z: u16) -> u8 {
        if (self.active_nodes as usize) < self.nodes.len() {
            let index = self.active_nodes;
            self.nodes[index as usize] = (x, z);
            self.active_nodes += 1;
            index
        } else {
            self.active_nodes.saturating_sub(1)
        }
    }
    
    pub fn connect_nodes(&mut self, from: u8, to: u8) {
        if from < self.active_nodes && to < self.active_nodes && from != to {
            // Set bit for connection
            let bit_index = (from as usize) * 8 + (to as usize);
            if bit_index < 64 {
                self.connections |= 1u64 << bit_index;
            }
        }
    }
    
    pub fn clear(&mut self) {
        self.nodes = [(0, 0); 4];
        self.connections = 0;
        self.active_nodes = 0;
    }
    
    pub fn get_node(&self, index: u8) -> Option<(u16, u16)> {
        if index < self.active_nodes {
            Some(self.nodes[index as usize])
        } else {
            None
        }
    }
    
    pub fn node_count(&self) -> usize {
        self.active_nodes as usize
    }
    
    pub fn is_connected(&self, from: u8, to: u8) -> bool {
        if from < self.active_nodes && to < self.active_nodes {
            let bit_index = (from as usize) * 8 + (to as usize);
            if bit_index < 64 {
                return (self.connections & (1u64 << bit_index)) != 0;
            }
        }
        false
    }
    
    pub fn is_near_road(&self, position: Vec3, max_distance: f32) -> bool {
        // Check if position is near any road node
        for i in 0..self.active_nodes as usize {
            let (x, z) = self.nodes[i];
            let node_pos = Vec3::new(x as f32, position.y, z as f32);
            if position.distance(node_pos) <= max_distance {
                return true;
            }
        }
        false
    }
    
    pub fn get_nearest_road_point(&self, position: Vec3) -> Option<Vec3> {
        if self.active_nodes == 0 {
            return None;
        }
        
        let mut nearest_point = None;
        let mut min_distance = f32::MAX;
        
        for i in 0..self.active_nodes as usize {
            let (x, z) = self.nodes[i];
            let node_pos = Vec3::new(x as f32, position.y, z as f32);
            let distance = position.distance(node_pos);
            if distance < min_distance {
                min_distance = distance;
                nearest_point = Some(node_pos);
            }
        }
        
        nearest_point
    }
    
    pub fn find_path(&self, start: Vec3, end: Vec3) -> Option<Vec<Vec3>> {
        // Simplified pathfinding - return direct path for now
        Some(vec![start, end])
    }
}

// RoadNetwork is a cache resource (not hot-path)
// Even though it's small, it's accessed infrequently (<100 times/frame)
// So no strict size assertion needed, but we keep it small for efficiency
// nodes: [(u16, u16); 4] = 16 bytes
// connections: u64 = 8 bytes
// active_nodes: u8 = 1 byte
// network_flags: u8 = 1 byte
// generation_seed: u32 = 4 bytes
// Total: 16 + 8 + 1 + 1 + 4 = 30 bytes + alignment = 32 bytes
static_assertions::const_assert!(std::mem::size_of::<RoadNetwork>() <= 32);
