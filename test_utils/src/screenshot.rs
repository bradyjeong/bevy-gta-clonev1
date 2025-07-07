use bevy::prelude::*;
use image::{ImageBuffer, Rgba, RgbaImage};
use std::path::Path;

/// Configuration for screenshot comparison
#[derive(Clone)]
pub struct ScreenshotConfig {
    /// Width of the screenshot in pixels
    pub width: u32,
    /// Height of the screenshot in pixels
    pub height: u32,
    /// Acceptable difference tolerance (0.0 to 1.0)
    pub tolerance: f32,
    /// Whether to save difference images when comparison fails
    pub save_diff: bool,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            tolerance: 0.01, // 1% tolerance
            save_diff: true,
        }
    }
}

/// Result of screenshot comparison
pub struct ComparisonResult {
    /// Whether the images match within tolerance
    pub matches: bool,
    /// Overall difference percentage (0.0 to 1.0)
    pub difference_percentage: f32,
    /// Maximum pixel difference found
    pub max_pixel_difference: f32,
}

/// Compare two screenshots for golden-frame testing
pub fn compare_screenshot<P: AsRef<Path>>(
    test_image_path: P,
    reference_image_path: P,
    config: &ScreenshotConfig,
) -> Result<ComparisonResult, String> {
    let test_image = load_image(test_image_path.as_ref())?;
    let reference_image = load_image(reference_image_path.as_ref())?;
    
    if test_image.width() != reference_image.width() || test_image.height() != reference_image.height() {
        return Err(format!(
            "Image dimensions don't match: test {}x{}, reference {}x{}",
            test_image.width(), test_image.height(),
            reference_image.width(), reference_image.height()
        ));
    }
    
    let (difference_percentage, max_pixel_difference) = calculate_difference(&test_image, &reference_image);
    
    let matches = difference_percentage <= config.tolerance;
    
    // Save difference image if requested and images don't match
    if config.save_diff && !matches {
        let diff_path = test_image_path.as_ref().with_extension("diff.png");
        save_difference_image(&test_image, &reference_image, &diff_path)?;
    }
    
    Ok(ComparisonResult {
        matches,
        difference_percentage,
        max_pixel_difference,
    })
}

/// Load an image from file
fn load_image<P: AsRef<Path>>(path: P) -> Result<RgbaImage, String> {
    Ok(image::open(path)
        .map_err(|e| format!("Failed to load image: {}", e))?
        .to_rgba8())
}

/// Calculate the difference between two images
fn calculate_difference(img1: &RgbaImage, img2: &RgbaImage) -> (f32, f32) {
    let mut total_difference = 0.0f32;
    let mut max_pixel_difference = 0.0f32;
    let pixel_count = (img1.width() * img1.height()) as f32;
    
    for (pixel1, pixel2) in img1.pixels().zip(img2.pixels()) {
        let diff = pixel_difference(pixel1, pixel2);
        total_difference += diff;
        max_pixel_difference = max_pixel_difference.max(diff);
    }
    
    let average_difference = total_difference / pixel_count;
    (average_difference, max_pixel_difference)
}

/// Calculate the difference between two pixels
fn pixel_difference(p1: &Rgba<u8>, p2: &Rgba<u8>) -> f32 {
    let r_diff = (p1[0] as f32 - p2[0] as f32).abs();
    let g_diff = (p1[1] as f32 - p2[1] as f32).abs();
    let b_diff = (p1[2] as f32 - p2[2] as f32).abs();
    let a_diff = (p1[3] as f32 - p2[3] as f32).abs();
    
    // Normalize to 0-1 range
    ((r_diff + g_diff + b_diff + a_diff) / 4.0) / 255.0
}

/// Save a difference image highlighting the differences
fn save_difference_image<P: AsRef<Path>>(
    img1: &RgbaImage,
    img2: &RgbaImage,
    path: P,
) -> Result<(), String> {
    let mut diff_image = ImageBuffer::new(img1.width(), img1.height());
    
    for (x, y, pixel) in diff_image.enumerate_pixels_mut() {
        let p1 = img1.get_pixel(x, y);
        let p2 = img2.get_pixel(x, y);
        let diff = pixel_difference(p1, p2);
        
        // Create a red-tinted pixel based on difference
        let intensity = (diff * 255.0) as u8;
        *pixel = Rgba([intensity, 0, 0, 255]);
    }
    
    diff_image.save(path)
        .map_err(|e| format!("Failed to save difference image: {}", e))?;
    
    Ok(())
}

