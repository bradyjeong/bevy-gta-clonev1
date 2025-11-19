#![allow(clippy::type_complexity)]
use crate::components::{
    Helicopter, HelicopterRuntime, MainRotor, RotorBlurDisk, SimpleHelicopterSpecs,
    SimpleHelicopterSpecsHandle, TailRotor,
};
use bevy::prelude::*;

type MainBlurQuery<'w, 's> = Query<
    'w,
    's,
    (&'static RotorBlurDisk, &'static mut Visibility),
    (With<RotorBlurDisk>, With<MainRotor>, Without<TailRotor>),
>;

type TailBlurQuery<'w, 's> = Query<
    'w,
    's,
    (&'static RotorBlurDisk, &'static mut Visibility),
    (With<RotorBlurDisk>, With<TailRotor>),
>;

/// OPTIMIZATION: Or<Changed<Children>> handles initialization path for newly spawned entities
/// OPTIMIZATION: With<MainRotor>/With<TailRotor> filters prevent query conflicts
/// OPTIMIZATION: Only writes Visibility when it actually changes
/// CRITICAL FIX: Per-helicopter iteration prevents rotor blur cross-contamination
pub fn update_rotor_blur_visibility(
    heli_specs_assets: Res<Assets<SimpleHelicopterSpecs>>,
    helicopter_query: Query<
        (
            &SimpleHelicopterSpecsHandle,
            &HelicopterRuntime,
            &Children,
            &GlobalTransform,
        ),
        (
            With<Helicopter>,
            Or<(
                Changed<SimpleHelicopterSpecsHandle>,
                Changed<HelicopterRuntime>,
                Changed<Children>,
            )>,
        ),
    >,
    mut main_blur_query: MainBlurQuery,
    mut tail_blur_query: TailBlurQuery,
    mut rotor_blade_query: Query<&mut Visibility, (With<MainRotor>, Without<RotorBlurDisk>)>,
    children_query: Query<&Children>,
    player_query: Query<&GlobalTransform, With<crate::components::ActiveEntity>>,
) {
    let player_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation())
        .unwrap_or_default();
    let has_player = player_query.iter().next().is_some();

    for (specs_handle, runtime, helicopter_children, heli_transform) in helicopter_query.iter() {
        // Distance culling
        if has_player && heli_transform.translation().distance(player_pos) > 200.0 {
            // Force blur Hidden, blades Visible (default state)
            for heli_child in helicopter_children.iter() {
                let Ok(visual_body_children) = children_query.get(heli_child) else {
                    continue;
                };

                for child in visual_body_children.iter() {
                    // Handle Main Blur Disk
                    if let Ok((blur_disk, mut visibility)) = main_blur_query.get_mut(child) {
                        if blur_disk.is_main_rotor {
                            if *visibility != Visibility::Hidden {
                                *visibility = Visibility::Hidden;
                            }
                            // Ensure blades are visible
                            if let Ok(child_children) = children_query.get(child) {
                                for blade_child in child_children.iter() {
                                    if let Ok(mut blade_vis) =
                                        rotor_blade_query.get_mut(blade_child)
                                    {
                                        if *blade_vis != Visibility::Visible {
                                            *blade_vis = Visibility::Visible;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Handle Tail Blur Disk
                    if let Ok((_, mut visibility)) = tail_blur_query.get_mut(child) {
                        if *visibility != Visibility::Hidden {
                            *visibility = Visibility::Hidden;
                        }
                    }
                }
            }
            continue;
        }

        let Some(specs) = heli_specs_assets.get(&specs_handle.0) else {
            continue;
        };
        let main_rpm = specs.main_rotor_rpm * runtime.rpm;
        let tail_rpm = specs.tail_rotor_rpm * runtime.rpm;

        // Navigate to HelicopterVisualBody children (rotors/blur disks are grandchildren now)
        for heli_child in helicopter_children.iter() {
            let Ok(visual_body_children) = children_query.get(heli_child) else {
                continue;
            };

            for child in visual_body_children.iter() {
                if let Ok((blur_disk, mut visibility)) = main_blur_query.get_mut(child) {
                    if blur_disk.is_main_rotor && main_rpm >= blur_disk.min_rpm_for_blur {
                        if *visibility != Visibility::Visible {
                            *visibility = Visibility::Visible;
                        }
                        if let Ok(child_children) = children_query.get(child) {
                            for blade_child in child_children.iter() {
                                if let Ok(mut blade_vis) = rotor_blade_query.get_mut(blade_child) {
                                    if *blade_vis != Visibility::Hidden {
                                        *blade_vis = Visibility::Hidden;
                                    }
                                }
                            }
                        }
                    } else if blur_disk.is_main_rotor {
                        if *visibility != Visibility::Hidden {
                            *visibility = Visibility::Hidden;
                        }
                        if let Ok(child_children) = children_query.get(child) {
                            for blade_child in child_children.iter() {
                                if let Ok(mut blade_vis) = rotor_blade_query.get_mut(blade_child) {
                                    if *blade_vis != Visibility::Visible {
                                        *blade_vis = Visibility::Visible;
                                    }
                                }
                            }
                        }
                    }
                }

                if let Ok((blur_disk, mut visibility)) = tail_blur_query.get_mut(child) {
                    let new_visibility =
                        if !blur_disk.is_main_rotor && tail_rpm >= blur_disk.min_rpm_for_blur {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };

                    if *visibility != new_visibility {
                        *visibility = new_visibility;
                    }
                }
            }
        }
    }
}
