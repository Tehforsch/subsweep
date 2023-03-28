use std::path::Path;
use std::path::PathBuf;

use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::prelude::*;

use crate::prelude::Float;
use crate::prelude::Simulation;
use crate::units::Dimension;
use crate::units::Quantity;

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
