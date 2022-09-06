mod attribute_plugin;
mod dataset_plugin;
mod parameters;
mod timer;

use std::fs;

use bevy::prelude::AmbiguitySetLabel;
use bevy::prelude::App;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::StageLabel;
use bevy::prelude::SystemStage;
use hdf5::File;

pub use self::dataset_plugin::DatasetPlugin;
use self::parameters::Parameters;
use self::timer::Timer;
use crate::communication::WorldRank;
use crate::parameters::ParameterPlugin;
use crate::physics::PhysicsStages;

#[derive(AmbiguitySetLabel)]
struct OutputSystemsAmbiguitySet;

#[derive(StageLabel)]
enum OutputStages {
    Output,
}

#[derive(Default)]
struct OutputFile {
    f: Option<File>,
}

fn output_setup(app: &mut App) {
    app.add_stage_after(
        PhysicsStages::Gravity,
        OutputStages::Output,
        SystemStage::parallel(),
    )
    .add_plugin(ParameterPlugin::<Parameters>::new("output"))
    .insert_resource(OutputFile::default())
    .add_startup_system(Timer::initialize_system)
    .add_startup_system(make_output_dir_system)
    .add_system_to_stage(
        OutputStages::Output,
        open_file_system.with_run_criteria(Timer::run_criterion),
    )
    .add_system_to_stage(
        OutputStages::Output,
        close_file_system
            .after(open_file_system)
            .with_run_criteria(Timer::run_criterion),
    )
    .add_system_to_stage(
        OutputStages::Output,
        Timer::update_system
            .after(close_file_system)
            .with_run_criteria(Timer::run_criterion),
    );
}

fn make_output_dir_system(parameters: Res<Parameters>) {
    fs::create_dir_all(&parameters.output_dir).expect(&format!(
        "Failed to create output dir: {:?}",
        parameters.output_dir
    ));
}

fn open_file_system(
    mut file: ResMut<OutputFile>,
    rank: Res<WorldRank>,
    parameters: Res<Parameters>,
    output_timer: Res<Timer>,
) {
    assert!(file.f.is_none());
    let filename = &format!("snapshot_{}_{}.hdf5", output_timer.snapshot_num(), rank.0);
    file.f = Some(
        File::create(&parameters.output_dir.join(filename)).expect("Failed to open output file"),
    );
}

fn close_file_system(mut file: ResMut<OutputFile>) {
    file.f = None;
}
