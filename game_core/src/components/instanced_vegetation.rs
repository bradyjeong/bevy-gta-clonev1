use bevy::prelude::*;

/// Instanced vegetation components for GPU batching
/// Reduces thousands of individual vegetation entities to instanced draws

/// Instance data for vegetation rendering
#[derive(Component, Debug, Clone)]
pub struct InstanceData {
    pub transform: Transform,
    pub color: Vec4,
    pub scale_variation: f32,
    pub sway_offset: f32,
    pub age: f32,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            transform: Transform::IDENTITY,
            color: Vec4::ONE,
            scale_variation: 1.0,
            sway_offset: 0.0,
            age: 0.0,
        }
    }
}

/// Palm frond instancing component
#[derive(Component, Debug, Clone, Default)]
pub struct InstancedPalmFrond {
    pub instances: Vec<InstanceData>,
    pub max_instances: usize,
    pub dirty: bool,
}

impl InstancedPalmFrond {
    #[must_use] pub fn new(max_instances: usize) -> Self {
        Self {
            instances: Vec::with_capacity(max_instances),
            max_instances,
            dirty: true,
        }
    }

    pub fn add_instance(&mut self, instance: InstanceData) -> bool {
        if self.instances.len() < self.max_instances {
            self.instances.push(instance);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.instances.clear();
        self.dirty = true;
    }

    #[must_use] pub fn is_full(&self) -> bool {
        self.instances.len() >= self.max_instances
    }
}

/// Leaf cluster instancing component
#[derive(Component, Debug, Clone, Default)]
pub struct InstancedLeafCluster {
    pub instances: Vec<InstanceData>,
    pub max_instances: usize,
    pub dirty: bool,
}

impl InstancedLeafCluster {
    #[must_use] pub fn new(max_instances: usize) -> Self {
        Self {
            instances: Vec::with_capacity(max_instances),
            max_instances,
            dirty: true,
        }
    }

    pub fn add_instance(&mut self, instance: InstanceData) -> bool {
        if self.instances.len() < self.max_instances {
            self.instances.push(instance);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.instances.clear();
        self.dirty = true;
    }

    #[must_use] pub fn is_full(&self) -> bool {
        self.instances.len() >= self.max_instances
    }
}

/// Tree trunk instancing component
#[derive(Component, Debug, Clone, Default)]
pub struct InstancedTreeTrunk {
    pub instances: Vec<InstanceData>,
    pub max_instances: usize,
    pub dirty: bool,
}

impl InstancedTreeTrunk {
    #[must_use] pub fn new(max_instances: usize) -> Self {
        Self {
            instances: Vec::with_capacity(max_instances),
            max_instances,
            dirty: true,
        }
    }

    pub fn add_instance(&mut self, instance: InstanceData) -> bool {
        if self.instances.len() < self.max_instances {
            self.instances.push(instance);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.instances.clear();
        self.dirty = true;
    }

    #[must_use] pub fn is_full(&self) -> bool {
        self.instances.len() >= self.max_instances
    }
}

/// Bush/shrub instancing component
#[derive(Component, Debug, Clone, Default)]
pub struct InstancedBush {
    pub instances: Vec<InstanceData>,
    pub max_instances: usize,
    pub dirty: bool,
}

impl InstancedBush {
    #[must_use] pub fn new(max_instances: usize) -> Self {
        Self {
            instances: Vec::with_capacity(max_instances),
            max_instances,
            dirty: true,
        }
    }

    pub fn add_instance(&mut self, instance: InstanceData) -> bool {
        if self.instances.len() < self.max_instances {
            self.instances.push(instance);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.instances.clear();
        self.dirty = true;
    }

    #[must_use] pub fn is_full(&self) -> bool {
        self.instances.len() >= self.max_instances
    }
}

/// Vegetation instancing configuration
#[derive(Resource, Debug, Clone)]
pub struct VegetationInstancingConfig {
    pub palm_frond_batch_size: usize,
    pub leaf_cluster_batch_size: usize,
    pub tree_trunk_batch_size: usize,
    pub bush_batch_size: usize,
    pub max_instances_per_draw: usize,
    pub culling_distance: f32,
    pub update_interval: f32,
}

impl Default for VegetationInstancingConfig {
    fn default() -> Self {
        Self {
            palm_frond_batch_size: 256,
            leaf_cluster_batch_size: 512,
            tree_trunk_batch_size: 128,
            bush_batch_size: 384,
            max_instances_per_draw: 1024,
            culling_distance: 500.0,
            update_interval: 0.5, // Update every 0.5 seconds
        }
    }
}

