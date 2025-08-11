use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Resource)]
pub struct GlobalRng(pub StdRng);

impl Default for GlobalRng {
    fn default() -> Self {
        Self(StdRng::from_entropy())
    }
}

impl GlobalRng {
    pub fn gen_range<T: rand::distributions::uniform::SampleUniform, R: rand::distributions::uniform::SampleRange<T>>(&mut self, range: R) -> T {
        self.0.gen_range(range)
    }
    
    pub fn gen_f32(&mut self) -> f32 {
        self.gen_range(0.0..1.0)
    }
}
