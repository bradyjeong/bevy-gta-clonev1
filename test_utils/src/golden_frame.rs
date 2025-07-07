use bevy::prelude::*;
use bevy::pbr::*;
use image::{ImageBuffer, Rgba, GenericImageView};
use std::path::Path;

/// Utilities for golden frame testing
pub struct GoldenFrameUtils;

impl GoldenFrameUtils {
    /// Create a deterministic test scene with fixed entities
    pub fn setup_deterministic_scene(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        scene_config: DeterministicSceneConfig,
    ) {
        // Fixed camera position
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(
                scene_config.camera_position.x,
                scene_config.camera_position.y,
                scene_config.camera_position.z,
            ).looking_at(scene_config.camera_target, Vec3::Y),
            Name::new("TestCamera"),
        ));

        // Fixed directional light
        commands.spawn((
            DirectionalLight {
                color: scene_config.light_color,
                illuminance: scene_config.light_intensity,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(
                scene_config.light_position.x,
                scene_config.light_position.y,
                scene_config.light_position.z,
            ).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("TestLight"),
        ));

        // Create test entities based on config
        for entity_config in &scene_config.entities {
            Self::spawn_test_entity(commands, meshes, materials, entity_config.clone());
        }
    }

    /// Spawn a single test entity
    fn spawn_test_entity(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        config: TestEntityConfig,
    ) {
        let mesh = match config.shape {
            TestShape::Cube(size) => meshes.add(Mesh::from(Cuboid::new(size.x, size.y, size.z))),
            TestShape::Sphere(radius) => meshes.add(Mesh::from(Sphere::new(radius))),
            TestShape::Plane(size) => meshes.add(Mesh::from(Plane3d::default().mesh().size(size.x, size.y))),
        };

        let material = materials.add(StandardMaterial {
            base_color: config.color,
            metallic: config.metallic,
            perceptual_roughness: config.roughness,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(
                config.position.x,
                config.position.y,
                config.position.z,
            ),
            Name::new(config.name),
        ));
    }

    /// Compare two images with tolerance
    pub fn compare_images(
        reference_path: &Path,
        current_path: &Path,
        epsilon: f32,
    ) -> Result<ImageComparisonResult, Box<dyn std::error::Error>> {
        let reference = image::open(reference_path)?;
        let current = image::open(current_path)?;
        
        if reference.dimensions() != current.dimensions() {
            return Err("Image dimensions don't match".into());
        }
        
        let reference_rgba = reference.to_rgba8();
        let current_rgba = current.to_rgba8();
        
        let mut diff_pixels = 0u32;
        let mut total_diff = 0f32;
        let total_pixels = reference_rgba.width() * reference_rgba.height();
        
        for (ref_pixel, cur_pixel) in reference_rgba.pixels().zip(current_rgba.pixels()) {
            let diff = Self::pixel_difference(ref_pixel, cur_pixel);
            if diff > epsilon {
                diff_pixels += 1;
            }
            total_diff += diff;
        }
        
        let avg_diff = total_diff / total_pixels as f32;
        let diff_percentage = (diff_pixels as f32 / total_pixels as f32) * 100.0;
        
        Ok(ImageComparisonResult {
            diff_pixels,
            total_pixels,
            avg_diff,
            diff_percentage,
            passed: diff_pixels == 0 || (diff_percentage < 5.0 && avg_diff < epsilon),
        })
    }

    /// Calculate normalized difference between two pixels
    fn pixel_difference(a: &Rgba<u8>, b: &Rgba<u8>) -> f32 {
        let r_diff = (a[0] as f32 - b[0] as f32).abs() / 255.0;
        let g_diff = (a[1] as f32 - b[1] as f32).abs() / 255.0;
        let b_diff = (a[2] as f32 - b[2] as f32).abs() / 255.0;
        let a_diff = (a[3] as f32 - b[3] as f32).abs() / 255.0;
        
        (r_diff + g_diff + b_diff + a_diff) / 4.0
    }

    /// Generate a diff image highlighting differences
    pub fn generate_diff_image(
        reference_path: &Path,
        current_path: &Path,
        diff_path: &Path,
        epsilon: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reference = image::open(reference_path)?.to_rgba8();
        let current = image::open(current_path)?.to_rgba8();
        
        let mut diff_image = ImageBuffer::new(reference.width(), reference.height());
        
        for (x, y, pixel) in diff_image.enumerate_pixels_mut() {
            let ref_pixel = reference.get_pixel(x, y);
            let cur_pixel = current.get_pixel(x, y);
            let diff = Self::pixel_difference(ref_pixel, cur_pixel);
            
            if diff > epsilon {
                // Highlight differences in red
                *pixel = Rgba([255, 0, 0, 255]);
            } else {
                // Show original pixel with reduced opacity
                *pixel = Rgba([
                    ref_pixel[0] / 2,
                    ref_pixel[1] / 2,
                    ref_pixel[2] / 2,
                    128,
                ]);
            }
        }
        
        diff_image.save(diff_path)?;
        Ok(())
    }
}

