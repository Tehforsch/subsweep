use std::marker::PhantomData;

use bevy::prelude::Component;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::StageLabel;
use bevy::prelude::SystemStage;

use crate::physics::PhysicsStages;
use crate::plugin_utils::run_once;

#[derive(StageLabel)]
enum OutputStages {
    Output,
}

#[derive(Default)]
struct OutputPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T: Component + Sync + Send + 'static> Plugin for OutputPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        run_once("output_plugin", app, |app| {
            app.add_stage_after(
                PhysicsStages::Gravity,
                OutputStages::Output,
                SystemStage::parallel(),
            );
        });
        app.add_system_to_stage(OutputStages::Output, Self::output_system);
    }
}

impl<T: Component> OutputPlugin<T> {
    fn output_system(query: Query<&T>) {}
}
