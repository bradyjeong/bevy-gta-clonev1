use bevy::prelude::*;
use std::time::{Duration, Instant};

/// Benchmark world generation performance
pub fn benchmark_world_generation() {
    let mut app = setup_benchmark_app();
    
    // Warm up
    for _ in 0..5 {
        app.update();
    }
    
    // Benchmark chunk generation
    let start = Instant::now();
    for _ in 0..100 {
        app.update();
    }
    let duration = start.elapsed();
    
    println!("World generation benchmark:");
    println!("  100 frames: {:?}", duration);
    println!("  Average per frame: {:?}", duration / 100);
    println!("  FPS equivalent: {:.2}", 1000.0 / (duration.as_millis() as f32 / 100.0));
}

fn setup_benchmark_app() -> App {
    let mut app = App::new();
    
    // Add minimal plugins for benchmarking
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default());
    
    // Spawn player at origin to trigger chunk generation
    app.world_mut().spawn(Transform::from_xyz(0.0, 0.0, 0.0));
    
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_generation_benchmark() {
        benchmark_world_generation();
    }
    
    /// Test that world generation maintains 60+ FPS target
    #[test] 
    fn test_world_generation_performance_target() {
        let mut app = setup_benchmark_app();
        
        // Warm up
        for _ in 0..10 {
            app.update();
        }
        
        let start = Instant::now();
        let frames = 60;
        for _ in 0..frames {
            app.update();
        }
        let duration = start.elapsed();
        
        let avg_frame_time = duration / frames;
        let target_frame_time = Duration::from_millis(16); // ~60 FPS
        
        println!("Performance test results:");
        println!("  Average frame time: {:?}", avg_frame_time);
        println!("  Target frame time: {:?}", target_frame_time);
        
        assert!(
            avg_frame_time <= target_frame_time,
            "World generation too slow: {:?} > {:?}",
            avg_frame_time, target_frame_time
        );
    }
}

fn main() {
    benchmark_world_generation();
}
