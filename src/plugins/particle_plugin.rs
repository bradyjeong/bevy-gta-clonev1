use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_systems(Startup, setup_particle_effects)
            .add_systems(Update, update_rotor_wash);
    }
}

#[derive(Resource)]
pub struct ParticleEffects {
    pub rotor_wash: Handle<EffectAsset>,
}

#[derive(Component)]
pub struct RotorWash;

fn setup_particle_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let rotor_wash = create_rotor_wash_effect(&mut effects);

    commands.insert_resource(ParticleEffects { rotor_wash });
}

fn create_rotor_wash_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.8, 0.8, 0.9, 0.3));
    gradient.add_key(0.5, Vec4::new(0.7, 0.7, 0.8, 0.2));
    gradient.add_key(1.0, Vec4::new(0.6, 0.6, 0.7, 0.0));

    let mut module = Module::default();

    let init_pos = SetPositionCircleModifier {
        center: module.lit(Vec3::ZERO),
        axis: module.lit(Vec3::Y),
        radius: module.lit(2.0),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        center: module.lit(Vec3::ZERO),
        axis: module.lit(Vec3::Y),
        speed: module.lit(3.0),
    };

    let lifetime = module.lit(2.0);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(0.3));

    let accel = module.lit(Vec3::new(0.0, -2.0, 0.0));
    let update_accel = AccelModifier::new(accel);

    let effect = EffectAsset::new(4096, SpawnerSettings::rate(100.0.into()), module)
        .with_name("RotorWash")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_size)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: {
                let mut gradient = Gradient::new();
                gradient.add_key(0.0, Vec3::splat(0.3));
                gradient.add_key(1.0, Vec3::splat(1.5));
                gradient
            },
            screen_space_size: false,
        });

    effects.add(effect)
}

fn update_rotor_wash(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<crate::components::vehicles::Helicopter>>,
    wash_query: Query<Entity, With<RotorWash>>,
    particle_effects: Option<Res<ParticleEffects>>,
) {
    let Some(effects) = particle_effects else {
        return;
    };

    for (entity, _transform) in query.iter() {
        let has_wash = wash_query.iter().any(|e| commands.entity(e).id() == entity);

        if !has_wash {
            let wash_offset = Vec3::new(0.0, -0.5, 0.0);

            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    ParticleEffect::new(effects.rotor_wash.clone()),
                    Transform::from_translation(wash_offset),
                    RotorWash,
                ));
            });
        }
    }
}
