mod attribute;
mod attribute_plugin;
pub(super) mod dataset_plugin;
mod parameters;
mod timer;

use std::fs;

use bevy::prelude::info;
use bevy::prelude::AmbiguitySetLabel;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::StageLabel;
use hdf5::File;

pub use self::attribute::Attribute;
pub use self::attribute_plugin::AttributeOutputPlugin;
pub use self::dataset_plugin::DatasetOutputPlugin;
pub use self::parameters::OutputParameters;
use self::timer::Timer;
use crate::communication::WorldRank;
use crate::named::Named;
use crate::prelude::WorldSize;
use crate::simulation::Simulation;

#[derive(AmbiguitySetLabel)]
struct OutputSystemsAmbiguitySet;

#[derive(StageLabel)]
pub enum OutputStages {
    Output,
}

#[derive(Default)]
struct OutputFile {
    f: Option<File>,
}

pub struct ShouldWriteOutput(pub bool);

fn output_setup(sim: &mut Simulation) {
    sim.add_parameter_type::<OutputParameters>()
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

fn make_output_dir_system(parameters: Res<OutputParameters>) {
    fs::create_dir_all(&parameters.output_dir)
        .unwrap_or_else(|_| panic!("Failed to create output dir: {:?}", parameters.output_dir));
}

fn open_file_system(
    mut file: ResMut<OutputFile>,
    rank: Res<WorldRank>,
    world_size: Res<WorldSize>,
    parameters: Res<OutputParameters>,
    output_timer: Res<Timer>,
) {
    assert!(file.f.is_none());
    let rank_padding = ((**world_size as f64).log10().floor() as usize) + 1;
    let filename = &format!(
        "snapshot_{:0snap_padding$}_{:0rank_padding$}.hdf5",
        output_timer.snapshot_num(),
        rank.0,
        snap_padding = parameters.snapshot_padding,
        rank_padding = rank_padding
    );
    info!("Writing snapshot: {}", output_timer.snapshot_num());
    file.f = Some(
        File::create(&parameters.output_dir.join(filename)).expect("Failed to open output file"),
    );
}

fn close_file_system(mut file: ResMut<OutputFile>) {
    file.f = None;
}

#[derive(Named)]
struct OutputMarker;

fn add_output_system<T: Named, P>(
    sim: &mut Simulation,
    system: impl ParallelSystemDescriptorCoercion<P>,
) {
    if !sim.already_added::<OutputMarker>() {
        output_setup(sim)
    }
    if OutputParameters::is_desired_field::<T>(sim) {
        sim.add_system_to_stage(
            OutputStages::Output,
            system
                .after(open_file_system)
                .before(close_file_system)
                .in_ambiguity_set(OutputSystemsAmbiguitySet)
                .with_run_criteria(Timer::run_criterion),
        );
    }
}
