use crate::components::{Helicopter, RotorWash};
use crate::constants::WorldEnvConfig;
use bevy::prelude::*;
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

type NewHelicopterQuery<'w, 's> = Query<'w, 's, Entity, Added<Helicopter>>;

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

type HelicopterStateQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Velocity), With<Helicopter>>;

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
    let spawner = SpawnerSettings::burst(120.0.into(), 0.016.into());

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
    helicopter_query: NewHelicopterQuery,
    rotor_wash_effect: Res<RotorWashEffect>,
) {
    for entity in helicopter_query.iter() {
        commands.spawn((
            Name::new("rotor_wash_particles"),
            ParticleEffect::new(rotor_wash_effect.handle.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            RotorWash,
            RotorWashOf(entity),
        ));
    }
}

pub fn update_rotor_wash_position_and_intensity(
    mut commands: Commands,
    helicopter_query: HelicopterStateQuery,
    mut particle_query: ParticleTransformQuery,
    env: Res<WorldEnvConfig>,
) {
    for (rotor_wash_entity, rotor_wash_of, mut particle_transform, mut spawner) in
        particle_query.iter_mut()
    {
        let heli_entity = rotor_wash_of.0;

        if let Ok((helicopter_transform, velocity)) = helicopter_query.get(heli_entity) {
            let heli_pos = helicopter_transform.translation;
            let vertical_velocity = velocity.linvel.y.abs();

            let ground_height = env.land_elevation;
            let altitude = (heli_pos.y - ground_height).max(0.0);

            particle_transform.translation.x = heli_pos.x;
            particle_transform.translation.z = heli_pos.z;
            particle_transform.translation.y = ground_height + 0.1;

            if altitude < 10.0 && vertical_velocity > 0.5 {
                let altitude_factor = (1.0 - (altitude / 10.0)).max(0.0);
                let velocity_factor = (vertical_velocity / 10.0).min(1.0);
                let intensity = altitude_factor * 0.7 + velocity_factor * 0.3;

                spawner.active = true;
                spawner.spawn_count = (intensity * 120.0) as u32;
            } else {
                spawner.active = false;
            }
        } else {
            commands.entity(rotor_wash_entity).despawn();
        }
    }
}

pub fn cleanup_rotor_wash_on_helicopter_despawn(
    mut commands: Commands,
    mut removed_helicopters: RemovedComponents<Helicopter>,
    rotor_wash_query: Query<(Entity, &RotorWashOf), With<RotorWash>>,
) {
    for removed_heli_entity in removed_helicopters.read() {
        for (rotor_wash_entity, rotor_wash_of) in rotor_wash_query.iter() {
            if rotor_wash_of.0 == removed_heli_entity {
                commands.entity(rotor_wash_entity).despawn();
            }
        }
    }
}
