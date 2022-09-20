mod gravity;
pub(super) mod hydrodynamics;
mod parameters;
mod time;

use bevy::prelude::*;
use mpi::traits::Equivalence;

use self::gravity::gravity_system;
pub use self::gravity::mass_moments::MassMoments;
pub use self::gravity::plugin::GravityPlugin;
pub use self::hydrodynamics::HydrodynamicsPlugin;
use self::parameters::Parameters;
pub use self::time::Time;
use crate::domain::ExchangeDataPlugin;
use crate::io::input::DatasetInputPlugin;
use crate::io::output::AttributeOutputPlugin;
use crate::io::output::DatasetOutputPlugin;
use crate::mass::Mass;
use crate::named::Named;
use crate::position::Position;
use crate::simulation::Simulation;
use crate::simulation::TenetPlugin;
use crate::units;
use crate::velocity::Velocity;

#[derive(Component)]
pub struct LocalParticle;

#[derive(Equivalence, Deref, DerefMut)]
pub struct Timestep(crate::units::Time);

#[derive(Named)]
pub struct PhysicsPlugin;

// Cannot wait for stageless
#[derive(StageLabel)]
pub enum PhysicsStages {
    Physics,
}

impl TenetPlugin for PhysicsPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let parameters = sim.add_parameter_type_and_get_result::<Parameters>("physics");
        sim.add_plugin(ExchangeDataPlugin::<Position>::default())
            .add_plugin(ExchangeDataPlugin::<Velocity>::default())
            .add_plugin(ExchangeDataPlugin::<Mass>::default())
            .add_plugin(DatasetOutputPlugin::<Position>::default())
            .add_plugin(DatasetOutputPlugin::<Velocity>::default())
            .add_plugin(DatasetOutputPlugin::<Mass>::default())
            .add_plugin(DatasetInputPlugin::<Position>::default())
            .add_plugin(DatasetInputPlugin::<Velocity>::default())
            .add_plugin(DatasetInputPlugin::<Mass>::default())
            .add_plugin(AttributeOutputPlugin::<Time>::default())
            .add_event::<StopSimulationEvent>()
            .insert_resource(Timestep(parameters.timestep))
            .insert_resource(Time(units::Time::seconds(0.00)))
            .add_system_to_stage(
                PhysicsStages::Physics,
                integrate_motion_system.after(gravity_system),
            )
            .add_system_to_stage(PhysicsStages::Physics, show_time_system.before(time_system))
            .add_system_to_stage(
                PhysicsStages::Physics,
                time_system.after(integrate_motion_system),
            )
            .add_system_to_stage(
                PhysicsStages::Physics,
                stop_simulation_system.after(time_system),
            );
    }
}

pub struct StopSimulationEvent;

fn stop_simulation_system(
    parameters: Res<Parameters>,
    current_time: Res<Time>,
    mut stop_sim: EventWriter<StopSimulationEvent>,
) {
    if let Some(time) = parameters.final_time {
        if **current_time >= time {
            stop_sim.send(StopSimulationEvent);
        }
    }
}

fn integrate_motion_system(mut query: Query<(&mut Position, &Velocity)>, timestep: Res<Timestep>) {
    for (mut pos, velocity) in query.iter_mut() {
        **pos += **velocity * timestep.0;
    }
}

pub fn time_system(mut time: ResMut<self::Time>, timestep: Res<Timestep>) {
    **time += **timestep;
}

pub fn show_time_system(time: Res<self::Time>) {
    debug!("Time: {:?}", **time);
}
