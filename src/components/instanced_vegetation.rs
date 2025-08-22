use bevy::prelude::*;

/// Instanced vegetation components for GPU batching
/// Reduces thousands of individual vegetation entities to instanced draws
///
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
    pub fn new(max_instances: usize) -> Self {
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

    pub fn is_full(&self) -> bool {
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
    pub fn new(max_instances: usize) -> Self {
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

    pub fn is_full(&self) -> bool {
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
    pub fn new(max_instances: usize) -> Self {
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

    pub fn is_full(&self) -> bool {
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
    pub fn new(max_instances: usize) -> Self {
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

    pub fn is_full(&self) -> bool {
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

/// Marker component for vegetation entities that need instancing
#[derive(Component, Debug, Clone)]
pub struct VegetationBatchable {
    pub vegetation_type: VegetationType,
    pub mesh_id: Option<Handle<Mesh>>,
    pub material_id: Option<Handle<StandardMaterial>>,
}

impl VegetationBatchable {
    pub fn palm_frond() -> Self {
        Self {
            vegetation_type: VegetationType::PalmFrond,
            mesh_id: None,
            material_id: None,
        }
    }

    pub fn leaf_cluster() -> Self {
        Self {
            vegetation_type: VegetationType::LeafCluster,
            mesh_id: None,
            material_id: None,
        }
    }

    pub fn tree_trunk() -> Self {
        Self {
            vegetation_type: VegetationType::TreeTrunk,
            mesh_id: None,
            material_id: None,
        }
    }

    pub fn bush() -> Self {
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
    pub fn new(name: &str, transform: Transform, vegetation_type: VegetationType) -> Self {
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
