use bevy::prelude::*;
use std::collections::HashMap;

/// Material registry to prevent duplicate StandardMaterial creation
/// Key performance optimization for world generation
#[derive(Resource, Default)]
pub struct MaterialRegistry {
    materials: HashMap<MaterialKey, Handle<StandardMaterial>>,
}

/// Unique key for material properties to enable caching
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct MaterialKey {
    /// Base color as RGBA bytes for precise matching
    pub base_color: [u8; 4],
    /// Roughness scaled to u8 (0-255)
    pub roughness: u8,
    /// Metallic scaled to u8 (0-255)
    pub metallic: u8,
    /// Reflectance scaled to u8 (0-255)  
    pub reflectance: u8,
    /// Emissive flag for special materials
    pub emissive: bool,
}

impl MaterialKey {
    /// Create key from StandardMaterial properties
    pub fn from_color(color: Color) -> Self {
        let rgba = color.to_srgba();
        Self {
            base_color: [
                (rgba.red * 255.0) as u8,
                (rgba.green * 255.0) as u8,
                (rgba.blue * 255.0) as u8,
                (rgba.alpha * 255.0) as u8,
            ],
            roughness: (0.9 * 255.0) as u8, // Default roughness
            metallic: 0,
            reflectance: (0.04 * 255.0) as u8, // Default reflectance
            emissive: false,
        }
    }

    /// Create key with custom properties
    pub fn new(
        color: Color,
        roughness: f32,
        metallic: f32,
        reflectance: f32,
        emissive: bool,
    ) -> Self {
        let rgba = color.to_srgba();
        Self {
            base_color: [
                (rgba.red * 255.0) as u8,
                (rgba.green * 255.0) as u8,
                (rgba.blue * 255.0) as u8,
                (rgba.alpha * 255.0) as u8,
            ],
            roughness: (roughness.clamp(0.0, 1.0) * 255.0) as u8,
            metallic: (metallic.clamp(0.0, 1.0) * 255.0) as u8,
            reflectance: (reflectance.clamp(0.0, 1.0) * 255.0) as u8,
            emissive,
        }
    }

    /// Road material key
    pub fn road(color: Color) -> Self {
        Self::new(color, 0.9, 0.0, 0.04, false)
    }

    /// Road marking material key (emissive for visibility)
    pub fn road_marking(color: Color) -> Self {
        Self::new(color, 0.5, 0.0, 0.04, true)
    }

    /// Building material key
    pub fn building(color: Color) -> Self {
        Self::new(color, 0.7, 0.1, 0.04, false)
    }

    /// Vegetation material key
    pub fn vegetation(color: Color) -> Self {
        Self::new(color, 0.8, 0.0, 0.04, false)
    }

    /// Builder method to set custom roughness
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = (roughness.clamp(0.0, 1.0) * 255.0) as u8;
        self
    }
}

impl MaterialRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    /// Get or create material with given key
    /// Returns existing handle if material already exists, creates new one if not
    pub fn get_or_create(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        key: MaterialKey,
    ) -> Handle<StandardMaterial> {
        if let Some(handle) = self.materials.get(&key) {
            return handle.clone();
        }

        // Create new material from key
        let material = self.create_material_from_key(&key);
        let handle = materials.add(material);
        self.materials.insert(key, handle.clone());
        handle
    }

    /// Helper to create StandardMaterial from MaterialKey
    fn create_material_from_key(&self, key: &MaterialKey) -> StandardMaterial {
        let color = Color::srgba_u8(
            key.base_color[0],
            key.base_color[1],
            key.base_color[2],
            key.base_color[3],
        );

        let mut material = StandardMaterial {
            base_color: color,
            perceptual_roughness: key.roughness as f32 / 255.0,
            metallic: key.metallic as f32 / 255.0,
            reflectance: key.reflectance as f32 / 255.0,
            ..default()
        };

        if key.emissive {
            material.emissive = LinearRgba::from(color) * 0.3;
        }

        material
    }

    /// Get registry statistics for debugging
    pub fn stats(&self) -> MaterialRegistryStats {
        MaterialRegistryStats {
            cached_materials: self.materials.len(),
        }
    }
}

/// Statistics for material registry performance monitoring
#[derive(Debug)]
pub struct MaterialRegistryStats {
    pub cached_materials: usize,
}
