use crate::components::player::ActiveEntity;
use crate::components::water::{Yacht, YachtState};
use crate::util::transform_utils::horizontal_forward;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct BowSplash;

#[derive(Component)]
pub struct PropWash;

#[derive(Component)]
pub struct WakeFoam;

#[derive(Resource)]
pub struct YachtEffects {
    pub bow_splash: Handle<EffectAsset>,
    pub prop_wash: Handle<EffectAsset>,
    pub wake_foam: Handle<EffectAsset>,
}

pub fn setup_yacht_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let bow_splash = create_bow_splash_effect(&mut effects);
    let prop_wash = create_prop_wash_effect(&mut effects);
    let wake_foam = create_wake_foam_effect(&mut effects);

    commands.insert_resource(YachtEffects {
        bow_splash,
        prop_wash,
        wake_foam,
    });
}

fn create_bow_splash_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut module = Module::default();

    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, 0.9));
    color_gradient.add_key(0.25, Vec4::new(0.95, 0.98, 1.0, 0.7));
    color_gradient.add_key(1.0, Vec4::new(0.8, 0.9, 0.98, 0.0));

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(2.0),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(12.0),
    };

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(1.5));
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(0.4));

    let update_accel = AccelModifier::new(module.lit(Vec3::new(0.0, -9.0, 0.0)));
    let update_drag = LinearDragModifier::new(module.lit(2.0));

    let effect = EffectAsset::new(4096, SpawnerSettings::rate(500.0.into()), module)
        .with_name("BowSplash")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_size)
        .update(update_accel)
        .update(update_drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec3::splat(0.4));
                gradient.add_key(0.3, Vec3::splat(1.5));
                gradient.add_key(1.0, Vec3::splat(0.2));
                gradient
            },
            screen_space_size: false,
        });

    effects.add(effect)
}

fn create_prop_wash_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut module = Module::default();

    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.95, 0.98, 1.0, 0.5));
    color_gradient.add_key(0.5, Vec4::new(0.86, 0.92, 0.96, 0.35));
    color_gradient.add_key(1.0, Vec4::new(0.75, 0.88, 0.92, 0.0));

    let init_pos = SetPositionCircleModifier {
        center: module.lit(Vec3::ZERO),
        axis: module.lit(Vec3::Y),
        radius: module.lit(1.8),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, module.lit(Vec3::Z * 7.0));

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(2.8));
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(0.6));

    let update_buoyancy = AccelModifier::new(module.lit(Vec3::new(0.0, 0.4, 0.0)));
    let update_drag = LinearDragModifier::new(module.lit(1.2));

    let effect = EffectAsset::new(4096, SpawnerSettings::rate(400.0.into()), module)
        .with_name("PropWash")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_size)
        .update(update_buoyancy)
        .update(update_drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec3::splat(0.6));
                gradient.add_key(0.6, Vec3::splat(2.0));
                gradient.add_key(1.0, Vec3::splat(2.6));
                gradient
            },
            screen_space_size: false,
        });

    effects.add(effect)
}

fn create_wake_foam_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut module = Module::default();

    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.95, 0.98, 1.0, 0.7));
    color_gradient.add_key(0.5, Vec4::new(0.9, 0.95, 0.98, 0.5));
    color_gradient.add_key(1.0, Vec4::new(0.85, 0.9, 0.95, 0.0));

    let init_pos = SetPositionCircleModifier {
        center: module.lit(Vec3::ZERO),
        axis: module.lit(Vec3::Y),
        radius: module.lit(3.0),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, module.lit(Vec3::Z * -4.0));
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(6.0));
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(0.8));

    let update_accel = AccelModifier::new(module.lit(Vec3::new(0.0, -0.5, 0.0)));
    let update_drag = LinearDragModifier::new(module.lit(1.2));

    let effect = EffectAsset::new(8192, SpawnerSettings::rate(300.0.into()), module)
        .with_name("WakeFoam")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_size)
        .update(update_accel)
        .update(update_drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec3::splat(0.8));
                gradient.add_key(0.5, Vec3::splat(2.5));
                gradient.add_key(1.0, Vec3::splat(1.8));
                gradient
            },
            screen_space_size: false,
        });

    effects.add(effect)
}