/// Vegetation type identifier for batching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VegetationType {
    PalmFrond,
    LeafCluster,
    TreeTrunk,
    Bush,
}

/// Vegetation LOD detail levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum VegetationDetailLevel {
    #[default]
    Full,
    Medium,
    Billboard,
    Culled,
}


/// Vegetation LOD component
#[derive(Component, Debug, Clone)]
pub struct VegetationLOD {
    pub detail_level: VegetationDetailLevel,
    pub distance: f32,
    pub distance_to_player: f32,
    pub last_update: f32,
}

impl Default for VegetationLOD {
    fn default() -> Self {
        Self {
            detail_level: VegetationDetailLevel::Full,
            distance: 0.0,
            distance_to_player: 0.0,
            last_update: 0.0,
        }
    }
}

impl VegetationLOD {
    pub fn update_from_distance(&mut self, distance: f32, frame: u64) {
        self.distance_to_player = distance;
        self.distance = distance;
        self.last_update = frame as f32;
        
        // Update detail level based on distance
        self.detail_level = match distance {
            d if d < 50.0 => VegetationDetailLevel::Full,
            d if d < 150.0 => VegetationDetailLevel::Medium,
            d if d < 300.0 => VegetationDetailLevel::Billboard,
            _ => VegetationDetailLevel::Culled,
        };
    }
    
    #[must_use] pub fn should_be_visible(&self) -> bool {
        !matches!(self.detail_level, VegetationDetailLevel::Culled)
    }
}

/// Mesh LOD component for vegetation
#[derive(Component, Debug, Clone)]
#[derive(Default)]
pub struct VegetationMeshLOD {
    pub current_mesh: Handle<Mesh>,
    pub full_mesh: Handle<Mesh>,
    pub medium_mesh: Handle<Mesh>,
    pub billboard_mesh: Handle<Mesh>,
}


impl VegetationMeshLOD {
    #[must_use] pub fn get_mesh_for_level(&self, level: VegetationDetailLevel) -> Option<Handle<Mesh>> {
        match level {
            VegetationDetailLevel::Full => Some(self.full_mesh.clone()),
            VegetationDetailLevel::Medium => Some(self.medium_mesh.clone()),
            VegetationDetailLevel::Billboard => Some(self.billboard_mesh.clone()),
            VegetationDetailLevel::Culled => None,
        }
    }
}

/// Billboard component for vegetation
#[derive(Component, Debug, Clone)]
pub struct VegetationBillboard {
    pub texture: Handle<Image>,
    pub size: Vec2,
    pub always_face_camera: bool,
    pub original_scale: Vec3,
}

impl Default for VegetationBillboard {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            size: Vec2::new(1.0, 1.0),
            always_face_camera: true,
            original_scale: Vec3::ONE,
        }
    }
}

/// Marker component for vegetation entities that need instancing
#[derive(Component, Debug, Clone)]
pub struct VegetationBatchable {
    pub vegetation_type: VegetationType,
    pub mesh_id: Option<Handle<Mesh>>,
    pub material_id: Option<Handle<StandardMaterial>>,
}

impl VegetationBatchable {
    #[must_use] pub fn palm_frond() -> Self {
        Self {
            vegetation_type: VegetationType::PalmFrond,
            mesh_id: None,
            material_id: None,
        }
    }

    #[must_use] pub fn leaf_cluster() -> Self {
        Self {
            vegetation_type: VegetationType::LeafCluster,
            mesh_id: None,
            material_id: None,
        }
    }

    #[must_use] pub fn tree_trunk() -> Self {
        Self {
            vegetation_type: VegetationType::TreeTrunk,
            mesh_id: None,
            material_id: None,
        }
    }

    #[must_use] pub fn bush() -> Self {
        Self {
            vegetation_type: VegetationType::Bush,
            mesh_id: None,
            material_id: None,
        }
    }
}

/// Bundle for instanced vegetation entities
#[derive(Bundle)]
pub struct InstancedVegetationBundle {
    pub name: Name,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub batchable: VegetationBatchable,
}

impl InstancedVegetationBundle {
    #[must_use] pub fn new(name: &str, transform: Transform, vegetation_type: VegetationType) -> Self {
        Self {
            name: Name::new(name.to_string()),
            transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            batchable: match vegetation_type {
                VegetationType::PalmFrond => VegetationBatchable::palm_frond(),
                VegetationType::LeafCluster => VegetationBatchable::leaf_cluster(),
                VegetationType::TreeTrunk => VegetationBatchable::tree_trunk(),
                VegetationType::Bush => VegetationBatchable::bush(),
            },
        }
    }
}
