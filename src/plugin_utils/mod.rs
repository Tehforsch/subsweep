mod tenet_plugin;

use std::collections::HashSet;

use bevy::prelude::App;

use crate::communication::WorldRank;
use crate::named::Named;

#[derive(Default)]
struct RunOnceLabels(HashSet<&'static str>);

#[derive(Default)]
struct AlreadyAddedLabels(HashSet<&'static str>);

pub fn run_once<P: Named>(app: &mut App, f: impl Fn(&mut App)) {
    let mut labels = app
        .world
        .get_resource_or_insert_with(RunOnceLabels::default);
    if labels.0.insert(P::name()) {
        f(app);
    }
}

/// Panics if a named item was (accidentally) added twice
pub fn panic_if_already_added<P: Named>(app: &mut App) {
    let mut labels = app
        .world
        .get_resource_or_insert_with(AlreadyAddedLabels::default);
    if !labels.0.insert(P::name()) {
        panic!("Added twice: {}", P::name())
    }
}

pub fn is_main_rank(app: &App) -> bool {
    app.world.get_resource::<WorldRank>().unwrap().is_main()
}

pub fn get_parameters<P: Clone + Sync + Send + 'static>(app: &App) -> P {
    app.world.get_resource::<P>().unwrap().clone()
}
