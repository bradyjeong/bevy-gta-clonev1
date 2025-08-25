use bevy::prelude::*;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Deterministic RNG resource for world generation
/// Ensures reproducible worlds given the same seed
#[derive(Resource)]
pub struct WorldRng {
    /// Global RNG for world-level decisions
    global_rng: StdRng,
}

impl WorldRng {
    /// Create new WorldRng with given seed
    pub fn new(seed: u64) -> Self {
        Self {
            global_rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Get global RNG for world-level decisions
    pub fn global(&mut self) -> &mut StdRng {
        &mut self.global_rng
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
