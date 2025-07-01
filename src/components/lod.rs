use bevy::prelude::*;

/// Level of Detail component with vegetation-specific settings
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct VegetationLOD {
    pub detail_level: VegetationDetailLevel,
    pub distance_to_player: f32,
    pub last_update_frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VegetationDetailLevel {
    /// Full geometry, all details visible <50m
    Full,
    /// Medium detail geometry, reduced complexity <150m  
    Medium,
    /// Simple billboard texture >150m
    Billboard,
    /// Not visible, culled
    Culled,
}

impl VegetationLOD {
    pub fn new() -> Self {
        Self {
            detail_level: VegetationDetailLevel::Full,
            distance_to_player: 0.0,
            last_update_frame: 0,
        }
    }

    pub fn from_distance(distance: f32) -> Self {
        let detail_level = match distance {
            d if d < 50.0 => VegetationDetailLevel::Full,
            d if d < 150.0 => VegetationDetailLevel::Medium,
            d if d < 300.0 => VegetationDetailLevel::Billboard,
            _ => VegetationDetailLevel::Culled,
        };

        Self {
            detail_level,
            distance_to_player: distance,
            last_update_frame: 0,
        }
    }

    pub fn update_from_distance(&mut self, distance: f32, frame: u64) {
        let new_level = match distance {
            d if d < 50.0 => VegetationDetailLevel::Full,
            d if d < 150.0 => VegetationDetailLevel::Medium,
            d if d < 300.0 => VegetationDetailLevel::Billboard,
            _ => VegetationDetailLevel::Culled,
        };

        if new_level != self.detail_level {
            self.detail_level = new_level;
            self.last_update_frame = frame;
        }
        
        self.distance_to_player = distance;
    }

    pub fn should_be_visible(&self) -> bool {
        !matches!(self.detail_level, VegetationDetailLevel::Culled)
    }
}

impl Default for VegetationLOD {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker component for vegetation entities that can be rendered as billboards
#[derive(Component)]
pub struct VegetationBillboard {
    pub original_scale: Vec3,
    pub billboard_size: Vec2,
}

impl VegetationBillboard {
    pub fn new(original_scale: Vec3, billboard_size: Vec2) -> Self {
        Self {
            original_scale,
            billboard_size,
        }
    }
}

impl Default for VegetationBillboard {
    fn default() -> Self {
        Self {
            original_scale: Vec3::ONE,
            billboard_size: Vec2::new(2.0, 3.0), // Default tree-like proportions
        }
    }
}

/// Component to store multiple mesh handles for different LOD levels
#[derive(Component)]
pub struct VegetationMeshLOD {
    pub full_mesh: Handle<Mesh>,
    pub medium_mesh: Option<Handle<Mesh>>,
    pub billboard_mesh: Handle<Mesh>,
}

impl VegetationMeshLOD {
    pub fn new(
        full_mesh: Handle<Mesh>,
        medium_mesh: Option<Handle<Mesh>>,
        billboard_mesh: Handle<Mesh>,
    ) -> Self {
        Self {
            full_mesh,
            medium_mesh,
            billboard_mesh,
        }
    }

    pub fn get_mesh_for_level(&self, level: VegetationDetailLevel) -> Option<&Handle<Mesh>> {
        match level {
            VegetationDetailLevel::Full => Some(&self.full_mesh),
            VegetationDetailLevel::Medium => {
                self.medium_mesh.as_ref().unwrap_or(&self.full_mesh).into()
            }
            VegetationDetailLevel::Billboard => Some(&self.billboard_mesh),
            VegetationDetailLevel::Culled => None,
        }
    }
}
