mod simulation;
mod tenet_plugin;

use std::collections::HashSet;

pub use simulation::Simulation;
pub use tenet_plugin::TenetPlugin;

use crate::communication::WorldRank;
use crate::named::Named;

#[derive(Default)]
struct RunOnceLabels(HashSet<&'static str>);

#[derive(Default)]
struct AlreadyAddedLabels(HashSet<&'static str>);

pub fn run_once<P: Named>(sim: &mut Simulation, f: impl Fn(&mut Simulation)) {
    let mut labels = sim.get_resource_or_insert_with(RunOnceLabels::default);
    if labels.0.insert(P::name()) {
        f(sim);
    }
}

pub fn is_main_rank(app: &Simulation) -> bool {
    app.get_resource::<WorldRank>().unwrap().is_main()
}

pub fn get_parameters<P: Clone + Sync + Send + 'static>(sim: &Simulation) -> P {
    sim.unwrap_resource::<P>().clone()
}
