use bevy::prelude::*;

/// WorldCoordinator - Lightweight coordinator for world resources (≤32 bytes)
#[derive(Resource, Debug)]
pub struct WorldCoordinator {
    /// Current focus position for streaming (12 bytes)
    pub focus_position: IVec3,
    /// Streaming radius for all systems (4 bytes)
    pub streaming_radius: f32,
    /// Generation frame counter (4 bytes)
    pub generation_frame: u32,
    /// Coordination flags (4 bytes)
    pub flags: CoordinationFlags,
    /// Reserved for future expansion (8 bytes)
    pub _reserved: [u32; 2],
}

/// Coordination flags for world management (4 bytes)
#[derive(Debug, Clone, Copy)]
pub struct CoordinationFlags {
    /// Packed bitflags for various states
    pub bits: u32,
}

impl CoordinationFlags {
    pub const STREAMING_ACTIVE: u32 = 1 << 0;
    pub const GENERATION_PAUSED: u32 = 1 << 1;
    pub const REBUILD_REQUESTED: u32 = 1 << 2;
    pub const PERFORMANCE_MODE: u32 = 1 << 3;
    
    pub fn new() -> Self {
        Self { bits: Self::STREAMING_ACTIVE }
    }
    
    pub fn has(&self, flag: u32) -> bool {
        self.bits & flag != 0
    }
    
    pub fn set(&mut self, flag: u32) {
        self.bits |= flag;
    }
    
    pub fn clear(&mut self, flag: u32) {
        self.bits &= !flag;
    }
}

impl Default for WorldCoordinator {
    fn default() -> Self {
        Self {
            focus_position: IVec3::ZERO,
            streaming_radius: 500.0,
            generation_frame: 0,
            flags: CoordinationFlags::new(),
            _reserved: [0; 2],
        }
    }
}

impl WorldCoordinator {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update_focus(&mut self, position: Vec3) {
        self.focus_position = IVec3::new(position.x as i32, position.y as i32, position.z as i32);
        self.generation_frame += 1;
    }
    
    pub fn update_focus_ivec(&mut self, position: IVec3) {
        self.focus_position = position;
        self.generation_frame += 1;
    }
    
    pub fn get_focus_vec3(&self) -> Vec3 {
        Vec3::new(self.focus_position.x as f32, self.focus_position.y as f32, self.focus_position.z as f32)
    }
    
    pub fn is_streaming_active(&self) -> bool {
        self.flags.has(CoordinationFlags::STREAMING_ACTIVE)
    }
    
    pub fn set_streaming_active(&mut self, active: bool) {
        if active {
            self.flags.set(CoordinationFlags::STREAMING_ACTIVE);
        } else {
            self.flags.clear(CoordinationFlags::STREAMING_ACTIVE);
        }
    }
    
    pub fn is_generation_paused(&self) -> bool {
        self.flags.has(CoordinationFlags::GENERATION_PAUSED)
    }
    
    pub fn pause_generation(&mut self) {
        self.flags.set(CoordinationFlags::GENERATION_PAUSED);
    }
    
    pub fn resume_generation(&mut self) {
        self.flags.clear(CoordinationFlags::GENERATION_PAUSED);
    }
    
    pub fn request_rebuild(&mut self) {
        self.flags.set(CoordinationFlags::REBUILD_REQUESTED);
    }
    
    pub fn consume_rebuild_request(&mut self) -> bool {
        let requested = self.flags.has(CoordinationFlags::REBUILD_REQUESTED);
        if requested {
            self.flags.clear(CoordinationFlags::REBUILD_REQUESTED);
        }
        requested
    }
    
    pub fn set_max_chunks_per_frame(&mut self, max_chunks: usize) {
        // Store in reserved field
        self._reserved[0] = max_chunks as u32;
    }
    
    pub fn get_max_chunks_per_frame(&self) -> usize {
        self._reserved[0] as usize
    }
}

// Static size assertion - ensure WorldCoordinator ≤32 bytes
static_assertions::const_assert!(std::mem::size_of::<WorldCoordinator>() <= 32);
