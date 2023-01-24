mod parameters;
mod time;

#[cfg(all(not(feature = "2d"), test))]
mod tests;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::exit_on_all_closed;
use mpi::traits::Equivalence;

pub use self::parameters::SimulationParameters;
pub use self::parameters::TimestepParameters;
pub use self::time::Time;
use crate::communication::CommunicationPlugin;
use crate::communication::Communicator;
use crate::components::Mass;
use crate::components::Position;
use crate::components::Velocity;
use crate::io::output::Attribute;
use crate::io::output::OutputPlugin;
use crate::named::Named;
use crate::parameters::SimulationBox;
use crate::particle::ParticlePlugin;
use crate::prelude::Particles;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units;

#[derive(Named)]
pub struct SimulationPlugin;

// Cannot wait for stageless
#[derive(StageLabel)]
pub enum SimulationStages {
    ForceCalculation,
    Integration,
    SetTimestep,
}

#[derive(StageLabel)]
pub enum SimulationStartupStages {
    InsertComponents,
    InsertDerivedComponents,
}

#[derive(Equivalence, Clone)]
pub(super) struct ShouldExit(bool);

pub struct StopSimulationEvent;

impl RaxiomPlugin for SimulationPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<SimulationParameters>()
            .add_parameter_type::<TimestepParameters>()
            .add_parameter_type::<SimulationBox>()
            .add_required_component::<Position>()
            .add_required_component::<Mass>()
            .add_required_component::<Velocity>()
            .add_plugin(ParticlePlugin)
            .add_plugin(OutputPlugin::<Attribute<Time>>::default())
            .add_plugin(CommunicationPlugin::<ShouldExit>::default())
            .add_event::<StopSimulationEvent>()
            .insert_resource(Time(units::Time::seconds(0.00)))
            .add_system_to_stage(SimulationStages::Integration, integrate_motion_system)
            .add_system_to_stage(
                SimulationStages::Integration,
                show_time_system.before(time_system),
            )
            .add_system_to_stage(
                SimulationStages::Integration,
                time_system.after(integrate_motion_system),
            )
            .add_system_to_stage(CoreStage::PostUpdate, stop_simulation_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                handle_app_exit_system
                    .after(stop_simulation_system)
                    .before(exit_on_all_closed),
            );
    }
}

pub fn stop_simulation_system(
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

pub fn integrate_motion_system(
    mut query: Particles<(&mut Position, &Velocity)>,
    parameters: Res<TimestepParameters>,
    box_: Res<SimulationBox>,
) {
    for (mut pos, velocity) in query.iter_mut() {
        **pos += **velocity * parameters.max_timestep;
        **pos = box_.periodic_wrap(**pos);
    }
}

fn time_system(mut time: ResMut<Time>, parameters: Res<TimestepParameters>) {
    **time += parameters.max_timestep;
}

pub fn show_time_system(time: Res<self::Time>) {
    debug!("Time: {:?}", **time);
}
