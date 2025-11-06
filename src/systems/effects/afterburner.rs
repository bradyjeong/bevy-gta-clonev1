use crate::components::{ActiveEntity, AircraftFlight, ControlState, F16, SimpleF16SpecsHandle};
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_hanabi::prelude::*;
use bevy_rapier3d::prelude::Velocity;

/// Resource that caches the afterburner flame effect handle.
/// Created once at startup and reused across all F16s for optimal performance.
#[derive(Resource)]
pub struct AfterburnerFlameEffect {
    pub handle: Handle<EffectAsset>,
}

/// Component linking an afterburner particle effect to its F16.
/// Used to maintain O(N) updates and proper cleanup.
#[derive(Component, Debug, Clone, Copy)]
pub struct AfterburnerFlameOf(pub Entity);

/// Marker component for afterburner particle effects
#[derive(Component)]
pub struct AfterburnerFlame;

type ParticleQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static AfterburnerFlameOf,
        &'static mut Transform,
        &'static mut EffectSpawner,
    ),
    (With<AfterburnerFlame>, Without<F16>),
>;

type F16StateQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static Velocity,
        &'static AircraftFlight,
        &'static ControlState,
        &'static SimpleF16SpecsHandle,
        Option<&'static ActiveEntity>,
    ),
    With<F16>,
>;

type F16ExistsQuery<'w, 's> = Query<'w, 's, (), With<F16>>;

/// Creates the afterburner flame particle effect asset.
/// Called once at startup to initialize the cached effect handle.
pub fn create_afterburner_flame_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let writer = ExprWriter::new();

    // Initialize particle age to 0
    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Short lifetime for fast-moving flames (0.3-0.5 seconds)
    let base_lifetime = writer.lit(0.3);
    let lifetime_jitter = writer.rand(ScalarType::Float) * writer.lit(0.2);
    let lifetime = (base_lifetime + lifetime_jitter).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Spawn particles in a cone shape pointing backwards from engine
    let init_pos = SetPositionCone3dModifier {
        height: writer.lit(0.5).expr(),
        base_radius: writer.lit(0.1).expr(),
        top_radius: writer.lit(0.3).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Directional velocity - shoot particles backwards along -Z axis
    let base_vel = writer.lit(Vec3::new(0.0, 0.0, -25.0));
    let random_offset =
        (writer.rand(VectorType::VEC3F) * writer.lit(2.0) - writer.lit(1.0)) * writer.lit(3.0);
    let vel = (base_vel + random_offset).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);

    // Add drag to slow particles down quickly (creates flame taper effect)
    let drag = writer.lit(2.5).expr();
    let update_drag = LinearDragModifier::new(drag);

    // GTA V style color gradient: Bright core -> Orange -> Red -> Transparent
    // Reduced HDR values to prevent excessive bloom
    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(3.0, 3.0, 2.5, 1.0)); // Bright white-yellow core
    color_gradient.add_key(0.2, Vec4::new(3.0, 2.0, 0.8, 1.0)); // Bright orange-yellow
    color_gradient.add_key(0.5, Vec4::new(2.5, 1.2, 0.3, 1.0)); // Orange-red
    color_gradient.add_key(0.8, Vec4::new(2.0, 0.5, 0.0, 0.8)); // Deep red
    color_gradient.add_key(1.0, Vec4::new(1.0, 0.0, 0.0, 0.0)); // Fade to transparent

    // Size over lifetime - starts large, tapers to point
    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::new(0.6, 0.6, 1.5)); // Large, elongated
    size_gradient.add_key(0.3, Vec3::new(0.5, 0.5, 1.2));
    size_gradient.add_key(0.7, Vec3::new(0.3, 0.3, 0.8));
    size_gradient.add_key(1.0, Vec3::new(0.05, 0.05, 0.2)); // Taper to point

    // Orient particles along velocity for streak effect
    let orient = OrientModifier::new(OrientMode::AlongVelocity);

    let module = writer.finish();
    let spawner = SpawnerSettings::rate(2000.0.into());

    effects.add(
        EffectAsset::new(8192, spawner, module)
            .with_name("afterburner_flame")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .update(update_drag)
            .render(ColorOverLifetimeModifier::new(color_gradient))
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
                screen_space_size: false,
            })
            .render(orient),
    )
}