pub fn spawn_or_update_wake_foam(
    mut commands: Commands,
    yacht_q: Query<(&Velocity, Entity, Option<&ActiveEntity>), With<Yacht>>,
    foam_q: Query<Entity, With<WakeFoam>>,
    yacht_effects: Option<Res<YachtEffects>>,
) {
    let Some(effects) = yacht_effects else {
        return;
    };

    for (vel, yacht_e, is_active) in yacht_q.iter() {
        // Only spawn particles for the active yacht to save CPU/GPU
        if is_active.is_none() {
            for e in foam_q.iter() {
                commands.entity(e).despawn();
            }
            continue;
        }
        let speed = vel.linvel.length();

        if speed < 2.0 {
            for e in foam_q.iter() {
                commands.entity(e).despawn();
            }
            continue;
        }

        let width = 2.0 + ((speed / 20.0).clamp(0.0, 1.0) * 4.0);

        if foam_q.is_empty() {
            commands.entity(yacht_e).with_children(|parent| {
                parent.spawn((
                    ParticleEffect::new(effects.wake_foam.clone()),
                    Transform {
                        translation: Vec3::new(0.0, 0.05, -28.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(width, 1.0, 1.0),
                    },
                    WakeFoam,
                    // Bug #43 fix: Match parent vehicle visibility range (1000m with ±10% variance)
                    VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 900.0..1100.0,
                        use_aabb: false,
                    },
                ));
            });
        }
    }
}

pub fn spawn_bow_splash(
    mut commands: Commands,
    yacht_query: Query<(&Transform, &Velocity, Entity, Option<&ActiveEntity>), With<Yacht>>,
    splash_query: Query<Entity, With<BowSplash>>,
    yacht_effects: Option<Res<YachtEffects>>,
) {
    let Some(effects) = yacht_effects else {
        return;
    };

    for (xf, vel, yacht_e, is_active) in yacht_query.iter() {
        // Only spawn particles for the active yacht to save CPU/GPU
        if is_active.is_none() {
            for e in splash_query.iter() {
                commands.entity(e).despawn();
            }
            continue;
        }
        let fwd = horizontal_forward(xf);
        let fwd_speed = vel.linvel.dot(fwd).max(0.0);

        if fwd_speed < 6.0 {
            for e in splash_query.iter() {
                commands.entity(e).despawn();
            }
            continue;
        }

        if splash_query.is_empty() {
            commands.entity(yacht_e).with_children(|parent| {
                parent.spawn((
                    ParticleEffect::new(effects.bow_splash.clone()),
                    Transform::from_translation(Vec3::new(0.0, 0.6, 29.0)),
                    BowSplash,
                    // Bug #43 fix: Match parent vehicle visibility range (1000m with ±10% variance)
                    VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 900.0..1100.0,
                        use_aabb: false,
                    },
                ));
            });
        }
    }
}

pub fn spawn_prop_wash(
    mut commands: Commands,
    yacht_query: Query<(&YachtState, &Velocity, Entity, Option<&ActiveEntity>), With<Yacht>>,
    wash_query: Query<Entity, With<PropWash>>,
    yacht_effects: Option<Res<YachtEffects>>,
) {
    let Some(effects) = yacht_effects else {
        return;
    };

    for (state, vel, yacht_e, is_active) in yacht_query.iter() {
        // Only spawn particles for the active yacht to save CPU/GPU
        if is_active.is_none() {
            for e in wash_query.iter() {
                commands.entity(e).despawn();
            }
            continue;
        }
        let throttle = state.throttle.abs();
        let speed = vel.linvel.length();

        let width = (1.8 + (speed / 20.0).clamp(0.0, 1.0) * 2.0).max(1.8);

        if throttle < 0.1 {
            for e in wash_query.iter() {
                commands.entity(e).despawn();
            }
            continue;
        }

        if wash_query.is_empty() {
            commands.entity(yacht_e).with_children(|parent| {
                parent.spawn((
                    ParticleEffect::new(effects.prop_wash.clone()),
                    Transform {
                        translation: Vec3::new(0.0, -0.4, -29.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(width, 1.0, 1.0),
                    },
                    PropWash,
                    // Bug #43 fix: Match parent vehicle visibility range (1000m with ±10% variance)
                    VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: 900.0..1100.0,
                        use_aabb: false,
                    },
                ));
            });
        }
    }
}

pub fn cleanup_yacht_particles_on_despawn(
    mut commands: Commands,
    mut removed_yachts: RemovedComponents<Yacht>,
    children_query: Query<&Children>,
) {
    for yacht_entity in removed_yachts.read() {
        if let Ok(children) = children_query.get(yacht_entity) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
    }
}
