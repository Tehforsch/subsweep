use std::marker::PhantomData;

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

use crate::communication::WorldRank;
use crate::physics::PhysicsStages;
use crate::plugin_utils::run_once;

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
            .insert_resource(OutputFile::default())
            .add_system_to_stage(OutputStages::Output, open_file_system)
            .add_system_to_stage(
                OutputStages::Output,
                close_file_system.after(open_file_system),
            );
        });
        let output_name = self.output_name.clone();
        app.add_system_to_stage(
            OutputStages::Output,
            (move |query: Query<&T>, file: ResMut<OutputFile>| {
                Self::write_output(&output_name, query, file)
            })
            .after(open_file_system)
            .before(close_file_system),
        );
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

fn open_file_system(mut file: ResMut<OutputFile>, rank: Res<WorldRank>) {
    assert!(file.f.is_none());
    file.f =
        Some(File::create(&format!("out{}.hdf5", rank.0)).expect("Failed to open output file"));
}

fn close_file_system(mut file: ResMut<OutputFile>) {
    file.f = None;
}
