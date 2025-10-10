use crate::components::{Helicopter, MainRotor, RotorBlurDisk, SimpleHelicopterSpecs, TailRotor};
use bevy::prelude::*;

type MainBlurQuery<'w, 's> = Query<
    'w,
    's,
    (&'static RotorBlurDisk, &'static mut Visibility),
    (With<RotorBlurDisk>, Without<TailRotor>),
>;

type TailBlurQuery<'w, 's> = Query<
    'w,
    's,
    (&'static RotorBlurDisk, &'static mut Visibility),
    (With<RotorBlurDisk>, With<TailRotor>),
>;

pub fn update_rotor_blur_visibility(
    helicopter_query: Query<&SimpleHelicopterSpecs, With<Helicopter>>,
    mut main_blur_query: MainBlurQuery,
    mut tail_blur_query: TailBlurQuery,
    mut rotor_blade_query: Query<&mut Visibility, (With<MainRotor>, Without<RotorBlurDisk>)>,
) {
    let (main_rpm, tail_rpm) = helicopter_query
        .iter()
        .next()
        .map(|specs| (specs.main_rotor_rpm, specs.tail_rotor_rpm))
        .unwrap_or((20.0, 35.0));

    for (blur_disk, mut visibility) in main_blur_query.iter_mut() {
        if blur_disk.is_main_rotor && main_rpm >= blur_disk.min_rpm_for_blur {
            *visibility = Visibility::Visible;
            for mut blade_vis in rotor_blade_query.iter_mut() {
                *blade_vis = Visibility::Hidden;
            }
        } else if !blur_disk.is_main_rotor {
            *visibility = Visibility::Hidden;
            for mut blade_vis in rotor_blade_query.iter_mut() {
                *blade_vis = Visibility::Visible;
            }
        }
    }

    for (blur_disk, mut visibility) in tail_blur_query.iter_mut() {
        if !blur_disk.is_main_rotor && tail_rpm >= blur_disk.min_rpm_for_blur {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
