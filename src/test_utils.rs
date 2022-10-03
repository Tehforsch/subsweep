use std::path::Path;
use std::path::PathBuf;

use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::prelude::*;

use crate::prelude::Simulation;

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
