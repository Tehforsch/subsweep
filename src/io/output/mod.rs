mod attribute;
pub(crate) mod parameters;
pub(super) mod plugin;
pub mod timer;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::ResMut;
use bevy_ecs::prelude::Resource;
use bevy_ecs::system::Commands;
use bevy_ecs::system::NonSend;
use hdf5::Dataset;
use hdf5::File;
use log::info;
use mpi::traits::CommunicatorCollectives;
use mpi::traits::Equivalence;

pub use self::attribute::Attribute;
pub use self::attribute::ToAttribute;
use self::parameters::OutputParameters;
pub use self::plugin::OutputPlugin;
use self::timer::Timer;
use super::file_distribution::Region;
use super::input::NumParticlesTotal;
use super::to_dataset::ToDataset;
use super::DatasetDescriptor;
use super::OutputDatasetDescriptor;
use crate::communication::communicator::Communicator;
use crate::communication::MPI_UNIVERSE;
use crate::io::file_distribution::get_rank_output_assignment_for_rank;
use crate::io::file_distribution::RankAssignment;
use crate::parameter_plugin::ParameterFileContents;
use crate::prelude::Particles;
use crate::prelude::WorldRank;
use crate::units::Dimension;

pub const SCALE_FACTOR_IDENTIFIER: &str = "scale_factor_si";
pub const LENGTH_IDENTIFIER: &str = "scaling_length";
pub const TIME_IDENTIFIER: &str = "scaling_time";
pub const MASS_IDENTIFIER: &str = "scaling_mass";
pub const TEMPERATURE_IDENTIFIER: &str = "scaling_temperature";
pub const H_SCALING_IDENTIFIER: &str = "scaling_h";
pub const A_SCALING_IDENTIFIER: &str = "scaling_a";

// Output order:
// Output proceeds as follows
// 1. Main rank creates files
// 2. Main rank creates datasets with correct shape
// 3. Main rank creates attributes to the newly created datasets
// 4. Main rank closes files
// 5. All ranks open files
// 6. All ranks write data
// 7. All ranks close files

#[derive(Default, Resource)]
pub struct OutputFiles(pub Option<Vec<FileWithRegion>>);

#[derive(Debug)]
pub struct FileWithRegion {
    file: File,
    region: Region,
}

fn write_used_parameters_system(
    parameter_file_contents: Res<ParameterFileContents>,
    parameters: Res<OutputParameters>,
) {
    fs::write(
        parameters
            .output_dir
            .join(&parameters.used_parameters_filename),
        parameter_file_contents.contents(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to write used parameters to file: {}: {}",
            &parameter_file_contents.contents(),
            e
        )
    });
}

pub fn make_output_dirs(parameters: &OutputParameters) {
    if parameters.output_dir.exists() {
        match parameters.handle_existing_output {
            parameters::HandleExistingOutput::Panic => panic!(
                "Output folder at {} already exists.",
                parameters.output_dir.to_str().unwrap()
            ),
            parameters::HandleExistingOutput::Delete => {
                fs::remove_dir_all(&parameters.output_dir)
                    .unwrap_or_else(|e| panic!("Failed to remove output directory. {e}"));
            }
            parameters::HandleExistingOutput::Overwrite => {}
        }
    }
    fs::create_dir_all(&parameters.output_dir)
        .unwrap_or_else(|_| panic!("Failed to create output dir: {:?}", parameters.output_dir));
    fs::create_dir_all(parameters.snapshot_dir()).unwrap_or_else(|_| {
        panic!(
            "Failed to create snapshots dir: {:?}",
            parameters.snapshot_dir()
        )
    });
}

fn make_snapshot_dir(snapshot_dir: &Path) {
    fs::create_dir_all(snapshot_dir)
        .unwrap_or_else(|_| panic!("Failed to create snapshot dir: {snapshot_dir:?}"));
}

pub fn compute_output_rank_assignment_system(
    mut commands: Commands,
    rank: Res<WorldRank>,
    parameters: Res<OutputParameters>,
    particles: Particles<Entity>,
    num_particles_total: Res<NumParticlesTotal>,
) {
    #[derive(Equivalence, Clone)]
    struct NumParticles(usize);
    let num_particles_local = NumParticles(particles.iter().count());
    let num_particles_per_rank =
        Communicator::<NumParticles>::new().all_gather(&num_particles_local);
    let num_particles_per_rank: Vec<_> = num_particles_per_rank.into_iter().map(|x| x.0).collect();
    assert_eq!(
        num_particles_per_rank.iter().sum::<usize>(),
        num_particles_total.0
    );
    let rank_assignment = get_rank_output_assignment_for_rank(
        &num_particles_per_rank,
        parameters.num_output_files,
        **rank,
    );
    commands.insert_resource(rank_assignment);
}

