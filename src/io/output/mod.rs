mod attribute;
mod attribute_plugin;
pub(crate) mod parameters;
pub(super) mod plugin;
mod timer;

use std::fs;
use std::path::Path;

use bevy::prelude::info;
use bevy::prelude::AmbiguitySetLabel;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::StageLabel;
use hdf5::File;

pub use self::attribute::Attribute;
pub use self::attribute_plugin::AttributeOutputPlugin;
use self::parameters::OutputParameters;
pub use self::plugin::OutputPlugin;
use self::timer::Timer;
use crate::communication::WorldRank;
use crate::parameter_plugin::ParameterFileContents;
use crate::prelude::WorldSize;

#[derive(AmbiguitySetLabel)]
pub(super) struct OutputSystemsAmbiguitySet;

#[derive(StageLabel)]
pub enum OutputStages {
    Output,
}

#[derive(Default)]
pub(super) struct OutputFile {
    pub f: Option<File>,
}

pub struct ShouldWriteOutput(pub bool);

fn write_used_parameters_system(
    parameter_file_contents: Res<ParameterFileContents>,
    parameters: Res<OutputParameters>,
) {
    fs::write(
        &parameters
            .output_dir
            .join(&parameters.used_parameters_filename),
        &**parameter_file_contents,
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to write used parameters to file: {}: {}",
            **parameter_file_contents, e
        )
    });
}

fn make_output_dirs_system(parameters: Res<OutputParameters>) {
    if parameters.output_dir.exists() {
        match parameters.handle_existing_output {
            parameters::HandleExistingOutput::Panic => panic!(
                "Output folder at {} already exists.",
                parameters.output_dir.to_str().unwrap()
            ),
            parameters::HandleExistingOutput::Delete => {
                fs::remove_dir_all(&parameters.output_dir)
                    .unwrap_or_else(|e| panic!("Failed to remove output directory. {}", e));
            }
            parameters::HandleExistingOutput::Overwrite => {}
        }
    }
    fs::create_dir_all(&parameters.output_dir)
        .unwrap_or_else(|_| panic!("Failed to create output dir: {:?}", parameters.output_dir));
    fs::create_dir_all(&parameters.snapshot_dir()).unwrap_or_else(|_| {
        panic!(
            "Failed to create snapshots dir: {:?}",
            parameters.snapshot_dir()
        )
    });
}

fn make_snapshot_dir(snapshot_dir: &Path) {
    fs::create_dir_all(snapshot_dir)
        .unwrap_or_else(|_| panic!("Failed to create snapshot dir: {:?}", snapshot_dir));
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
    let snapshot_name = format!(
        "{:0snap_padding$}",
        output_timer.snapshot_num(),
        snap_padding = parameters.snapshot_padding
    );
    let snapshot_dir = parameters.snapshot_dir().join(&snapshot_name);
    make_snapshot_dir(&snapshot_dir);
    let filename = &format!(
        "{:0rank_padding$}.hdf5",
        rank.0,
        rank_padding = rank_padding
    );
    info!("Writing snapshot: {}", &snapshot_name);
    file.f = Some(File::create(snapshot_dir.join(filename)).expect("Failed to open output file"));
}

fn close_file_system(mut file: ResMut<OutputFile>) {
    file.f = None;
}