/// Create a test screenshot with specific pattern for testing
pub fn create_test_pattern(width: u32, height: u32) -> RgbaImage {
    let mut image = ImageBuffer::new(width, height);
    
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let r = (x * 255 / width) as u8;
        let g = (y * 255 / height) as u8;
        let b = ((x + y) * 255 / (width + height)) as u8;
        *pixel = Rgba([r, g, b, 255]);
    }
    
    image
}

/// Save a screenshot to file
pub fn save_screenshot<P: AsRef<Path>>(image: &RgbaImage, path: P) -> Result<(), String> {
    image.save(path)
        .map_err(|e| format!("Failed to save screenshot: {}", e))
}

/// Helper for creating golden-frame test directories
pub fn ensure_golden_frame_dir<P: AsRef<Path>>(test_name: &str, base_path: P) -> Result<std::path::PathBuf, String> {
    let dir = base_path.as_ref().join("golden_frames").join(test_name);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create golden frame directory: {}", e))?;
    Ok(dir)
}

/// Golden frame test helper
pub struct GoldenFrameTest {
    /// Test name identifier
    pub name: String,
    /// Screenshot configuration
    pub config: ScreenshotConfig,
    /// Base path for test files
    pub base_path: std::path::PathBuf,
}

impl GoldenFrameTest {
    /// Creates a new screenshot test with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config: ScreenshotConfig::default(),
            base_path: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        }
    }
    
    /// Sets the configuration for the screenshot test
    pub fn with_config(mut self, config: ScreenshotConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Sets the base path for test files
    pub fn with_base_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.base_path = path.into();
        self
    }
    
    /// Runs the screenshot test against the given image
    pub fn run_test(&self, test_image: &RgbaImage) -> Result<ComparisonResult, String> {
        let golden_dir = ensure_golden_frame_dir(&self.name, &self.base_path)?;
        let reference_path = golden_dir.join("reference.png");
        let test_path = golden_dir.join("test.png");
        
        // Save the test image
        save_screenshot(test_image, &test_path)?;
        
        // If reference doesn't exist, create it
        if !reference_path.exists() {
            save_screenshot(test_image, &reference_path)?;
            return Ok(ComparisonResult {
                matches: true,
                difference_percentage: 0.0,
                max_pixel_difference: 0.0,
            });
        }
        
        // Compare with reference
        compare_screenshot(&test_path, &reference_path, &self.config)
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    
    fn create_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir().join("test_utils_screenshot_tests");
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }
    
    #[test]
    fn test_create_test_pattern() {
        let image = create_test_pattern(100, 100);
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);
    }
    
    #[test]
    fn test_pixel_difference() {
        let p1 = Rgba([255, 0, 0, 255]);
        let p2 = Rgba([0, 255, 0, 255]);
        let diff = pixel_difference(&p1, &p2);
        assert!(diff > 0.0);
    }
    
    #[test]
    fn test_save_and_load_screenshot() {
        let temp_dir = create_temp_dir();
        let test_path = temp_dir.join("test.png");
        
        let image = create_test_pattern(50, 50);
        save_screenshot(&image, &test_path).unwrap();
        
        let loaded = load_image(&test_path).unwrap();
        assert_eq!(loaded.width(), 50);
        assert_eq!(loaded.height(), 50);
    }
    
    #[test]
    fn test_compare_identical_images() {
        let temp_dir = create_temp_dir();
        let path1 = temp_dir.join("img1.png");
        let path2 = temp_dir.join("img2.png");
        
        let image = create_test_pattern(50, 50);
        save_screenshot(&image, &path1).unwrap();
        save_screenshot(&image, &path2).unwrap();
        
        let config = ScreenshotConfig::default();
        let result = compare_screenshot(&path1, &path2, &config).unwrap();
        
        assert!(result.matches);
        assert_eq!(result.difference_percentage, 0.0);
    }
    
    #[test]
    fn test_golden_frame_test() {
        let temp_dir = create_temp_dir();
        let test = GoldenFrameTest::new("test_pattern")
            .with_base_path(temp_dir);
        
        let image = create_test_pattern(50, 50);
        let result = test.run_test(&image).unwrap();
        
        assert!(result.matches);
    }
}
