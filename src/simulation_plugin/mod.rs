mod parameters;
mod time;

use bevy::prelude::*;
use mpi::traits::Equivalence;

pub use self::parameters::SimulationParameters;
pub use self::parameters::TimestepParameters;
pub use self::time::Time;
use crate::components::Position;
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
    InsertGrid,
    InsertComponentsAfterGrid,
    Sweep,
    Final,
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
            .add_plugin(ParticlePlugin)
            .add_plugin(OutputPlugin::<Attribute<Time>>::default())
            .add_event::<StopSimulationEvent>()
            .insert_resource(Time(units::Time::seconds(0.00)))
            .add_startup_system_to_stage(
                SimulationStartupStages::InsertComponents,
                check_particles_in_simulation_box_system,
            )
            .add_system_to_stage(
                SimulationStages::Integration,
                show_time_system.after(time_system),
            )
            .add_system_to_stage(SimulationStages::Integration, time_system)
            .add_system_to_stage(CoreStage::PostUpdate, stop_simulation_system);
    }
}

fn check_particles_in_simulation_box_system(
    box_: Res<SimulationBox>,
    particles: Particles<&Position>,
) {
    for p in particles.iter() {
        assert!(
            box_.contains(p),
            "Found particle outside of simulation box: {:?}",
            p
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

pub fn time_system(mut time: ResMut<Time>, parameters: Res<TimestepParameters>) {
    **time += parameters.max_timestep;
}

pub fn show_time_system(time: Res<self::Time>) {
    info!("Time: {:?}", **time);
}
