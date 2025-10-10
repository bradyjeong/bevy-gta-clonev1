use crate::components::{ActiveEntity, Helicopter, RotorWash};
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::Velocity;

type ActiveHelicopterQuery<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<Helicopter>,
        With<ActiveEntity>,
        Without<ParticleEffect>,
    ),
>;

type ParticleTransformQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut EffectSpawner),
    (With<RotorWash>, Without<Helicopter>),
>;

type HelicopterStateQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Velocity), (With<Helicopter>, With<ActiveEntity>)>;

fn create_rotor_wash_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
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

pub fn spawn_rotor_wash_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    helicopter_query: ActiveHelicopterQuery,
) {
    for _entity in helicopter_query.iter() {
        let effect_handle = create_rotor_wash_effect(&mut effects);

        commands.spawn((
            Name::new("rotor_wash_particles"),
            ParticleEffect::new(effect_handle),
            Transform::from_xyz(0.0, 0.0, 0.0),
            RotorWash,
        ));
    }
}

pub fn update_rotor_wash_position_and_intensity(
    helicopter_query: HelicopterStateQuery,
    mut particle_query: ParticleTransformQuery,
) {
    for (helicopter_transform, velocity) in helicopter_query.iter() {
        let altitude = helicopter_transform.translation.y;
        let vertical_velocity = velocity.linvel.y.abs();

        for (mut particle_transform, mut spawner) in particle_query.iter_mut() {
            particle_transform.translation.x = helicopter_transform.translation.x;
            particle_transform.translation.z = helicopter_transform.translation.z;
            particle_transform.translation.y = 0.1;

            if altitude < 10.0 && vertical_velocity > 0.5 {
                let altitude_factor = (1.0 - (altitude / 10.0)).max(0.0);
                let velocity_factor = (vertical_velocity / 10.0).min(1.0);
                let intensity = altitude_factor * 0.7 + velocity_factor * 0.3;

                spawner.active = true;
                spawner.settings.set_count((intensity * 120.0).into());
            } else {
                spawner.active = false;
            }
        }
    }
}