fn get_snapshot_dir(parameters: &OutputParameters, output_timer: &Timer) -> PathBuf {
    let snapshot_name = format!(
        "{:0snap_padding$}",
        output_timer.snapshot_num(),
        snap_padding = parameters.snapshot_padding
    );
    parameters.snapshot_dir().join(&snapshot_name)
}

fn get_output_files(
    parameters: &OutputParameters,
    output_timer: &Timer,
    assignment: &RankAssignment,
    get_file: impl Fn(PathBuf) -> hdf5::Result<File>,
) -> Vec<FileWithRegion> {
    let file_index_padding = ((parameters.num_output_files as f64).log10().floor() as usize) + 1;
    let snapshot_dir = get_snapshot_dir(parameters, output_timer);
    make_snapshot_dir(&snapshot_dir);
    assignment
        .regions
        .iter()
        .map(|region| {
            let filename = &format!(
                "{:0file_index_padding$}.hdf5",
                region.file_index,
                file_index_padding = file_index_padding
            );
            let file = get_file(snapshot_dir.join(filename)).expect("Failed to open output file");
            FileWithRegion {
                file,
                region: region.clone(),
            }
        })
        .collect()
}

fn create_file_system(
    mut file: ResMut<OutputFiles>,
    parameters: Res<OutputParameters>,
    output_timer: Res<Timer>,
    num_particles_total: Res<NumParticlesTotal>,
    _rank: Res<WorldRank>,
) {
    info!("Writing snapshot: {}", &output_timer.snapshot_num());
    assert!(file.0.is_none());

    // In order to know how large the datasets are that we need to create:
    // Compute rank assignment for one rank.
    let assignment = get_rank_output_assignment_for_rank(
        &[num_particles_total.0],
        parameters.num_output_files,
        0,
    );
    file.0 = Some(get_output_files(
        &parameters,
        &output_timer,
        &assignment,
        create_file_rw,
    ));
}

#[cfg(feature = "parallel-hdf5")]
fn make_mpi_file_builder() -> hdf5::FileBuilder {
    use hdf5::file::LibraryVersion;
    use hdf5::plist;
    use hdf5::FileBuilder;
    use mpi::traits::AsRaw;

    let comm = MPI_UNIVERSE.world();
    let fapl = plist::FileAccess::build()
        .mpio(comm.as_raw(), None)
        .libver_bounds(LibraryVersion::V18, LibraryVersion::V110)
        .finish()
        .unwrap();
    let mut builder = FileBuilder::new();
    builder.set_access_plist(&fapl).unwrap();
    builder
}

#[cfg(feature = "parallel-hdf5")]
fn create_file_rw(path: PathBuf) -> hdf5::Result<File> {
    use hdf5::plist;

    let mut builder = make_mpi_file_builder();
    let fcpl = plist::FileCreate::build().finish().unwrap();
    builder.set_create_plist(&fcpl).unwrap().create(path)
}

#[cfg(feature = "parallel-hdf5")]
fn open_file_rw(path: PathBuf) -> hdf5::Result<File> {
    let builder = make_mpi_file_builder();
    builder.open_rw(path)
}

#[cfg(not(feature = "parallel-hdf5"))]
fn create_file_rw(path: PathBuf) -> hdf5::Result<File> {
    File::create(path)
}

#[cfg(not(feature = "parallel-hdf5"))]
fn open_file_rw(path: PathBuf) -> hdf5::Result<File> {
    File::open_rw(path)
}

fn open_file_system(
    mut file: ResMut<OutputFiles>,
    parameters: Res<OutputParameters>,
    output_timer: Res<Timer>,
    assignment: Res<RankAssignment>,
) {
    assert!(file.0.is_none());
    file.0 = Some(get_output_files(
        &parameters,
        &output_timer,
        &assignment,
        open_file_rw,
    ))
}

fn close_file_system(mut file: ResMut<OutputFiles>) {
    file.0 = None;
}

