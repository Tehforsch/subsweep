mod simulation;
mod tenet_plugin;

use std::collections::HashSet;

pub use simulation::Simulation;
pub use tenet_plugin::TenetPlugin;

#[derive(Default)]
struct RunOnceLabels(HashSet<&'static str>);

#[derive(Default)]
struct AlreadyAddedLabels(HashSet<&'static str>);
