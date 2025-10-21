use crate::components::water::{Yacht, YachtState};
use crate::util::transform_utils::horizontal_forward;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct YachtWakeTrail {
    last_spawn_time: f32,
    spawn_interval: f32,
}

impl Default for YachtWakeTrail {
    fn default() -> Self {
        Self {
            last_spawn_time: 0.0,
            spawn_interval: 0.1,
        }
    }
}

#[derive(Component)]
pub struct WakeTrailPoint {
    lifetime: f32,
    max_lifetime: f32,
}

#[derive(Component)]
pub struct BowSplash;

#[derive(Component)]
pub struct PropWash;

#[derive(Resource)]
pub struct YachtEffects {
    pub bow_splash: Handle<EffectAsset>,
    pub prop_wash: Handle<EffectAsset>,
}

pub fn setup_yacht_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let bow_splash = create_bow_splash_effect(&mut effects);
    let prop_wash = create_prop_wash_effect(&mut effects);

    commands.insert_resource(YachtEffects {
        bow_splash,
        prop_wash,
    });
}

fn create_bow_splash_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, 0.8));
    color_gradient.add_key(0.3, Vec4::new(0.9, 0.95, 1.0, 0.6));
    color_gradient.add_key(1.0, Vec4::new(0.7, 0.85, 0.95, 0.0));

    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(3.0),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(8.0),
    };

    let lifetime = module.lit(1.5);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(0.5));

    let accel = module.lit(Vec3::new(0.0, -5.0, 0.0));
    let update_accel = AccelModifier::new(accel);

    let effect = EffectAsset::new(2048, SpawnerSettings::rate(300.0.into()), module)
        .with_name("BowSplash")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_size)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec3::splat(0.5));
                gradient.add_key(0.5, Vec3::splat(1.2));
                gradient.add_key(1.0, Vec3::splat(0.3));
                gradient
            },
            screen_space_size: false,
        });

    effects.add(effect)
}

fn create_prop_wash_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.9, 0.95, 1.0, 0.4));
    color_gradient.add_key(0.5, Vec4::new(0.8, 0.9, 0.95, 0.3));
    color_gradient.add_key(1.0, Vec4::new(0.7, 0.85, 0.9, 0.0));

    let mut module = Module::default();

    let init_pos = SetPositionCircleModifier {
        center: module.lit(Vec3::ZERO),
        axis: module.lit(Vec3::Y),
        radius: module.lit(4.0),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, module.lit(Vec3::Z * 5.0));

    let lifetime = module.lit(3.0);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(0.8));

    let accel = module.lit(Vec3::new(0.0, -1.0, 0.0));
    let update_accel = AccelModifier::new(accel);

    let effect = EffectAsset::new(4096, SpawnerSettings::rate(200.0.into()), module)
        .with_name("PropWash")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_size)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec3::splat(0.8));
                gradient.add_key(1.0, Vec3::splat(2.5));
                gradient
            },
            screen_space_size: false,
        });

    effects.add(effect)
}

pub fn spawn_yacht_wake_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut yacht_query: Query<(&Transform, &Velocity, &mut YachtWakeTrail), With<Yacht>>,
) {
    for (transform, velocity, mut wake_trail) in yacht_query.iter_mut() {
        let speed = velocity.linvel.length();

        if speed < 2.0 {
            continue;
        }

        let elapsed = time.elapsed_secs();
        if elapsed - wake_trail.last_spawn_time < wake_trail.spawn_interval {
            continue;
        }

        wake_trail.last_spawn_time = elapsed;

        let forward = horizontal_forward(transform);
        let right = forward.cross(Vec3::Y).normalize_or_zero();

        let stern_offset = -forward * 25.0;
        let base_pos = transform.translation + stern_offset;

        let speed_factor = (speed / 20.0).clamp(0.0, 1.0);
        let width = 2.0 + speed_factor * 3.0;

        for side in [-1.0, 1.0] {
            let spawn_pos = base_pos + right * side * width;

            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.9, 0.95, 1.0, 0.6),
                    emissive: LinearRgba::new(0.3, 0.35, 0.4, 1.0),
                    alpha_mode: bevy::prelude::AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(spawn_pos),
                WakeTrailPoint {
                    lifetime: 0.0,
                    max_lifetime: 8.0,
                },
            ));
        }
    }
}

pub fn update_wake_trail_points(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut WakeTrailPoint)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut point) in query.iter_mut() {
        point.lifetime += dt;

        if point.lifetime >= point.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let life_factor = point.lifetime / point.max_lifetime;
        let scale = 1.0 - life_factor * 0.7;
        transform.scale = Vec3::splat(scale * (0.5 + life_factor * 2.0));
    }
}

pub fn spawn_bow_splash(
    mut commands: Commands,
    yacht_query: Query<(&Transform, &Velocity), With<Yacht>>,
    splash_query: Query<Entity, With<BowSplash>>,
    yacht_effects: Option<Res<YachtEffects>>,
) {
    let Some(effects) = yacht_effects else {
        return;
    };

    for (transform, velocity) in yacht_query.iter() {
        let forward = horizontal_forward(transform);
        let forward_speed = velocity.linvel.dot(forward);
        let speed = forward_speed.abs();

        if speed < 6.0 {
            for splash_entity in splash_query.iter() {
                commands.entity(splash_entity).despawn();
            }
            continue;
        }

        let has_splash = !splash_query.is_empty();

        if !has_splash {
            let bow_offset = forward * 29.0 + Vec3::Y * 0.5;
            let splash_pos = transform.translation + bow_offset;

            commands.spawn((
                ParticleEffect::new(effects.bow_splash.clone()),
                Transform::from_translation(splash_pos),
                BowSplash,
            ));
        } else {
            for splash_entity in splash_query.iter() {
                let bow_offset = forward * 29.0 + Vec3::Y * 0.5;
                let splash_pos = transform.translation + bow_offset;

                if let Ok(mut entity_commands) = commands.get_entity(splash_entity) {
                    entity_commands.insert(Transform::from_translation(splash_pos));
                }
            }
        }
    }
}

pub fn spawn_prop_wash(
    mut commands: Commands,
    yacht_query: Query<(&Transform, &YachtState, Entity), With<Yacht>>,
    wash_query: Query<Entity, With<PropWash>>,
    yacht_effects: Option<Res<YachtEffects>>,
) {
    let Some(effects) = yacht_effects else {
        return;
    };

    for (transform, yacht_state, yacht_entity) in yacht_query.iter() {
        let throttle = yacht_state.throttle.abs();

        if throttle < 0.1 {
            for wash_entity in wash_query.iter() {
                commands.entity(wash_entity).despawn();
            }
            continue;
        }

        let has_wash = !wash_query.is_empty();

        if !has_wash {
            let forward = horizontal_forward(transform);
            let prop_offset = -forward * 29.0 + Vec3::Y * -1.0;

            commands.entity(yacht_entity).with_children(|parent| {
                parent.spawn((
                    ParticleEffect::new(effects.prop_wash.clone()),
                    Transform::from_translation(prop_offset),
                    PropWash,
                ));
            });
        }
    }
}

pub fn initialize_yacht_wake_trail(
    mut commands: Commands,
    yacht_query: Query<Entity, (With<Yacht>, Without<YachtWakeTrail>)>,
) {
    for yacht_entity in yacht_query.iter() {
        commands
            .entity(yacht_entity)
            .insert(YachtWakeTrail::default());
    }
}
