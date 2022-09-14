use std::collections::HashSet;

use bevy::prelude::App;

use crate::communication::WorldRank;

#[derive(Default)]
struct Labels(HashSet<&'static str>);

pub fn run_once(label: &'static str, app: &mut App, f: impl Fn(&mut App) -> ()) {
    let labels_resource_exists = app.world.get_resource_mut::<Labels>().is_some();
    if !labels_resource_exists {
        app.world.insert_resource(Labels::default());
    }
    let mut labels = app.world.get_resource_mut::<Labels>().unwrap();
    let contains_label = labels.0.contains(label);
    labels.0.insert(label);
    if !contains_label {
        f(app);
    }
}

pub fn is_main_rank(app: &App) -> bool {
    app.world.get_resource::<WorldRank>().unwrap().is_main()
}

pub fn get_parameters<P: Clone + Sync + Send + 'static>(app: &App) -> P {
    app.world.get_resource::<P>().unwrap().clone()
}
