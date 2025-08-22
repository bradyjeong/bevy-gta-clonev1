use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::collections::HashMap;

/// Deterministic RNG resource for world generation
/// Ensures reproducible worlds given the same seed
#[derive(Resource)]
pub struct WorldRng {
    /// Main world seed
    pub seed: u64,
    /// Per-chunk RNG states for reproducible chunk generation
    chunk_rngs: HashMap<(i32, i32), Xoshiro256PlusPlus>,
    /// Global RNG for world-level decisions
    global_rng: Xoshiro256PlusPlus,
}

impl WorldRng {
    /// Create new WorldRng with given seed
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            chunk_rngs: HashMap::new(),
            global_rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        }
    }

    /// Get RNG for specific chunk coordinates
    /// Creates deterministic seed based on chunk position and world seed
    pub fn get_chunk_rng(&mut self, chunk_x: i32, chunk_z: i32) -> &mut Xoshiro256PlusPlus {
        self.chunk_rngs.entry((chunk_x, chunk_z)).or_insert_with(|| {
            // Create deterministic seed from chunk coords and world seed
            let chunk_seed = self.seed
                .wrapping_add(chunk_x as u64)
                .wrapping_mul(1000000007) // Large prime
                .wrapping_add(chunk_z as u64)
                .wrapping_mul(1000000009); // Another large prime
            Xoshiro256PlusPlus::seed_from_u64(chunk_seed)
        })
    }

    /// Get global RNG for world-level decisions
    pub fn global(&mut self) -> &mut Xoshiro256PlusPlus {
        &mut self.global_rng
    }

    /// Generate random float in range [0.0, 1.0) for chunk
    pub fn gen_f32_for_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> f32 {
        use rand::Rng;
        self.get_chunk_rng(chunk_x, chunk_z).gen()
    }

    /// Generate random value in range for chunk
    pub fn gen_range_for_chunk<T, R>(&mut self, chunk_x: i32, chunk_z: i32, range: R) -> T 
    where
        T: rand::distributions::uniform::SampleUniform,
        R: rand::distributions::uniform::SampleRange<T>,
    {
        self.get_chunk_rng(chunk_x, chunk_z).gen_range(range)
    }

    /// Clear chunk RNG cache to free memory
    /// Call periodically to prevent unbounded growth
    pub fn clear_chunk_cache(&mut self) {
        self.chunk_rngs.clear();
    }

    /// Get current cache size for monitoring
    pub fn cache_size(&self) -> usize {
        self.chunk_rngs.len()
    }
}

impl Default for WorldRng {
    fn default() -> Self {
        // Use current timestamp as default seed for variety
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(12345);
        Self::new(seed)
    }
}
