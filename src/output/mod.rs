mod parameters;

use std::fs;
use std::marker::PhantomData;

use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::AmbiguitySetLabel;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::StageLabel;
use bevy::prelude::SystemStage;
use hdf5::File;
use hdf5::H5Type;

use self::parameters::Parameters;
use crate::communication::WorldRank;
use crate::parameters::ParameterPlugin;
use crate::physics::PhysicsStages;
use crate::physics::Time;
use crate::plugin_utils::run_once;
use crate::units;

#[derive(AmbiguitySetLabel)]
struct OutputSystemsAmbiguitySet;

#[derive(StageLabel)]
enum OutputStages {
    Output,
}

pub struct OutputPlugin<T> {
    _marker: PhantomData<T>,
    output_name: String,
}

impl<T> OutputPlugin<T> {
    pub fn new(name: &str) -> Self {
        Self {
            _marker: PhantomData::default(),
            output_name: name.into(),
        }
    }
}

#[derive(Default)]
struct OutputFile {
    f: Option<File>,
}

impl<T: Clone + H5Type + Component + Sync + Send + 'static> Plugin for OutputPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        run_once("output_plugin", app, |app| {
            app.add_stage_after(
                PhysicsStages::Gravity,
                OutputStages::Output,
                SystemStage::parallel(),
            )
            .add_plugin(ParameterPlugin::<Parameters>::new("output"))
            .insert_resource(OutputFile::default())
            .add_startup_system(set_output_timer_system)
            .add_startup_system(make_output_dir_system)
            .add_system_to_stage(
                OutputStages::Output,
                open_file_system.with_run_criteria(OutputTimer::run_criterion),
            )
            .add_system_to_stage(
                OutputStages::Output,
                close_file_system
                    .after(open_file_system)
                    .with_run_criteria(OutputTimer::run_criterion),
            )
            .add_system_to_stage(
                OutputStages::Output,
                update_output_timer_system
                    .after(close_file_system)
                    .with_run_criteria(OutputTimer::run_criterion),
            );
        });
        let output_name = self.output_name.clone();
        app.add_system_to_stage(
            OutputStages::Output,
            (move |query: Query<&T>, file: ResMut<OutputFile>| {
                Self::write_output(&output_name, query, file)
            })
            .after(open_file_system)
            .before(close_file_system)
            .in_ambiguity_set(OutputSystemsAmbiguitySet)
            .with_run_criteria(OutputTimer::run_criterion),
        );
    }
}

fn make_output_dir_system(parameters: Res<Parameters>) {
    fs::create_dir_all(&parameters.output_dir).expect(&format!(
        "Failed to create output dir: {:?}",
        parameters.output_dir
    ));
}

fn set_output_timer_system(mut commands: Commands, parameters: Res<Parameters>) {
    commands.insert_resource(OutputTimer {
        next_output_time: parameters.time_first_snapshot,
        snapshot_num: 0,
    });
}

struct OutputTimer {
    next_output_time: units::Time,
    snapshot_num: usize,
}

impl OutputTimer {
    fn run_criterion(time: Res<Time>, timer: Res<Self>) -> ShouldRun {
        if time.0 >= timer.next_output_time {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        }
    }
}

impl<T: Clone + H5Type + Component> OutputPlugin<T> {
    fn write_output(name: &str, query: Query<&T>, file: ResMut<OutputFile>) {
        let f = file.f.as_ref().unwrap();
        let data: Vec<T> = query.iter().cloned().collect();
        f.new_dataset_builder()
            .with_data(&data)
            .create(name)
            .expect("Failed to write dataset");
    }
}

fn open_file_system(
    mut file: ResMut<OutputFile>,
    rank: Res<WorldRank>,
    parameters: Res<Parameters>,
    output_timer: Res<OutputTimer>,
) {
    assert!(file.f.is_none());
    let filename = &format!("snapshot_{}_{}.hdf5", output_timer.snapshot_num, rank.0);
    file.f = Some(
        File::create(&parameters.output_dir.join(filename)).expect("Failed to open output file"),
    );
}

fn close_file_system(mut file: ResMut<OutputFile>) {
    file.f = None;
}

fn update_output_timer_system(mut output_timer: ResMut<OutputTimer>, parameters: Res<Parameters>) {
    output_timer.snapshot_num += 1;
    output_timer.next_output_time += parameters.time_between_snapshots;
}
