use bevy::prelude::*;
use crate::config::{GameConfig, WorldConfig};

pub struct WorldParams<'a> {
    pub cfg: &'a WorldConfig,
}

impl<'a> WorldParams<'a> {
    pub fn new(config: &'a GameConfig) -> Self {
        Self { cfg: &config.world }
    }

    pub fn chunk_coord_from_world(&self, pos: Vec3) -> IVec2 {
        IVec2::new(
            (pos.x / self.cfg.chunk_size).floor() as i32,
            (pos.z / self.cfg.chunk_size).floor() as i32,
        )
    }

    pub fn world_pos_from_chunk(&self, coord: IVec2) -> Vec3 {
        Vec3::new(
            coord.x as f32 * self.cfg.chunk_size + self.cfg.chunk_size * 0.5,
            0.0,
            coord.y as f32 * self.cfg.chunk_size + self.cfg.chunk_size * 0.5,
        )
    }

    pub fn lod_level(&self, distance: f32) -> usize {
        for (i, &max_distance) in self.cfg.lod_distances.iter().enumerate() {
            if distance <= max_distance {
                return i;
            }
        }
        self.cfg.lod_distances.len() - 1
    }

    pub fn streaming_radius_chunks(&self) -> i32 {
        (self.cfg.streaming_radius / self.cfg.chunk_size).ceil() as i32
    }

    pub fn is_chunk_in_range(&self, chunk_coord: IVec2, center: IVec2) -> bool {
        let dx = (chunk_coord.x - center.x).abs();
        let dz = (chunk_coord.y - center.y).abs();
        let radius_chunks = self.streaming_radius_chunks();
        dx <= radius_chunks && dz <= radius_chunks
    }
}
