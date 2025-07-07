use bevy::prelude::*;
use game_core::prelude::*;

/// Bundle specification trait for generic entity creation
pub trait BundleSpec: Send + Sync + 'static {
    type Bundle: Bundle;
    fn create_bundle(&self, position: Vec3) -> Self::Bundle;
}

/// Errors that can occur during bundle creation
#[derive(Debug, Clone)]
pub enum BundleError {
    InvalidPosition,
    InvalidSize { size: Vec3, min_size: f32, max_size: f32 },
    InvalidMass { mass: f32, min_mass: f32, max_mass: f32 },
    NotImplemented,
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::InvalidPosition => write!(f, "Invalid position for entity"),
            BundleError::InvalidSize { size, min_size, max_size } => {
                write!(f, "Invalid size {:?}, must be between {} and {}", size, min_size, max_size)
            }
            BundleError::InvalidMass { mass, min_mass, max_mass } => {
                write!(f, "Invalid mass {}, must be between {} and {}", mass, min_mass, max_mass)
            }
            BundleError::NotImplemented => write!(f, "Feature not yet implemented"),
        }
    }
}

impl std::error::Error for BundleError {}

/// Generic bundle factory for creating different types of entities
#[derive(Resource)]
pub struct GenericBundleFactory {
    physics_config: PhysicsConfig,
}

impl GenericBundleFactory {
    pub fn new(config: &GameConfig) -> Self {
        Self {
            physics_config: config.physics.clone(),
        }
    }

    /// Validate position is within safe bounds
    pub fn validate_position(&self, position: Vec3) -> Result<Vec3, BundleError> {
        let max_coord = self.physics_config.max_world_coord;
        let min_coord = self.physics_config.min_world_coord;
        
        if position.x < min_coord || position.x > max_coord ||
           position.z < min_coord || position.z > max_coord ||
           position.y < 0.0 || position.y > 1000.0 {
            return Err(BundleError::InvalidPosition);
        }
        
        Ok(Vec3::new(
            position.x.clamp(min_coord, max_coord),
            position.y.clamp(0.0, 1000.0),
            position.z.clamp(min_coord, max_coord),
        ))
    }

    /// Validate collider size
    pub fn validate_collider_size(&self, size: Vec3) -> Result<Vec3, BundleError> {
        let min_size = self.physics_config.min_collider_size;
        let max_size = self.physics_config.max_collider_size;
        
        if size.x < min_size || size.x > max_size ||
           size.y < min_size || size.y > max_size ||
           size.z < min_size || size.z > max_size {
            return Err(BundleError::InvalidSize { size, min_size, max_size });
        }
        
        Ok(size)
    }

    /// Validate mass
    pub fn validate_mass(&self, mass: f32) -> Result<f32, BundleError> {
        let min_mass = self.physics_config.min_mass;
        let max_mass = self.physics_config.max_mass;
        
        if mass < min_mass || mass > max_mass {
            return Err(BundleError::InvalidMass { mass, min_mass, max_mass });
        }
        
        Ok(mass)
    }

    /// Create a basic vehicle bundle
    pub fn create_vehicle_bundle(
        &self,
        position: Vec3,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
    ) -> Result<(
        Vehicle,
        Cullable,
        MaterialMeshBundle<StandardMaterial>,
    ), BundleError> {
        let safe_position = self.validate_position(position)?;
        
        let vehicle = Vehicle {
            max_speed: 60.0,
            current_speed: 0.0,
            fuel: 100.0,
            engine_power: 150.0,
            vehicle_type: VehicleType::Car,
            spawn_time: 0.0,
        };
        
        let cullable = Cullable {
            is_culled: false,
            max_distance: 150.0,
        };
        
        let mesh_bundle = MaterialMeshBundle {
            mesh,
            material,
            transform: Transform::from_translation(safe_position),
            ..default()
        };
        
        Ok((vehicle, cullable, mesh_bundle))
    }

    /// Create a basic building bundle
    pub fn create_building_bundle(
        &self,
        position: Vec3,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
    ) -> Result<(
        Building,
        Cullable,
        MaterialMeshBundle<StandardMaterial>,
    ), BundleError> {
        let safe_position = self.validate_position(position)?;
        
        let building = Building {
            building_type: BuildingType::Residential,
            height: 12.0,
            max_occupants: 4,
            current_occupants: 0,
            spawn_time: 0.0,
        };
        
        let cullable = Cullable {
            is_culled: false,
            max_distance: 300.0,
        };
        
        let mesh_bundle = MaterialMeshBundle {
            mesh,
            material,
            transform: Transform::from_translation(safe_position),
            ..default()
        };
        
        Ok((building, cullable, mesh_bundle))
    }

    /// Create a basic NPC bundle
    pub fn create_npc_bundle(
        &self,
        position: Vec3,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
    ) -> Result<(
        NPC,
        Cullable,
        MaterialMeshBundle<StandardMaterial>,
    ), BundleError> {
        let safe_position = self.validate_position(position)?;
        
        let npc = NPC {
            health: 100.0,
            max_health: 100.0,
            speed: 2.0,
            behavior_state: NPCBehaviorState::Idle,
            spawn_time: 0.0,
        };
        
        let cullable = Cullable {
            is_culled: false,
            max_distance: 100.0,
        };
        
        let mesh_bundle = MaterialMeshBundle {
            mesh,
            material,
            transform: Transform::from_translation(safe_position),
            ..default()
        };
        
        Ok((npc, cullable, mesh_bundle))
    }
}

impl Default for GenericBundleFactory {
    fn default() -> Self {
        Self {
            physics_config: PhysicsConfig::default(),
        }
    }
}
