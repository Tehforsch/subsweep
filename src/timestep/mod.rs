mod active_particles;
mod constant_timestep;
mod parameters;
mod time_bins;

use std::marker::PhantomData;

pub use active_particles::ActiveParticles;
use bevy::ecs::query::QueryItem;
use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::ecs::query::WorldQuery;
use bevy::prelude::Commands;
use bevy::prelude::CoreStage;
use bevy::prelude::Entity;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::StartupStage;
use bevy::prelude::Without;

pub use self::constant_timestep::ConstantTimestep;
pub use self::parameters::TimestepParameters;
use self::time_bins::TimeBins;
use crate::components::Timestep;
use crate::named::Named;
use crate::prelude::Float;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;
use crate::units::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub struct TimestepState {
    /// The currently active timestep. Level 0
    /// is the highest possible timestep T_0 and level i
    /// corresponds to the timestep T_i = T_0 2^{-i}
    count: usize,
    max_num_bins: usize,
}

impl TimestepState {
    fn new(max_num_bins: usize) -> Self {
        Self {
            max_num_bins,
            count: 0,
        }
    }

    fn next(self) -> Self {
        let max_count = 2usize.pow(self.max_num_bins as u32 - 1);
        Self {
            count: (self.count + 1).rem_euclid(max_count),
            max_num_bins: self.max_num_bins,
        }
    }

    fn is_active_bin(&self, level: usize) -> bool {
        self.count
            .rem_euclid(2usize.pow((self.max_num_bins - 1 - level) as u32))
            == 0
    }

    pub fn on_synchronization_step(&self) -> bool {
        self.count == 0
    }
}

pub trait TimestepCriterion: Sync + Send {
    type Query: ReadOnlyWorldQuery + WorldQuery;
    type Filter: ReadOnlyWorldQuery + WorldQuery;
    fn timestep(parameters: &TimestepParameters, query_item: QueryItem<Self::Query>) -> Time;
}

#[derive(Named)]
pub struct TimestepPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for TimestepPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: TimestepCriterion + 'static> RaxiomPlugin for TimestepPlugin<T> {
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn build_once_everywhere(&self, sim: &mut Simulation) {
        let parameters = sim
            .add_derived_component::<Timestep>()
            .add_parameter_type_and_get_result::<TimestepParameters>()
            .clone();
        sim.insert_resource(TimestepState::new(parameters.num_levels))
            .add_system_to_stage(CoreStage::PostUpdate, timestep_transition_system);
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        let num_levels = sim.unwrap_resource::<TimestepParameters>().num_levels;
        sim.add_startup_system_to_stage(
            StartupStage::PostStartup,
            add_timestep_component_system::<T>,
        )
        .add_system_to_stage(CoreStage::PreUpdate, determine_timesteps_system::<T>)
        .insert_resource(TimeBins::<T>::new(num_levels));
    }
}

fn timestep_transition_system(mut state: ResMut<TimestepState>) {
    let new_state = state.next();
    *state = new_state;
}

fn add_timestep_component_system<T: TimestepCriterion + 'static>(
    mut commands: Commands,
    particles: Particles<(Entity, T::Query), (T::Filter, Without<Timestep>)>,
) {
    for (entity, _) in particles.iter() {
        commands.entity(entity).insert(Timestep(Time::zero()));
    }
}

fn determine_timesteps_system<T: TimestepCriterion + 'static>(
    parameters: Res<TimestepParameters>,
    mut bins: ResMut<TimeBins<T>>,
    mut particles: Particles<(Entity, &mut Timestep, T::Query), T::Filter>,
) {
    bins.reset();
    for (entity, mut timestep, data) in particles.iter_mut() {
        let desired_timestep = T::timestep(&parameters, data);
        let timestep_ratio = (parameters.max_timestep / desired_timestep).value();
        // bin = log2(T_0 / T) clamped to [0, num_levels)
        let bin_level = timestep_ratio
            .log2()
            .clamp(0.0, (parameters.num_levels - 1) as Float) as usize;
        **timestep = parameters.max_timestep / (2u32.pow(bin_level as u32) as Float);
        bins.insert(bin_level, entity);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Commands;
    use bevy::prelude::Component;
    use bevy::prelude::IntoSystemDescriptor;

    use super::parameters::TimestepParameters;
    use super::TimestepCriterion;
    use super::TimestepPlugin;
    use crate::components::Timestep;
    use crate::prelude::Float;
    use crate::prelude::LocalParticle;
    use crate::prelude::Particles;
    use crate::prelude::Simulation;
    use crate::test_utils::assert_is_close;
    use crate::test_utils::run_system_on_sim;
    use crate::timestep::active_particles::ActiveParticles;
    use crate::timestep::add_timestep_component_system;
    use crate::units::Time;

    #[derive(Component)]
    struct DesiredTimestep(Time);

    #[derive(Component, Default)]
    struct Counter {
        counter: usize,
        total_time: Time,
    }

    struct DumbCriterion;
    impl TimestepCriterion for DumbCriterion {
        type Filter = ();

        type Query = &'static DesiredTimestep;

        fn timestep(_parameters: &TimestepParameters, query_item: &DesiredTimestep) -> Time {
            query_item.0
        }
    }

    #[test]
    fn check_timestepping() {
        let mut sim = Simulation::test();
        const BASE_TIMESTEP: Time = Time::seconds(1.0);
        fn spawn_particles_system(mut commands: Commands) {
            let spawn = |commands: &mut Commands, factor: usize| {
                // Add an epsilon to make sure we slip into the correct bin
                let epsilon = Time::seconds(1e-5);
                let timestep = BASE_TIMESTEP / (factor as Float) - epsilon;
                commands.spawn((DesiredTimestep(timestep), Counter::default(), LocalParticle));
            };
            spawn(&mut commands, 1);
            spawn(&mut commands, 2);
            spawn(&mut commands, 4);
            spawn(&mut commands, 8);
        }
        fn count_timesteps_system(
            mut particles: ActiveParticles<DumbCriterion, (&mut Counter, &Timestep)>,
        ) {
            for (mut counter, timestep) in particles.iter_mut() {
                counter.counter += 1;
                counter.total_time += **timestep;
            }
        }
        fn check_counters_system(particles: Particles<(&Counter, &DesiredTimestep)>) {
            for (counter, timestep) in particles.iter() {
                let desired_num_updates = (BASE_TIMESTEP / timestep.0).value() as usize;
                assert_eq!(desired_num_updates, counter.counter);
                assert_is_close(counter.total_time, BASE_TIMESTEP);
            }
        }
        sim.add_parameter_file_contents("".into());
        sim.add_parameters_explicitly(TimestepParameters {
            num_levels: 4,
            max_timestep: Time::seconds(1.0),
        });
        sim.add_plugin(TimestepPlugin::<DumbCriterion>::default());
        sim.add_startup_system(
            spawn_particles_system.before(add_timestep_component_system::<DumbCriterion>),
        );
        sim.add_system(count_timesteps_system);
        // Run one full timestep
        sim.timestep();
        run_system_on_sim(&mut sim, check_counters_system);
    }
}
