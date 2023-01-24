use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::prelude::*;
use derive_custom::raxiom_parameters;
use mpi::traits::Equivalence;

use crate::communication::CommunicationPlugin;
use crate::communication::Communicator;
use crate::io::output::OutputStages;
use crate::named::Named;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;

#[raxiom_parameters("memory_usage")]
pub struct MemoryUsageParameters {
    /// Whether to compute and display memory usage.
    #[serde(default)]
    show: bool,
}

#[derive(Clone, Equivalence)]
struct Memory(usize);

#[derive(Default, Resource)]
struct MemoryUsage {
    by_component: HashMap<&'static str, usize>,
}

impl MemoryUsage {
    fn total(&self) -> Memory {
        Memory(self.by_component.values().sum())
    }
}

#[derive(Named)]
pub(super) struct ComponentMemoryUsagePlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for ComponentMemoryUsagePlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

#[derive(SystemLabel)]
struct ComponentMemoryUsageLabel;

impl<T: Named + Component> RaxiomPlugin for ComponentMemoryUsagePlugin<T> {
    fn build_always_once(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<MemoryUsageParameters>();
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        let params = sim.get_parameters::<MemoryUsageParameters>();
        params.show
    }

    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system_to_stage(
            OutputStages::Output,
            calculate_memory_usage_system::<T>
                .after(reset_memory_usage_system)
                .before(communicate_memory_usage_system)
                .label(ComponentMemoryUsageLabel)
                .ambiguous_with(ComponentMemoryUsageLabel),
        );
    }

    fn build_once_everywhere(&self, sim: &mut Simulation) {
        sim.insert_resource(MemoryUsage::default())
            .add_plugin(CommunicationPlugin::<Memory>::default())
            .add_system_to_stage(OutputStages::Output, reset_memory_usage_system)
            .add_system_to_stage(OutputStages::Output, communicate_memory_usage_system);
    }
}

fn calculate_memory_usage_system<T: Named + Component>(
    query: Particles<&T>,
    mut memory: ResMut<MemoryUsage>,
) {
    memory
        .by_component
        .insert(T::name(), query.iter().count() * std::mem::size_of::<T>());
}

fn reset_memory_usage_system(mut memory: ResMut<MemoryUsage>) {
    memory.by_component.clear();
}

fn communicate_memory_usage_system(memory: Res<MemoryUsage>, mut comm: Communicator<Memory>) {
    let total_memory_used_this_rank = memory.total();
    let total_memory_used: usize = comm
        .all_gather(&total_memory_used_this_rank)
        .into_iter()
        .map(|x| x.0)
        .sum();
    let total_memory_used_this_rank_megabytes = total_memory_used_this_rank.0 as f64 / 1e6;
    let total_memory_used_megabytes = total_memory_used as f64 / 1e6;
    info!(
        "Memory used for components:\n\tThis rank: {:.1} MB\n\tAll ranks: {:.1} MB",
        total_memory_used_this_rank_megabytes, total_memory_used_megabytes
    );
}
