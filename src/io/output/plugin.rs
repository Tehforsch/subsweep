use bevy_ecs::prelude::*;
use bevy_ecs::schedule::SystemDescriptor;
use log::error;

use super::close_file_system;
use super::create_file_system;
use super::finish_wait_for_other_ranks_system;
use super::init_wait_for_other_ranks_system;
use super::make_output_dirs_system;
use super::open_file_system;
use super::parameters::is_desired_field;
use super::parameters::Fields;
use super::parameters::OutputParameters;
use super::timer::Timer;
use super::write_used_parameters_system;
use super::OutputFiles;
use crate::communication::communicator::Communicator;
use crate::io::input::file_distribution::get_rank_output_assignment_for_rank;
use crate::io::DatasetDescriptor;
use crate::io::OutputDatasetDescriptor;
use crate::named::Named;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::Stages;
use crate::prelude::StartupStages;
use crate::prelude::WorldRank;
use crate::prelude::WorldSize;
use crate::simulation::SubsweepPlugin;

pub(crate) trait IntoOutputSystem {
    fn write_system() -> SystemDescriptor;
    fn create_system() -> SystemDescriptor;
}

#[derive(SystemLabel)]
struct OutputSystemLabel;

#[derive(Resource, Default)]
struct RegisteredFields(pub Vec<String>);

#[derive(Named)]
pub struct OutputPlugin<T> {
    descriptor: OutputDatasetDescriptor<T>,
}

impl<T: Named> Default for OutputPlugin<T> {
    fn default() -> Self {
        Self {
            descriptor: OutputDatasetDescriptor::<T>::new(DatasetDescriptor::default_for::<T>()),
        }
    }
}

impl<T> OutputPlugin<T> {
    pub fn from_descriptor(descriptor: DatasetDescriptor) -> Self {
        Self {
            descriptor: OutputDatasetDescriptor::<T>::new(descriptor),
        }
    }
}

impl<T: 'static> SubsweepPlugin for OutputPlugin<T>
where
    T: IntoOutputSystem + Named,
{
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        sim.write_output
    }

    fn build_once_everywhere(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<OutputParameters>()
            .insert_resource(OutputFiles::default())
            .add_startup_system_to_stage(
                StartupStages::Final,
                compute_output_rank_assignment_system,
            )
            .add_startup_system(Timer::initialize_system)
            .add_system_to_stage(
                Stages::Output,
                open_file_system.with_run_criteria(Timer::run_criterion),
            )
            .add_system_to_stage(
                Stages::Output,
                close_file_system
                    .after(open_file_system)
                    .with_run_criteria(Timer::run_criterion),
            )
            .add_system_to_stage(
                Stages::Output,
                Timer::update_system
                    .after(close_file_system)
                    .with_run_criteria(Timer::run_criterion),
            );
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.insert_non_send_resource::<OutputDatasetDescriptor<T>>(
            OutputDatasetDescriptor::<T>::new(self.descriptor.descriptor.clone()),
        );
        if is_desired_field::<T>(sim) {
            sim.add_system_to_stage(
                Stages::Output,
                init_wait_for_other_ranks_system.before(open_file_system),
            )
            .add_system_to_stage(
                Stages::Output,
                finish_wait_for_other_ranks_system.after(close_file_system),
            )
            .add_system_to_stage(
                Stages::Output,
                T::write_system()
                    .after(open_file_system)
                    .before(close_file_system)
                    .label(OutputSystemLabel)
                    .ambiguous_with(OutputSystemLabel),
            );
        }
    }

    fn build_once_on_main_rank(&self, sim: &mut Simulation) {
        sim.insert_resource(RegisteredFields::default());
        sim.add_startup_system(make_output_dirs_system)
            .add_startup_system(write_used_parameters_system.after(make_output_dirs_system))
            .add_startup_system(verify_output_fields_system)
            .add_system_to_stage(
                Stages::CreateOutputFiles,
                create_file_system.with_run_criteria(Timer::run_criterion),
            )
            .add_system_to_stage(
                Stages::CreateOutputFiles,
                close_file_system.with_run_criteria(Timer::run_criterion),
            );
    }

    fn build_on_main_rank(&self, sim: &mut Simulation) {
        sim.get_resource_mut::<RegisteredFields>()
            .unwrap()
            .0
            .push(T::name().into());
        if is_desired_field::<T>(sim) {
            sim.add_system_to_stage(
                Stages::CreateOutputFiles,
                T::create_system()
                    .after(create_file_system)
                    .before(close_file_system)
                    .label(OutputSystemLabel)
                    .ambiguous_with(OutputSystemLabel),
            );
        }
    }
}

fn verify_output_fields_system(
    parameters: Res<OutputParameters>,
    registered: Res<RegisteredFields>,
) {
    if let Fields::Some(ref fields) = parameters.fields {
        for field in fields.iter() {
            if !registered.0.contains(field) {
                error!("Unknown field specified: {}", field);
            }
        }
    }
}

fn compute_output_rank_assignment_system(
    mut commands: Commands,
    particles: Particles<()>,
    rank: Res<WorldRank>,
    world_size: Res<WorldSize>,
    parameters: Res<OutputParameters>,
) {
    let num_particles = particles.iter().count();
    let mut comm: Communicator<usize> = Communicator::new();
    let total_num_particles: usize = comm.all_gather_sum(&num_particles);
    let rank_assignment = get_rank_output_assignment_for_rank(
        total_num_particles,
        parameters.num_output_files,
        **world_size,
        **rank,
    );
    commands.insert_resource(rank_assignment);
}