/// Spawns afterburner particle effects for newly created F16s.
/// Runs once per F16, attaches particle effect as a child entity.
pub fn spawn_afterburner_particles(
    mut commands: Commands,
    f16_query: Query<Entity, (With<F16>, Without<Children>)>,
    afterburner_effect: Res<AfterburnerFlameEffect>,
) {
    for f16_entity in f16_query.iter() {
        commands.spawn((
            Name::new("afterburner_particles"),
            ParticleEffect::new(afterburner_effect.handle.clone()),
            AfterburnerFlame,
            AfterburnerFlameOf(f16_entity),
            ChildOf(f16_entity),
            // Position behind F16 engine (matches old cone position)
            Transform::from_xyz(0.0, 0.0, 10.5),
            VisibilityRange::abrupt(0.0, 2000.0),
            InheritedVisibility::default(),
        ));
    }
}

/// Updates afterburner particle position and intensity based on F16 flight state.
/// Modulates spawn rate based on throttle and afterburner status.
pub fn update_afterburner_position_and_intensity(
    mut particle_query: ParticleQuery,
    f16_query: F16StateQuery,
    f16_exists: F16ExistsQuery,
    mut commands: Commands,
) {
    for (particle_entity, afterburner_of, mut particle_transform, mut spawner) in
        particle_query.iter_mut()
    {
        let f16_entity = afterburner_of.0;

        // Cleanup if F16 no longer exists
        if f16_exists.get(f16_entity).is_err() {
            commands.entity(particle_entity).despawn();
            continue;
        }

        // Get F16 state
        let Ok((_f16_transform, _velocity, flight, _control, _specs, active)) =
            f16_query.get(f16_entity)
        else {
            continue;
        };

        // Turn off particles immediately if not the active entity
        if active.is_none() {
            spawner.reset();
            continue;
        }

        // Calculate flame intensity based on throttle and afterburner
        let base_intensity = flight.throttle.clamp(0.0, 1.0) * 0.5;
        let afterburner_boost = if flight.afterburner_active { 1.0 } else { 0.0 };
        let flame_intensity = (base_intensity + afterburner_boost).clamp(0.0, 1.0);

        // Early exit if flames should be off - just reset spawner
        if flame_intensity < 0.1 {
            spawner.reset();
            continue;
        }

        // Modulate spawn rate based on intensity
        // Base: 2000 particles/sec, afterburner: up to 4000 particles/sec
        let spawn_rate = 2000.0 + (flame_intensity * 2000.0);
        *spawner = EffectSpawner::new(&SpawnerSettings::rate(spawn_rate.into()));

        // Update position to follow F16 (particles are already in local space)
        // Transform is already relative to F16 parent, just ensure proper rotation
        particle_transform.rotation = Quat::IDENTITY;
    }
}

/// Ensures all existing F16s have afterburner particle effects.
/// Runs at startup to attach effects to any F16s spawned before particle system initialized.
pub fn ensure_afterburner_for_existing_f16s(
    mut commands: Commands,
    f16_query: Query<Entity, With<F16>>,
    afterburner_query: Query<&AfterburnerFlameOf>,
    afterburner_effect: Res<AfterburnerFlameEffect>,
) {
    let f16s_with_afterburners: std::collections::HashSet<Entity> =
        afterburner_query.iter().map(|af| af.0).collect();

    for f16_entity in f16_query.iter() {
        if !f16s_with_afterburners.contains(&f16_entity) {
            commands.spawn((
                Name::new("afterburner_particles"),
                ParticleEffect::new(afterburner_effect.handle.clone()),
                AfterburnerFlame,
                AfterburnerFlameOf(f16_entity),
                ChildOf(f16_entity),
                Transform::from_xyz(0.0, 0.0, 10.5),
                VisibilityRange::abrupt(0.0, 2000.0),
                InheritedVisibility::default(),
            ));
        }
    }
}

/// Cleanup system: removes afterburner particles when F16 is despawned.
pub fn cleanup_afterburner_on_f16_despawn(
    mut commands: Commands,
    afterburner_query: Query<(Entity, &AfterburnerFlameOf)>,
    f16_query: F16ExistsQuery,
) {
    for (particle_entity, afterburner_of) in afterburner_query.iter() {
        if f16_query.get(afterburner_of.0).is_err() {
            commands.entity(particle_entity).despawn();
        }
    }
}

/// Cleanup system: removes orphaned afterburner particle entities.
pub fn cleanup_afterburner_particle_entities(
    mut commands: Commands,
    afterburner_query: Query<Entity, With<AfterburnerFlame>>,
) {
    for entity in afterburner_query.iter() {
        commands.entity(entity).despawn();
    }
}