/// Configuration for deterministic test scenes
#[derive(Debug, Clone)]
pub struct DeterministicSceneConfig {
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub light_position: Vec3,
    pub light_color: Color,
    pub light_intensity: f32,
    pub entities: Vec<TestEntityConfig>,
}

impl Default for DeterministicSceneConfig {
    fn default() -> Self {
        Self {
            camera_position: Vec3::new(10.0, 8.0, 10.0),
            camera_target: Vec3::ZERO,
            light_position: Vec3::new(4.0, 8.0, 4.0),
            light_color: Color::WHITE,
            light_intensity: 10000.0,
            entities: vec![
                // Default car
                TestEntityConfig {
                    name: "TestCar".to_string(),
                    shape: TestShape::Cube(Vec3::new(2.0, 1.0, 4.0)),
                    position: Vec3::new(0.0, 0.5, 0.0),
                    color: Color::srgb(0.8, 0.2, 0.2),
                    metallic: 0.3,
                    roughness: 0.7,
                },
                // Default building
                TestEntityConfig {
                    name: "TestBuilding".to_string(),
                    shape: TestShape::Cube(Vec3::new(3.0, 5.0, 3.0)),
                    position: Vec3::new(-5.0, 2.5, -3.0),
                    color: Color::srgb(0.6, 0.6, 0.8),
                    metallic: 0.1,
                    roughness: 0.9,
                },
                // Ground plane
                TestEntityConfig {
                    name: "TestGround".to_string(),
                    shape: TestShape::Plane(Vec2::new(20.0, 20.0)),
                    position: Vec3::new(0.0, -0.5, 0.0),
                    color: Color::srgb(0.3, 0.5, 0.3),
                    metallic: 0.0,
                    roughness: 1.0,
                },
            ],
        }
    }
}

/// Configuration for individual test entities
#[derive(Debug, Clone)]
pub struct TestEntityConfig {
    pub name: String,
    pub shape: TestShape,
    pub position: Vec3,
    pub color: Color,
    pub metallic: f32,
    pub roughness: f32,
}

/// Supported test shapes
#[derive(Debug, Clone)]
pub enum TestShape {
    Cube(Vec3),
    Sphere(f32),
    Plane(Vec2),
}

/// Result of image comparison
#[derive(Debug)]
pub struct ImageComparisonResult {
    pub diff_pixels: u32,
    pub total_pixels: u32,
    pub avg_diff: f32,
    pub diff_percentage: f32,
    pub passed: bool,
}

impl ImageComparisonResult {
    pub fn print_summary(&self) {
        println!("Golden Frame Comparison Results:");
        println!("  Total pixels: {}", self.total_pixels);
        println!("  Different pixels: {}", self.diff_pixels);
        println!("  Difference percentage: {:.2}%", self.diff_percentage);
        println!("  Average difference: {:.4}", self.avg_diff);
        println!("  Test passed: {}", self.passed);
    }
}
