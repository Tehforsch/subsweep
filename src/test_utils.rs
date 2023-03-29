use std::path::Path;
use std::path::PathBuf;

use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::prelude::*;

use crate::domain::LeafData;
use crate::prelude::Float;
use crate::prelude::Simulation;
use crate::units::Dimension;
use crate::units::Quantity;
use crate::units::VecLength;

// This is currently only used in tests with the local communication
// but will very likely be used more, so prevent dead code warning
#[allow(dead_code)]
pub fn run_system_on_sim<P>(sim: &mut Simulation, system: impl IntoSystemDescriptor<P>) {
    run_system_on_world(sim.world(), system);
}

pub fn run_system_on_world<P>(world: &mut World, system: impl IntoSystemDescriptor<P>) {
    let mut stage = SystemStage::single_threaded().with_system(system);
    stage.run(world);
}

pub fn tests_path() -> PathBuf {
    Path::new(file!()).parent().unwrap().join("../tests")
}

pub fn assert_is_close<const U: Dimension>(x: Quantity<f64, U>, y: Quantity<f64, U>) {
    assert!(
        (x - y).abs().value_unchecked() < f64::EPSILON,
        "{} {}",
        x.value_unchecked(),
        y.value_unchecked()
    )
}

pub fn assert_float_is_close(x: Float, y: Float) {
    assert!((x - y).abs() < 10.0 * f64::EPSILON, "{} {}", x, y)
}

pub fn assert_float_is_close_high_error(x: Float, y: Float) {
    assert!((x - y).abs() < 1e3 * f64::EPSILON, "{} {}", x, y)
}

#[cfg(not(feature = "2d"))]
pub fn assert_vec_is_close<const U: Dimension>(
    x: Quantity<crate::prelude::MVec, U>,
    y: Quantity<crate::prelude::MVec, U>,
) {
    assert!(
        (x - y).length().value_unchecked() < f64::EPSILON,
        "{} {}",
        x.value_unchecked(),
        y.value_unchecked()
    )
}

pub fn get_particles(n: i32, m: i32) -> Vec<LeafData> {
    (1..n + 1)
        .flat_map(move |x| {
            (1..m + 1).map(move |y| LeafData {
                entity: Entity::from_raw((x * n + y) as u32),
                #[cfg(feature = "2d")]
                pos: VecLength::meters(x as f64, y as f64),
                #[cfg(feature = "3d")]
                pos: VecLength::meters(x as f64, y as f64, x as f64 * y as f64),
            })
        })
        .collect()
}
