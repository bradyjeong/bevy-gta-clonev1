use crate::components::control_state::ControlState;
use crate::components::{ActiveEntity, PropellerHub, Yacht};
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_hanabi::prelude::*;

/// Resource that caches the boat wake effect handle.
#[derive(Resource)]
pub struct BoatWakeEffect {
    pub handle: Handle<EffectAsset>,
}

/// Component linking a boat wake particle effect to its yacht.
#[derive(Component, Debug, Clone, Copy)]
pub struct BoatWakeOf(pub Entity);

/// Creates the boat wake particle effect asset.
pub fn create_boat_wake_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut color_gradient = bevy_hanabi::Gradient::new();
    // Foamy white water
    color_gradient.add_key(0.0, Vec4::new(0.9, 0.95, 1.0, 0.6));
    color_gradient.add_key(0.2, Vec4::new(0.9, 0.95, 1.0, 0.4));
    color_gradient.add_key(1.0, Vec4::new(0.9, 0.95, 1.0, 0.0));

    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.5));
    size_gradient.add_key(1.0, Vec3::splat(2.5));

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let base_lifetime = writer.lit(2.0);
    let lifetime_jitter = writer.rand(ScalarType::Float) * writer.lit(1.0);
    let lifetime = (base_lifetime + lifetime_jitter).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Spawn in a small radius around the source (propeller)
    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Initial velocity: Backwards and slightly up
    // We assume the effect is spawned at the prop with local rotation matching the boat?
    // Actually, easier to update position/rotation in the system.
    // Let's assume -Z is backward (local). 
    // BUT, we will handle position update in world space. 
    // If we update transform to match boat rotation, then -Z is backwards.
    
    // Velocity spread
    let speed = writer.lit(5.0).expr();
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed,
    };

    // Drag to simulate water resistance
    let drag = writer.lit(2.0).expr();
    let update_drag = LinearDragModifier::new(drag);

    let module = writer.finish();
    
    // High spawn rate for continuous wake
    let spawner = SpawnerSettings::rate(100.0.into());

    effects.add(
        EffectAsset::new(2000, spawner, module)
            .with_name("boat_wake")
            .with_simulation_space(SimulationSpace::Global) // Particles trail behind
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .update(update_drag)
            .render(ColorOverLifetimeModifier::new(color_gradient))
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
                screen_space_size: false,
            }),
    )
}

/// Spawns boat wake particles for newly added yachts.
pub fn spawn_boat_wake_particles(
    mut commands: Commands,
    yacht_query: Query<Entity, Added<Yacht>>,
    wake_effect: Res<BoatWakeEffect>,
) {
    for yacht_entity in yacht_query.iter() {
        commands.spawn((
            Name::new("boat_wake_particles"),
            ParticleEffect::new(wake_effect.handle.clone()),
            {
                let mut spawner = EffectSpawner::new(&SpawnerSettings::rate(100.0.into()));
                spawner.active = false;
                spawner
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            BoatWakeOf(yacht_entity),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 500.0..700.0,
                use_aabb: false,
            },
        ));
    }
}

/// Updates boat wake position and intensity based on throttle and propeller position.
#[allow(clippy::type_complexity)]
pub fn update_boat_wake_intensity(
    mut particle_query: Query<(&mut EffectSpawner, &mut Transform, &BoatWakeOf)>,
    yacht_query: Query<(&GlobalTransform, &ControlState, &Children), (With<Yacht>, With<ActiveEntity>)>,
    prop_query: Query<&GlobalTransform, With<PropellerHub>>,
) {
    for (mut spawner, mut particle_transform, wake_of) in particle_query.iter_mut() {
        if let Ok((yacht_transform, controls, children)) = yacht_query.get(wake_of.0) {
            // Find propeller position
            let mut prop_pos = yacht_transform.translation(); // Fallback to yacht center
            let mut found_prop = false;

            for child in children.iter() {
                if let Ok(prop_transform) = prop_query.get(child) {
                    prop_pos = prop_transform.translation();
                    // Offset slightly behind prop to avoid clipping
                    prop_pos -= yacht_transform.forward() * 1.0; 
                    found_prop = true;
                    break;
                }
            }

            if !found_prop {
                // If no prop hub found (maybe looking at wrong hierarchy?), approximate stern
                // Typical yacht stern is -Z local.
                prop_pos -= yacht_transform.forward() * 4.0;
            }
            
            // Position particles at prop/stern
            particle_transform.translation = prop_pos;
            // Align rotation with boat so particles shoot backward relative to boat?
            // If SimulationSpace::World is used, initial velocity is in local space? 
            // Wait, SimulationSpace::World means simulation runs in world space (particles stay behind).
            // Emitter transform determines spawn location/orientation.
            // So if we rotate emitter to match boat, particles spawn with velocity relative to emitter axis?
            // Let's assume SetVelocitySphere sends them in all directions, but we want backward jet.
            // Actually SetVelocitySphere is omnidirectional. 
            // To mimic prop wash, we want backward cone.
            // But simple sphere with World simulation creates a trail as the boat moves forward.
            // The particles spawn and stay (mostly) where they were, creating a trail.
            particle_transform.rotation = yacht_transform.compute_transform().rotation;

            // Intensity based on throttle
            let throttle_effort = (controls.throttle - controls.brake).abs();
            
            spawner.active = throttle_effort > 0.1;

        } else {
            spawner.active = false;
        }
    }
}

pub fn cleanup_boat_wake_on_despawn(
    mut commands: Commands,
    mut removed_yachts: RemovedComponents<Yacht>,
    particle_query: Query<(Entity, &BoatWakeOf)>,
) {
    for removed_entity in removed_yachts.read() {
        for (particle_entity, wake_of) in particle_query.iter() {
            if wake_of.0 == removed_entity {
                commands.entity(particle_entity).despawn();
            }
        }
    }
}
