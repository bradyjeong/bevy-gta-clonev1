use bevy::prelude::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshShape {
    Cuboid {
        x_bits: u32,
        y_bits: u32,
        z_bits: u32,
    },
    Sphere {
        radius_bits: u32,
    },
    Capsule {
        radius_bits: u32,
        height_bits: u32,
    },
}

impl MeshShape {
    fn normalize_float(value: f32) -> u32 {
        if !value.is_finite() {
            #[cfg(feature = "debug-ui")]
            eprintln!("Warning: Invalid mesh dimension {value} (NaN/Inf), using 0.0");
            return 0.0f32.to_bits();
        }
        if value == 0.0 {
            0.0f32.to_bits()
        } else {
            value.to_bits()
        }
    }

    pub fn cuboid(x: f32, y: f32, z: f32) -> Self {
        Self::Cuboid {
            x_bits: Self::normalize_float(x),
            y_bits: Self::normalize_float(y),
            z_bits: Self::normalize_float(z),
        }
    }

    pub fn sphere(radius: f32) -> Self {
        Self::Sphere {
            radius_bits: Self::normalize_float(radius),
        }
    }

    pub fn capsule(radius: f32, height: f32) -> Self {
        Self::Capsule {
            radius_bits: Self::normalize_float(radius),
            height_bits: Self::normalize_float(height),
        }
    }

    fn create_mesh(&self) -> Mesh {
        match self {
            MeshShape::Cuboid {
                x_bits,
                y_bits,
                z_bits,
            } => Cuboid::new(
                f32::from_bits(*x_bits),
                f32::from_bits(*y_bits),
                f32::from_bits(*z_bits),
            )
            .into(),
            MeshShape::Sphere { radius_bits } => Sphere::new(f32::from_bits(*radius_bits)).into(),
            MeshShape::Capsule {
                radius_bits,
                height_bits,
            } => Capsule3d::new(f32::from_bits(*radius_bits), f32::from_bits(*height_bits)).into(),
        }
    }
}

#[derive(Resource)]
pub struct NPCAssetCache {
    meshes: HashMap<MeshShape, Handle<Mesh>>,
    materials: HashMap<[u8; 4], Handle<StandardMaterial>>,
    cache_hits: u32,
    cache_misses: u32,
}

impl NPCAssetCache {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::default(),
            materials: HashMap::default(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn get_or_create_mesh(
        &mut self,
        shape: MeshShape,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        match self.meshes.entry(shape) {
            Entry::Occupied(e) => {
                self.cache_hits += 1;
                e.get().clone()
            }
            Entry::Vacant(e) => {
                self.cache_misses += 1;
                let mesh = meshes.add(shape.create_mesh());
                e.insert(mesh).clone()
            }
        }
    }

    pub fn get_or_create_material(
        &mut self,
        color: Color,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        let key = color.to_srgba().to_u8_array();

        match self.materials.entry(key) {
            Entry::Occupied(e) => {
                self.cache_hits += 1;
                e.get().clone()
            }
            Entry::Vacant(e) => {
                self.cache_misses += 1;
                let material = materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                });
                e.insert(material).clone()
            }
        }
    }

    pub fn initialize_common_assets(
        &mut self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        let skin_tones = [
            Color::srgb(0.8, 0.6, 0.4),
            Color::srgb(0.6, 0.4, 0.3),
            Color::srgb(0.9, 0.7, 0.5),
            Color::srgb(0.7, 0.5, 0.4),
        ];

        let shirt_colors = [
            Color::srgb(1.0, 0.0, 0.0),
            Color::srgb(0.0, 0.0, 1.0),
            Color::srgb(0.0, 1.0, 0.0),
            Color::srgb(1.0, 1.0, 0.0),
            Color::srgb(0.5, 0.5, 0.5),
        ];

        let pants_colors = [
            Color::srgb(0.2, 0.2, 0.8),
            Color::srgb(0.1, 0.1, 0.1),
            Color::srgb(0.4, 0.4, 0.4),
            Color::srgb(0.3, 0.2, 0.1),
        ];

        let shoe_color = Color::srgb(0.1, 0.1, 0.1);

        for &color in &skin_tones {
            self.get_or_create_material(color, materials);
        }
        for &color in &shirt_colors {
            self.get_or_create_material(color, materials);
        }
        for &color in &pants_colors {
            self.get_or_create_material(color, materials);
        }
        self.get_or_create_material(shoe_color, materials);

        self.get_or_create_mesh(MeshShape::cuboid(0.6, 0.8, 0.3), meshes);
        self.get_or_create_mesh(MeshShape::sphere(0.2), meshes);
        self.get_or_create_mesh(MeshShape::capsule(0.08, 0.5), meshes);
        self.get_or_create_mesh(MeshShape::capsule(0.12, 0.6), meshes);
        self.get_or_create_mesh(MeshShape::cuboid(0.2, 0.1, 0.35), meshes);

        #[cfg(feature = "debug-ui")]
        info!(
            "NPC Asset Cache initialized: {} meshes, {} materials",
            self.meshes.len(),
            self.materials.len()
        );
    }

    pub fn stats(&self) -> (u32, u32, f32) {
        let total = self.cache_hits + self.cache_misses;
        let hit_rate = if total > 0 {
            (self.cache_hits as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        (self.cache_hits, self.cache_misses, hit_rate)
    }
}

impl Default for NPCAssetCache {
    fn default() -> Self {
        Self::new()
    }
}
