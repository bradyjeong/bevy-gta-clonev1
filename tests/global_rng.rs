use bevy::prelude::*;
use gta_game::GlobalRng;

#[test]
fn test_global_rng_default() {
    let mut rng = GlobalRng::default();
    
    // Test basic functionality
    let value: f32 = rng.gen_range(0.0..1.0);
    assert!(value >= 0.0 && value < 1.0);
    
    let int_value: i32 = rng.gen_range(1..10);
    assert!(int_value >= 1 && int_value < 10);
}

#[test]
fn test_global_rng_as_bevy_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<GlobalRng>();
    
    // Verify resource is available
    let mut rng = app.world_mut().resource_mut::<GlobalRng>();
    let value = rng.gen_f32();
    assert!(value >= 0.0 && value < 1.0);
}

#[test]
fn test_global_rng_gen_f32() {
    let mut rng = GlobalRng::default();
    
    for _ in 0..100 {
        let value = rng.gen_f32();
        assert!(value >= 0.0 && value < 1.0, "Value {} out of range [0.0, 1.0)", value);
    }
}

#[test]
fn test_global_rng_deterministic_with_seed() {
    use rand::SeedableRng;
    
    let mut rng1 = GlobalRng(rand::rngs::StdRng::seed_from_u64(42));
    let mut rng2 = GlobalRng(rand::rngs::StdRng::seed_from_u64(42));
    
    // Same seed should produce same values
    for _ in 0..10 {
        assert_eq!(rng1.gen_f32(), rng2.gen_f32());
    }
}
