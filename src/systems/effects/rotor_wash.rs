use crate::components::{
    ActiveEntity, ControlState, Helicopter, HelicopterRuntime, RotorWash, SimpleHelicopterSpecs,
    SimpleHelicopterSpecsHandle,
};
use crate::constants::WorldEnvConfig;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::Velocity;

/// Resource that caches the rotor wash effect handle.
/// Created once at startup and reused across all helicopters for optimal performance.
#[derive(Resource)]
pub struct RotorWashEffect {
    pub handle: Handle<EffectAsset>,
}

/// Component linking a rotor wash particle effect to its helicopter.
/// Used to maintain O(N) updates and proper cleanup.
#[derive(Component, Debug, Clone, Copy)]
pub struct RotorWashOf(pub Entity);

type ParticleTransformQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static RotorWashOf,
        &'static mut Transform,
        &'static mut EffectSpawner,
    ),
    (With<RotorWash>, Without<Helicopter>),
>;

type HelicopterStateQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static Velocity,
        &'static HelicopterRuntime,
        &'static ControlState,
        &'static SimpleHelicopterSpecsHandle,
        Option<&'static ActiveEntity>,
    ),
    With<Helicopter>,
>;

/// Creates the rotor wash particle effect asset.
/// Called once at startup to initialize the cached effect handle.
pub fn create_rotor_wash_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.91, 0.84, 0.68, 0.0));
    color_gradient.add_key(0.1, Vec4::new(0.89, 0.82, 0.66, 0.7));
    color_gradient.add_key(0.7, Vec4::new(0.87, 0.79, 0.63, 0.5));
    color_gradient.add_key(1.0, Vec4::new(0.85, 0.76, 0.6, 0.0));

    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.12));
    size_gradient.add_key(0.4, Vec3::splat(0.4));
    size_gradient.add_key(1.0, Vec3::splat(0.15));

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(1.8).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let min_radius = writer.lit(2.5);
    let radius_range = writer.lit(3.0);
    let random_radius = min_radius + writer.rand(ScalarType::Float) * radius_range;

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: random_radius.expr(),
        dimension: ShapeDimension::Surface,
    };

    let base_speed = writer.lit(3.5);
    let random_factor = writer.rand(ScalarType::Float) * writer.lit(2.0);
    let speed = (base_speed + random_factor).expr();

    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        speed,
    };

    let accel = writer.lit(Vec3::new(0.0, 0.3, 0.0)).expr();
    let update_accel = AccelModifier::new(accel);

    let drag = writer.lit(2.0).expr();
    let update_drag = LinearDragModifier::new(drag);

    let module = writer.finish();
    let spawner = SpawnerSettings::rate(50.0.into());

    effects.add(
        EffectAsset::new(8192, spawner, module)
            .with_name("rotor_wash")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .update(update_accel)
            .update(update_drag)
            .render(ColorOverLifetimeModifier::new(color_gradient))
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
                screen_space_size: false,
            }),
    )
}

/// Spawns rotor wash particles for newly added helicopters.
/// Uses Added<Helicopter> filter to only spawn once per helicopter.
/// Clones the cached effect handle instead of creating a new effect asset.
pub fn spawn_rotor_wash_particles(
    mut commands: Commands,
    helicopter_query: Query<Entity, Added<Helicopter>>,
    rotor_wash_effect: Res<RotorWashEffect>,
) {
    for heli_entity in helicopter_query.iter() {
        #[cfg(feature = "debug-ui")]
        info!(
            "Spawning rotor wash particles for helicopter entity: {:?}",
            heli_entity
        );

        // Spawn as separate entity (not child) to use world coordinates for ground contact
        commands.spawn((
            Name::new("rotor_wash_particles"),
            ParticleEffect::new(rotor_wash_effect.handle.clone()),
            {
                let mut spawner = EffectSpawner::new(&SpawnerSettings::rate(50.0.into()));
                spawner.active = false; // Start inactive, will activate only for ActiveEntity helicopters
                spawner
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            RotorWash,
            RotorWashOf(heli_entity),
            // Bug #43 fix: Match parent vehicle visibility range (1000m with ±10% variance)
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 900.0..1100.0,
                use_aabb: false,
            },
        ));
    }
}

pub fn update_rotor_wash_position_and_intensity(
    mut commands: Commands,
    helicopter_query: HelicopterStateQuery,
    mut particle_query: ParticleTransformQuery,
    env: Res<WorldEnvConfig>,
    heli_specs_assets: Res<Assets<SimpleHelicopterSpecs>>,
) {
    for (rotor_wash_entity, rotor_wash_of, mut particle_transform, mut spawner) in
        particle_query.iter_mut()
    {
        let heli_entity = rotor_wash_of.0;

        if let Ok((helicopter_transform, _velocity, runtime, control_state, specs_handle, is_active)) =
            helicopter_query.get(heli_entity)
        {
            // Only update particles for the active helicopter to save CPU/GPU
            if is_active.is_none() {
                spawner.active = false;
                continue;
            }

            let Some(specs) = heli_specs_assets.get(&specs_handle.0) else {
                spawner.active = false;
                continue;
            };

            let heli_pos = helicopter_transform.translation;
            let ground_height = env.land_elevation;
            let altitude = (heli_pos.y - ground_height).max(0.0);

            // World coordinates - particles stay on ground
            particle_transform.translation.x = heli_pos.x;
            particle_transform.translation.z = heli_pos.z;
            particle_transform.translation.y = ground_height + 0.1;

            // Calculate RPM effectiveness (matches movement system)
            let rpm_eff = if runtime.rpm < specs.min_rpm_for_lift {
                0.0
            } else {
                ((runtime.rpm - specs.min_rpm_for_lift) / (1.0 - specs.min_rpm_for_lift))
                    .clamp(0.0, 1.0)
                    .powf(specs.rpm_to_lift_exp)
            };

            // Calculate lift scalar from collective input
            let collective_gain = 0.6;
            let lift_scalar: f32 = 1.0 + collective_gain * control_state.vertical;

            // Calculate intensity using oracle's formula
            let base_intensity = specs.rotor_wash_scale * rpm_eff * lift_scalar.max(0.0);

            // Apply altitude gating (only show near ground)
            let altitude_gate: f32 = if altitude < 10.0 {
                (1.0 - (altitude / 10.0)).max(0.0)
            } else {
                0.0
            };

            let final_intensity = base_intensity * altitude_gate;

            // Apply intensity to particle system - only below 10m altitude
            spawner.active = altitude < 10.0 && final_intensity > 0.05;
        } else {
            commands.entity(rotor_wash_entity).despawn();
        }
    }
}

pub fn cleanup_rotor_wash_on_helicopter_despawn(
    mut commands: Commands,
    mut removed_helicopters: RemovedComponents<Helicopter>,
    particle_query: Query<(Entity, &RotorWashOf)>,
) {
    // Clean up rotor wash particles linked to despawned helicopters
    for removed_heli_entity in removed_helicopters.read() {
        for (particle_entity, rotor_wash_of) in particle_query.iter() {
            if rotor_wash_of.0 == removed_heli_entity {
                commands.entity(particle_entity).despawn();
            }
        }
    }
}
