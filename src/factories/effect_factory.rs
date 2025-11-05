use crate::config::GameConfig;
use crate::factories::generic_bundle::{BundleError, GenericBundleFactory, ParticleEffectType};
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;

/// Effect Factory - Focused factory for particle effects and visual effects spawning only
/// Handles explosions, sparks, and other visual effects
/// Follows AGENT.md simplicity principles with single responsibility
#[derive(Debug, Clone)]
pub struct EffectFactory {
    pub config: GameConfig,
}

impl EffectFactory {
    /// Create new effect factory with default configuration
    pub fn new() -> Self {
        Self {
            config: GameConfig::default(),
        }
    }

    /// Create effect factory with custom configuration
    pub fn with_config(config: GameConfig) -> Self {
        Self { config }
    }

    /// Spawn explosion effect
    pub fn spawn_explosion(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        scale: Option<f32>,
    ) -> Result<Entity, BundleError> {
        let scale = scale.unwrap_or(1.0);
        let mesh = meshes.add(Sphere::new(0.5 * scale));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.8, 0.0),
            emissive: LinearRgba::rgb(1.0, 0.5, 0.0),
            ..default()
        });

        let effect_entity = commands
            .spawn((
                Transform::from_translation(position),
                Visibility::default(),
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                ParticleEffect {
                    effect_type: ParticleEffectType::Explosion,
                    lifetime: 1.0,
                    age: 0.0,
                },
                VisibilityRange::abrupt(0.0, self.config.performance.vehicle_visibility_distance),
                Name::new("Explosion"),
            ))
            .id();

        Ok(effect_entity)
    }

    /// Spawn spark effect
    pub fn spawn_spark(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Result<Entity, BundleError> {
        self.spawn_particle_effect(
            commands,
            meshes,
            materials,
            position,
            ParticleEffectType::Spark,
            0.5,
        )
    }

    /// Spawn generic particle effect
    pub fn spawn_particle_effect(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        effect_type: ParticleEffectType,
        lifetime: f32,
    ) -> Result<Entity, BundleError> {
        let (mesh, material_color, radius) = match effect_type {
            ParticleEffectType::Explosion => (
                meshes.add(Sphere::new(0.5)),
                Color::srgb(1.0, 0.8, 0.0),
                0.5,
            ),
            ParticleEffectType::Spark => (
                meshes.add(Sphere::new(0.05)),
                Color::srgb(1.0, 1.0, 0.8),
                0.05,
            ),
            _ => (
                meshes.add(Sphere::new(0.1)),
                Color::srgb(0.5, 0.5, 0.5),
                0.1,
            ),
        };

        let material_handle = materials.add(StandardMaterial {
            base_color: material_color,
            emissive: LinearRgba::rgb(1.0, 0.5, 0.0),
            ..default()
        });

        // Create minimal physics for particle if needed
        let bundle = GenericBundleFactory::physics_object(
            position,
            crate::factories::generic_bundle::ColliderShape::Sphere(radius),
            0.1, // Very light particle
            self.config.physics.static_group,
            true, // Dynamic particle
            &self.config,
        )?;

        let effect_entity = commands
            .spawn((
                bundle,
                Mesh3d(mesh),
                MeshMaterial3d(material_handle),
                ParticleEffect {
                    effect_type,
                    lifetime,
                    age: 0.0,
                },
                VisibilityRange::abrupt(0.0, self.config.performance.vehicle_visibility_distance),
                Name::new(format!("Effect_{effect_type:?}")),
            ))
            .id();

        Ok(effect_entity)
    }

    /// Spawn multiple effects in batch
    pub fn spawn_effect_batch(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        positions: Vec<Vec3>,
        effect_type: ParticleEffectType,
        lifetime: f32,
    ) -> Result<Vec<Entity>, BundleError> {
        let mut entities = Vec::new();

        for position in positions {
            let entity = self.spawn_particle_effect(
                commands,
                meshes,
                materials,
                position,
                effect_type,
                lifetime,
            )?;
            entities.push(entity);
        }

        Ok(entities)
    }
}

/// Simple particle effect component for tracking lifetime
#[derive(Component, Debug)]
pub struct ParticleEffect {
    pub effect_type: ParticleEffectType,
    pub lifetime: f32,
    pub age: f32,
}

impl Default for EffectFactory {
    fn default() -> Self {
        Self::new()
    }
}
