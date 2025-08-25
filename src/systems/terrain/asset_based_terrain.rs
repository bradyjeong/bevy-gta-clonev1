use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Asset-driven terrain configuration system
/// 
/// This system loads terrain settings from RON files instead of hardcoding them.
/// Benefits:
/// - No code changes needed for terrain parameter tweaks
/// - Easy to customize terrain without recompilation
/// - Single source of truth for all terrain settings
/// - Supports runtime terrain customization

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterArea {
    pub center: (f32, f32),
    pub radius: f32,
    pub depth: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub max_terrain_distance: f32,
    pub lod_levels: u32,
    pub enable_frustum_culling: bool,
    pub chunk_cache_size: usize,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            max_terrain_distance: 2000.0,
            lod_levels: 4,
            enable_frustum_culling: true,
            chunk_cache_size: 64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSettings {
    pub noise_octaves: u32,
    pub noise_frequency: f32,
    pub noise_persistence: f32,
    pub noise_lacunarity: f32,
    pub terrain_smoothing: bool,
    pub edge_falloff: f32,
}

impl Default for GenerationSettings {
    fn default() -> Self {
        Self {
            noise_octaves: 4,
            noise_frequency: 0.01,
            noise_persistence: 0.5,
            noise_lacunarity: 2.0,
            terrain_smoothing: true,
            edge_falloff: 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct TerrainConfig {
    pub world_size: f32,
    pub resolution: u32,
    pub base_height: f32,
    pub hill_scale: f32,
    pub noise_seed: u32,
    pub chunk_size: f32,
    pub water_areas: Vec<WaterArea>,
    pub performance: PerformanceSettings,
    pub generation: GenerationSettings,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            world_size: 4096.0,
            resolution: 512,
            base_height: -0.15,
            hill_scale: 15.0,
            noise_seed: 12345,
            chunk_size: 64.0,
            water_areas: Vec::new(),
            performance: PerformanceSettings::default(),
            generation: GenerationSettings::default(),
        }
    }
}

impl TerrainConfig {
    /// Validate terrain configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.world_size <= 0.0 {
            return Err("World size must be positive".to_string());
        }
        
        if self.resolution < 32 || self.resolution > 2048 {
            return Err("Resolution must be between 32 and 2048".to_string());
        }
        
        if self.chunk_size <= 0.0 {
            return Err("Chunk size must be positive".to_string());
        }
        
        if self.hill_scale < 0.0 {
            return Err("Hill scale cannot be negative".to_string());
        }
        
        if self.performance.max_terrain_distance <= 0.0 {
            return Err("Max terrain distance must be positive".to_string());
        }
        
        if self.performance.lod_levels == 0 {
            return Err("LOD levels must be at least 1".to_string());
        }
        
        if self.generation.noise_octaves == 0 {
            return Err("Noise octaves must be at least 1".to_string());
        }
        
        if self.generation.noise_frequency <= 0.0 {
            return Err("Noise frequency must be positive".to_string());
        }
        
        Ok(())
    }
    
    /// Get water area at a specific world position
    pub fn get_water_area_at(&self, x: f32, z: f32) -> Option<&WaterArea> {
        self.water_areas.iter().find(|water| {
            let distance = ((x - water.center.0).powi(2) + (z - water.center.1).powi(2)).sqrt();
            distance <= water.radius
        })
    }
    
    /// Check if position is in water
    pub fn is_water_at(&self, x: f32, z: f32) -> bool {
        self.get_water_area_at(x, z).is_some()
    }
    
    /// Get terrain height with water areas considered
    /// Phase 3: Uses heightmap data when available, fallback to noise generation
    pub fn get_terrain_height_at(&self, x: f32, z: f32) -> f32 {
        if let Some(water) = self.get_water_area_at(x, z) {
            water.depth
        } else {
            // Use simple noise-based height for real-time queries
            // This matches the heightmap generation but is computed on demand
            self.get_noise_height_at(x, z)
        }
    }
    
    /// Generate height using same noise algorithm as heightmap generator
    /// Used for real-time height queries that match the generated terrain mesh
    pub fn get_noise_height_at(&self, x: f32, z: f32) -> f32 {
        use noise::{NoiseFn, Perlin};
        
        let perlin = Perlin::new(self.noise_seed);
        let mut height = 0.0;
        let mut amplitude = self.hill_scale;
        let mut frequency = self.generation.noise_frequency;
        
        // Generate fractal noise (same as heightmap generator)
        for _ in 0..self.generation.noise_octaves {
            height += perlin.get([x as f64 * frequency as f64, z as f64 * frequency as f64]) as f32 * amplitude;
            amplitude *= self.generation.noise_persistence;
            frequency *= self.generation.noise_lacunarity;
        }
        
        // Add base height
        height += self.base_height;
        
        // Apply edge falloff
        if self.generation.edge_falloff > 0.0 {
            let half_size = self.world_size * 0.5;
            let edge_dist = half_size - (x.abs().max(z.abs()));
            if edge_dist < self.generation.edge_falloff {
                let falloff_factor = (edge_dist / self.generation.edge_falloff).clamp(0.0, 1.0);
                height = self.base_height + (height - self.base_height) * falloff_factor;
            }
        }
        
        height
    }
}