pub fn create_dataset_system<T: Component + ToDataset>(
    file: ResMut<OutputFiles>,
    descriptor: NonSend<OutputDatasetDescriptor<T>>,
) {
    let files = file.0.as_ref().unwrap();
    create_dataset_in_files::<T>(files, &descriptor);
}

pub fn create_dataset_in_files<T: ToDataset>(
    files: &[FileWithRegion],
    descriptor: &DatasetDescriptor,
) {
    for FileWithRegion { file, region } in files.iter() {
        assert!(region.start == 0);
        let dataset = file
            .new_dataset::<T>()
            .shape(&[region.end - region.start])
            .create(descriptor.dataset_name())
            .expect("Failed to create dataset");
        add_dimension_attrs::<T>(&dataset);
    }
}

pub fn write_dataset_system<T: Component + ToDataset>(
    query: Particles<&T>,
    file: ResMut<OutputFiles>,
    descriptor: NonSend<OutputDatasetDescriptor<T>>,
) {
    let files = file.0.as_ref().unwrap();
    let data: Vec<T> = query.iter().cloned().collect();
    write_dataset_to_files(data, files, &descriptor);
}

pub fn write_dataset_to_files<T: ToDataset>(
    data: Vec<T>,
    files: &[FileWithRegion],
    descriptor: &DatasetDescriptor,
) {
    let mut data_start = 0;
    for FileWithRegion { file, region } in files.iter() {
        let dataset = file
            .dataset(&descriptor.dataset_name())
            .expect("Failed to open dataset");
        let data_end = data_start + region.size();
        dataset
            .write_slice(&data[data_start..data_end], region.start..region.end)
            .expect("Failed to write slice to dataset");
        data_start += region.size();
    }
    assert_eq!(data_start, data.len());
}

pub fn add_dimension_attrs<T: ToDataset>(dataset: &Dataset) {
    let attr = dataset
        .new_attr::<f64>()
        .shape(())
        .create(SCALE_FACTOR_IDENTIFIER)
        .unwrap();
    let dimension = T::dimension();
    let scale_factor = dimension.base_conversion_factor();
    attr.write_scalar(&scale_factor).unwrap();
    // Unpack this slightly awkwardly here to make sure that we
    // remember to extend it once more units are added to the
    // Dimension struct
    let Dimension {
        length,
        time,
        mass,
        temperature,
        h,
        a,
    } = dimension;
    write_dimension(dataset, LENGTH_IDENTIFIER, length);
    write_dimension(dataset, TIME_IDENTIFIER, time);
    write_dimension(dataset, MASS_IDENTIFIER, mass);
    write_dimension(dataset, TEMPERATURE_IDENTIFIER, temperature);
    write_dimension(dataset, H_SCALING_IDENTIFIER, h);
    write_dimension(dataset, A_SCALING_IDENTIFIER, a);
}

fn write_dimension(dataset: &Dataset, identifier: &str, dimension: i32) {
    let attr = dataset
        .new_attr::<i32>()
        .shape(())
        .create(identifier)
        .unwrap();
    attr.write_scalar(&dimension).unwrap();
}

#[cfg(feature = "parallel-hdf5")]
pub fn init_wait_for_other_ranks_system(mut perf: ResMut<crate::performance::Performance>) {
    // Make sure all ranks wait for the main rank to arrive who
    // creates the datasets

    perf.start("output_dataset");

    let world = MPI_UNIVERSE.world();
    world.barrier();
}

#[cfg(feature = "parallel-hdf5")]
pub fn finish_wait_for_other_ranks_system(mut perf: ResMut<crate::performance::Performance>) {
    perf.stop("output_dataset");
}

#[cfg(not(feature = "parallel-hdf5"))]
pub fn init_wait_for_other_ranks_system(
    world_size: Res<crate::prelude::WorldSize>,
    rank: Res<WorldRank>,
) {
    if **world_size > 10 {
        log::warn!("Serial hdf5 output is very slow on many ranks, try compiling with the parallel-hdf5 feature enabled")
    }
    let world = MPI_UNIVERSE.world();
    for i in 0..**world_size {
        if i < **rank as usize {
            world.barrier();
        }
    }
}

#[cfg(not(feature = "parallel-hdf5"))]
pub fn finish_wait_for_other_ranks_system(
    world_size: Res<crate::prelude::WorldSize>,
    rank: Res<WorldRank>,
) {
    let world = MPI_UNIVERSE.world();
    for i in 0..**world_size {
        if i >= **rank as usize {
            world.barrier();
        }
    }
}
