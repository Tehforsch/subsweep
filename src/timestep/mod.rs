mod parameters;
mod time_bins;

use std::marker::PhantomData;

use bevy::ecs::query::ROQueryItem;
use bevy::ecs::query::WorldQuery;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;

use self::parameters::TimestepParameters;
use self::time_bins::TimeBins;
use crate::named::Named;
use crate::prelude::Float;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;
use crate::units::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ActiveTimestep {
    /// The currently active timestep. Level 0
    /// is the highest possible timestep T_0 and level i
    /// corresponds to the timestep T_i = T_0 2^{-i}
    level: usize,
}

trait TimestepCriterion: Sync + Send {
    type Query: WorldQuery;
    type Filter: WorldQuery;
    fn timestep(query_item: ROQueryItem<Self::Query>) -> Time;
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
        sim.add_parameter_type::<TimestepParameters>()
            .add_state(ActiveTimestep::default());
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_system(determine_timesteps_system::<T>);
    }
}

fn determine_timesteps_system<T: TimestepCriterion + 'static>(
    parameters: Res<TimestepParameters>,
    mut bins: ResMut<TimeBins<T>>,
    particles: Particles<(Entity, T::Query), T::Filter>,
) {
    bins.reset();
    for (entity, data) in particles.iter() {
        let timestep = T::timestep(data);
        let timestep_ratio = (timestep / parameters.max_timestep).value();
        let bin_level = timestep_ratio
            .log2()
            .max(0.0)
            .min((parameters.num_levels - 1) as Float) as usize;
        bins.insert_up_to(bin_level, entity);
    }
}

pub fn on_synchronization_step(step: Res<ActiveTimestep>) -> ShouldRun {
    match step.level {
        0 => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Component;

    use super::TimestepCriterion;
    use super::TimestepPlugin;
    use crate::prelude::Simulation;
    use crate::units::Time;

    #[derive(Component)]
    struct DesiredTimestep(Time);

    struct DumbCriterion;
    impl TimestepCriterion for DumbCriterion {
        type Filter = ();

        type Query = &'static DesiredTimestep;

        fn timestep(query_item: &DesiredTimestep) -> Time {
            query_item.0
        }
    }

    #[test]
    fn check_timestepping() {
        let mut sim = Simulation::default();
        sim.add_plugin(TimestepPlugin::<DumbCriterion>::default());
    }
}
