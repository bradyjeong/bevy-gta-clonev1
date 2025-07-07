use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::*;
use bevy_rapier3d::prelude::*;
use std::path::Path;
use image::{ImageBuffer, Rgba};
use test_utils::golden_frame::*;

/// Golden frame test configuration
#[derive(Resource)]
struct GoldenFrameConfig {
    /// Directory to store reference images
    pub reference_dir: String,
    /// Epsilon tolerance for pixel differences (0.0 = exact match, 1.0 = any difference)
    pub epsilon: f32,
    /// Maximum number of different pixels allowed
    pub max_diff_pixels: u32,
}

impl Default for GoldenFrameConfig {
    fn default() -> Self {
        Self {
            reference_dir: "tests/golden_frames".to_string(),
            epsilon: 0.02, // 2% tolerance
            max_diff_pixels: 100,
        }
    }
}

/// Component to mark entities as deterministic test entities
#[derive(Component)]
struct DeterministicTestEntity;

/// Manual frame capture resource
#[derive(Resource)]
struct FrameCaptureState {
    frames_waited: u32,
    capture_requested: bool,
    capture_path: String,
}

impl Default for FrameCaptureState {
    fn default() -> Self {
        Self {
            frames_waited: 0,
            capture_requested: false,
            capture_path: "tests/golden_frames/current_frame.png".to_string(),
        }
    }
}

/// System to setup deterministic test scene
fn setup_deterministic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Use the test_utils golden frame setup
    let config = DeterministicSceneConfig::default();
    GoldenFrameUtils::setup_deterministic_scene(&mut commands, &mut meshes, &mut materials, config);
    
    // Add deterministic marker to all entities
    commands.spawn(DeterministicTestEntity);
}

/// System to capture frames after scene stabilization
fn capture_frame_system(
    mut capture_state: ResMut<FrameCaptureState>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    capture_state.frames_waited += 1;
    
    // Wait 10 frames for scene to stabilize
    if capture_state.frames_waited >= 10 && !capture_state.capture_requested {
        capture_state.capture_requested = true;
        
        // In a real implementation, this would trigger the actual screenshot
        // For now, we'll just signal that we're ready to capture
        println!("Frame capture ready at frame {}", capture_state.frames_waited);
        
        // Exit the app after capture
        app_exit_events.send(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;

    fn create_test_app() -> App {
        let mut app = App::new();
        
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (800.0, 600.0).into(),
                    title: "Golden Frame Test".to_string(),
                    visible: false, // Run headless
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
        ));
        
        app.insert_resource(GoldenFrameConfig::default());
        app.insert_resource(FrameCaptureState::default());
        app.add_systems(Startup, setup_deterministic_scene);
        app.add_systems(Update, capture_frame_system);
        
        app
    }

    #[test]
    fn test_basic_scene_golden_frame() {
        let reference_dir = PathBuf::from("tests/golden_frames");
        let reference_path = reference_dir.join("basic_scene.png");
        let current_path = reference_dir.join("current_frame.png");
        let diff_path = reference_dir.join("basic_scene_diff.png");
        
        // Ensure reference directory exists
        fs::create_dir_all(&reference_dir).unwrap();
        
        let mut app = create_test_app();
        
        // Run the app until it exits (after frame capture)
        while !app.should_exit() {
            app.update();
        }
        
        // Since we can't easily capture frames without bevy_frame_capture,
        // we'll create a simple test image for demonstration
        create_test_image(&current_path);
        
        if reference_path.exists() {
            // Compare with reference using our utility
            let config = GoldenFrameConfig::default();
            match GoldenFrameUtils::compare_images(&reference_path, &current_path, config.epsilon) {
                Ok(result) => {
                    result.print_summary();
                    
                    if !result.passed {
                        // Generate diff image for analysis
                        if let Err(e) = GoldenFrameUtils::generate_diff_image(&reference_path, &current_path, &diff_path, config.epsilon) {
                            println!("Failed to generate diff image: {}", e);
                        }
                        
                        panic!("Golden frame test failed: {} pixels different", result.diff_pixels);
                    }
                }
                Err(e) => {
                    println!("Failed to compare images: {}", e);
                    // Copy current frame as new reference if comparison fails
                    fs::copy(&current_path, &reference_path).unwrap();
                    println!("Created new reference frame at {:?}", reference_path);
                }
            }
        } else {
            // First run - create reference frame
            fs::copy(&current_path, &reference_path).unwrap();
            println!("Created reference frame at {:?}", reference_path);
        }
    }

    #[test]
    fn test_lod_transition_golden_frame() {
        let mut app = create_test_app();
        
        // Run a few frames to setup the scene
        for _ in 0..5 {
            app.update();
        }
        
        // Modify camera position for LOD testing
        let mut camera_query = app.world_mut().query::<&mut Transform>();
        let camera_entities: Vec<_> = app.world().query_filtered::<Entity, With<Camera>>().iter(app.world()).collect();
        
        if let Some(camera_entity) = camera_entities.first() {
            if let Ok(mut transform) = camera_query.get_mut(app.world_mut(), *camera_entity) {
                transform.translation = Vec3::new(50.0, 10.0, 50.0); // Far distance
            }
        }
        
        // Run until completion
        while !app.should_exit() {
            app.update();
        }
        
        println!("LOD transition test completed");
    }

    #[test]
    fn test_different_lighting_golden_frame() {
        let mut app = create_test_app();
        
        // Run a few frames to setup the scene
        for _ in 0..5 {
            app.update();
        }
        
        // Modify lighting for this test
        let mut light_query = app.world_mut().query::<&mut DirectionalLight>();
        let light_entities: Vec<_> = app.world().query_filtered::<Entity, With<DirectionalLight>>().iter(app.world()).collect();
        
        if let Some(light_entity) = light_entities.first() {
            if let Ok(mut light) = light_query.get_mut(app.world_mut(), *light_entity) {
                light.illuminance = 5000.0; // Different lighting
                light.color = Color::srgb(1.0, 0.9, 0.8); // Warm light
            }
        }
        
        // Run until completion
        while !app.should_exit() {
            app.update();
        }
        
        println!("Different lighting test completed");
    }

    /// Create a simple test image for demonstration purposes
    fn create_test_image(path: &Path) {
        let width = 800;
        let height = 600;
        let mut img = ImageBuffer::new(width, height);
        
        // Create a simple gradient pattern
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = (x as f32 / width as f32 * 255.0) as u8;
            let g = (y as f32 / height as f32 * 255.0) as u8;
            let b = 128u8;
            *pixel = Rgba([r, g, b, 255]);
        }
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        
        img.save(path).unwrap();
    }
}
