use bevy::prelude::*;
use test_utils::golden_frame::*;
use std::path::PathBuf;
use std::fs;

/// Standalone golden frame test that doesn't depend on the main codebase modules
#[cfg(test)]
mod tests {
    #![deny(clippy::all, clippy::pedantic)]
    use super::*;

    fn create_minimal_test_app() -> App {
        let mut app = App::new();
        
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (800.0, 600.0).into(),
                title: "Golden Frame Test".to_string(),
                visible: false, // Run headless
                ..default()
            }),
            ..default()
        }));
        
        app
    }

    #[test]
    fn test_golden_frame_infrastructure() {
        let reference_dir = PathBuf::from("tests/golden_frames");
        let test_image_path = reference_dir.join("test_infrastructure.png");
        
        // Ensure directory exists
        fs::create_dir_all(&reference_dir).unwrap();
        
        // Create test images
        create_test_image(&test_image_path, 800, 600, |x, y| {
            let r = (x as f32 / 800.0 * 255.0) as u8;
            let g = (y as f32 / 600.0 * 255.0) as u8;
            let b = 128u8;
            image::Rgba([r, g, b, 255])
        });
        
        let test_image_path2 = reference_dir.join("test_infrastructure_similar.png");
        create_test_image(&test_image_path2, 800, 600, |x, y| {
            let r = (x as f32 / 800.0 * 255.0) as u8;
            let g = (y as f32 / 600.0 * 255.0) as u8;
            let b = 130u8; // Slightly different
            image::Rgba([r, g, b, 255])
        });
        
        // Test comparison
        let result = GoldenFrameUtils::compare_images(&test_image_path, &test_image_path2, 0.02)
            .expect("Should be able to compare images");
        
        result.print_summary();
        
        // Should detect differences but they should be small
        assert!(result.diff_pixels > 0, "Should detect some differences");
        assert!(result.avg_diff < 0.1, "Differences should be small");
        
        // Test diff image generation
        let diff_path = reference_dir.join("test_infrastructure_diff.png");
        GoldenFrameUtils::generate_diff_image(&test_image_path, &test_image_path2, &diff_path, 0.02)
            .expect("Should generate diff image");
        
        assert!(diff_path.exists(), "Diff image should be created");
        
        println!("Golden frame infrastructure test passed!");
    }

    #[test]
    fn test_deterministic_scene_config() {
        let config = DeterministicSceneConfig::default();
        
        // Verify default configuration
        assert_eq!(config.camera_position, Vec3::new(10.0, 8.0, 10.0));
        assert_eq!(config.camera_target, Vec3::ZERO);
        assert_eq!(config.light_position, Vec3::new(4.0, 8.0, 4.0));
        assert_eq!(config.light_intensity, 10000.0);
        assert_eq!(config.entities.len(), 3); // car, building, ground
        
        // Verify entity configurations
        let car = &config.entities[0];
        assert_eq!(car.name, "TestCar");
        assert_eq!(car.position, Vec3::new(0.0, 0.5, 0.0));
        
        let building = &config.entities[1];
        assert_eq!(building.name, "TestBuilding");
        assert_eq!(building.position, Vec3::new(-5.0, 2.5, -3.0));
        
        let ground = &config.entities[2];
        assert_eq!(ground.name, "TestGround");
        assert_eq!(ground.position, Vec3::new(0.0, -0.5, 0.0));
        
        println!("Deterministic scene config test passed!");
    }

    #[test]
    fn test_bevy_scene_setup() {
        let mut app = create_minimal_test_app();
        
        let config = DeterministicSceneConfig::default();
        
        // Setup the scene
        app.world_mut().run_system_once(|mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>| {
            GoldenFrameUtils::setup_deterministic_scene(&mut commands, &mut meshes, &mut materials, config);
        });
        
        // Run a few frames
        for _ in 0..5 {
            app.update();
        }
        
        // Verify entities were created
        let camera_count = app.world().query::<&Camera>().iter(app.world()).count();
        let light_count = app.world().query::<&DirectionalLight>().iter(app.world()).count();
        let pbr_count = app.world().query::<&Handle<StandardMaterial>>().iter(app.world()).count();
        
        assert_eq!(camera_count, 1, "Should have one camera");
        assert_eq!(light_count, 1, "Should have one directional light");
        assert_eq!(pbr_count, 3, "Should have three PBR entities (car, building, ground)");
        
        println!("Bevy scene setup test passed!");
    }

    /// Helper function to create test images
    fn create_test_image<F>(path: &std::path::Path, width: u32, height: u32, pixel_fn: F) 
    where
        F: Fn(u32, u32) -> image::Rgba<u8>,
    {
        use image::ImageBuffer;
        
        let mut img = ImageBuffer::new(width, height);
        
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = pixel_fn(x, y);
        }
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        
        img.save(path).unwrap();
    }
}