#[derive(Resource, Default)]
pub struct LoadedTerrainConfig {
    pub config: Option<TerrainConfig>,
    pub loading: bool,
}

#[derive(Resource)]
pub struct TerrainConfigHandle(pub Handle<TerrainConfig>);

/// System to load terrain configuration from assets
pub fn load_terrain_config_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loaded_config: ResMut<LoadedTerrainConfig>,
) {
    if loaded_config.config.is_none() && !loaded_config.loading {
        info!("Loading terrain configuration from assets/config/terrain.ron");
        let handle: Handle<TerrainConfig> = asset_server.load("config/terrain.ron");
        commands.insert_resource(TerrainConfigHandle(handle));
        loaded_config.loading = true;
    }
}

/// System to process loaded terrain configuration assets
pub fn process_loaded_terrain_config_system(
    mut loaded_config: ResMut<LoadedTerrainConfig>,
    config_assets: Res<Assets<TerrainConfig>>,
    config_handle: Option<Res<TerrainConfigHandle>>,
) {
    if let Some(handle) = config_handle
        && let Some(config) = config_assets.get(&handle.0)
        && loaded_config.config.is_none()
    {
        // Validate configuration before using it
        match config.validate() {
            Ok(()) => {
                info!("Terrain configuration loaded and validated successfully!");
                info!("  World size: {}x{}", config.world_size, config.world_size);
                info!("  Resolution: {}x{}", config.resolution, config.resolution);
                info!("  Base height: {}", config.base_height);
                info!("  Hill scale: {}", config.hill_scale);
                info!("  Water areas: {}", config.water_areas.len());
                loaded_config.config = Some(config.clone());
                loaded_config.loading = false;
            }
            Err(error) => {
                error!("Invalid terrain configuration: {}", error);
                error!("Using default terrain configuration instead");
                loaded_config.config = Some(TerrainConfig::default());
                loaded_config.loading = false;
            }
        }
    }
}

/// Debug system to display loaded terrain configuration
pub fn debug_terrain_config_system(
    loaded_config: Res<LoadedTerrainConfig>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard_input.just_pressed(KeyCode::F4) {
        return;
    }

    if let Some(ref config) = loaded_config.config {
        info!("=== LOADED TERRAIN CONFIGURATION ===");
        info!("World size: {}x{}", config.world_size, config.world_size);
        info!("Resolution: {}x{}", config.resolution, config.resolution);
        info!("Base height: {}", config.base_height);
        info!("Hill scale: {}", config.hill_scale);
        info!("Noise seed: {}", config.noise_seed);
        info!("Chunk size: {}", config.chunk_size);
        info!("Water areas: {}", config.water_areas.len());
        for (i, water) in config.water_areas.iter().enumerate() {
            info!("  Water {}: {} at ({}, {}) radius={}", 
                  i + 1, water.description, water.center.0, water.center.1, water.radius);
        }
        info!("Max terrain distance: {}", config.performance.max_terrain_distance);
        info!("LOD levels: {}", config.performance.lod_levels);
        info!("Noise octaves: {}", config.generation.noise_octaves);
        info!("Noise frequency: {}", config.generation.noise_frequency);
    } else {
        info!("Terrain configuration not yet loaded");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_config_validation() {
        let mut config = TerrainConfig::default();
        assert!(config.validate().is_ok());

        config.world_size = -100.0;
        assert!(config.validate().is_err());

        config.world_size = 1000.0;
        config.resolution = 10;
        assert!(config.validate().is_err());

        config.resolution = 512;
        config.chunk_size = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_water_area_detection() {
        let config = TerrainConfig {
            water_areas: vec![WaterArea {
                center: (0.0, 0.0),
                radius: 10.0,
                depth: -5.0,
                description: "Test lake".to_string(),
            }],
            ..Default::default()
        };

        assert!(config.is_water_at(0.0, 0.0));
        assert!(config.is_water_at(5.0, 5.0)); // Inside radius
        assert!(!config.is_water_at(15.0, 15.0)); // Outside radius
    }

    #[test]
    fn test_terrain_height_calculation() {
        let config = TerrainConfig {
            base_height: -0.15,
            water_areas: vec![WaterArea {
                center: (10.0, 10.0),
                radius: 5.0,
                depth: -3.0,
                description: "Test water".to_string(),
            }],
            ..Default::default()
        };

        assert_eq!(config.get_terrain_height_at(0.0, 0.0), -0.15); // Land
        assert_eq!(config.get_terrain_height_at(10.0, 10.0), -3.0); // Water
    }
}
