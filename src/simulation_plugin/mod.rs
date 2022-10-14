mod parameters;
mod time;

use bevy::app::AppExit;
use bevy::prelude::*;
use mpi::traits::Equivalence;

pub use self::parameters::SimulationParameters;
pub use self::time::Time;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::Communicator;
use crate::components::Mass;
use crate::components::Position;
use crate::components::Velocity;
use crate::gravity::gravity_system;
use crate::io::output::Attribute;
use crate::io::output::OutputPlugin;
use crate::named::Named;
use crate::particle::ParticlePlugin;
use crate::prelude::Particles;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units;

#[derive(Equivalence, Deref, DerefMut)]
pub struct Timestep(crate::units::Time);

#[derive(Named)]
pub struct SimulationPlugin;

// Cannot wait for stageless
#[derive(StageLabel)]
pub enum SimulationStages {
    Physics,
}

#[derive(Equivalence, Clone)]
pub(super) struct ShouldExit(bool);

impl RaxiomPlugin for SimulationPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let parameters = sim
            .add_parameter_type_and_get_result::<SimulationParameters>()
            .clone();
        sim.add_required_component::<Position>()
            .add_required_component::<Mass>()
            .add_required_component::<Velocity>()
            .add_plugin(ParticlePlugin)
            .add_plugin(OutputPlugin::<Attribute<Time>>::default())
            .add_plugin(CommunicationPlugin::<ShouldExit>::new(
                CommunicationType::AllGather,
            ))
            .add_event::<StopSimulationEvent>()
            .insert_resource(Timestep(parameters.timestep))
            .insert_resource(Time(units::Time::seconds(0.00)))
            .add_system_to_stage(
                SimulationStages::Physics,
                integrate_motion_system.after(gravity_system),
            )
            .add_system_to_stage(
                SimulationStages::Physics,
                show_time_system.before(time_system),
            )
            .add_system_to_stage(
                SimulationStages::Physics,
                time_system.after(integrate_motion_system),
            )
            .add_system_to_stage(
                SimulationStages::Physics,
                stop_simulation_system.after(time_system),
            )
            .add_system_to_stage(
                SimulationStages::Physics,
                handle_app_exit_system.after(stop_simulation_system),
            );
    }
}

pub struct StopSimulationEvent;

fn stop_simulation_system(
    parameters: Res<SimulationParameters>,
    current_time: Res<Time>,
    mut stop_sim: EventWriter<StopSimulationEvent>,
) {
    if let Some(time) = parameters.final_time {
        if **current_time >= time {
            stop_sim.send(StopSimulationEvent);
        }
    }
}

fn handle_app_exit_system(
    mut event_reader: EventReader<StopSimulationEvent>,
    mut event_writer: EventWriter<AppExit>,
    mut comm: Communicator<ShouldExit>,
) {
    let result = if event_reader.iter().count() > 0 {
        comm.all_gather(&ShouldExit(true))
    } else {
        comm.all_gather(&ShouldExit(false))
    };
    let should_exit = result.into_iter().any(|x| x.0);
    if should_exit {
        event_writer.send(AppExit);
    }
}

fn integrate_motion_system(
    mut query: Particles<(&mut Position, &Velocity)>,
    timestep: Res<Timestep>,
) {
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