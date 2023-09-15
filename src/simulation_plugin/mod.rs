mod parameters;
mod time;

use bevy_app::AppExit;
use bevy_ecs::prelude::*;
use log::info;
use mpi::traits::Equivalence;

pub use self::parameters::SimulationParameters;
pub use self::time::SimulationTime;
use crate::components::Position;
use crate::io::output::Attribute;
use crate::io::output::OutputPlugin;
use crate::named::Named;
use crate::parameters::SimulationBox;
use crate::particle::ParticlePlugin;
use crate::performance_data::write_performance_data_system;
use crate::performance_data::Timers;
use crate::prelude::Particles;
use crate::prelude::WorldSize;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::simulation_box::SimulationBoxPlugin;
use crate::units;

#[derive(Named)]
pub struct SimulationPlugin;

#[derive(StageLabel)]
pub enum Stages {
    Initial,
    Sweep,
    AfterSweep,
    Output,
    Final,
}

#[derive(StageLabel)]
pub enum StartupStages {
    Initial,
    ReadInput,
    InsertDerivedComponents,
    Decomposition,
    SetOutgoingEntities,
    Exchange,
    AssignParticleIds,
    TreeConstruction,
    Remap,
    InsertGrid,
    InsertComponentsAfterGrid,
    InitSweep,
    Final,
}

#[derive(Equivalence, Clone)]
pub(super) struct ShouldExit(bool);

pub struct StopSimulationEvent;

impl RaxiomPlugin for SimulationPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let mut perf = Timers::default();
        perf.start("total");
        sim.insert_non_send_resource(perf)
            .add_parameter_type::<SimulationParameters>()
            .add_required_component::<Position>()
            .add_plugin(SimulationBoxPlugin)
            .add_plugin(ParticlePlugin)
            .add_plugin(OutputPlugin::<Attribute<SimulationTime>>::default())
            .add_event::<StopSimulationEvent>()
            .insert_resource(SimulationTime(units::Time::seconds(0.00)))
            .add_startup_system_to_stage(
                StartupStages::ReadInput,
                check_particles_in_simulation_box_system,
            )
            .add_startup_system_to_stage(StartupStages::ReadInput, show_num_cores_system)
            .add_system_to_stage(Stages::Output, write_performance_data_system)
            .add_system_to_stage(Stages::Initial, show_time_system)
            .add_system_to_stage(Stages::Final, exit_system)
            .add_system_to_stage(Stages::Initial, stop_simulation_system);
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

fn stop_simulation_system(
    parameters: Res<SimulationParameters>,
    current_time: Res<SimulationTime>,
    mut stop_sim: EventWriter<StopSimulationEvent>,
) {
    if let Some(time) = parameters.final_time {
        if **current_time >= time {
            stop_sim.send(StopSimulationEvent);
        }
    }
}

fn show_time_system(time: Res<SimulationTime>) {
    info!("Time: {:?}", **time);
}

fn exit_system(
    mut evs: EventWriter<AppExit>,
    mut stop_sim: EventReader<StopSimulationEvent>,
    mut timers: NonSendMut<Timers>,
) {
    if stop_sim.iter().count() > 0 {
        timers.stop("total");
        let time_in_secs = timers.total("total").in_seconds();
        info!("Run finished after {:.03} seconds.", time_in_secs);
        evs.send(AppExit);
    }
}

fn show_num_cores_system(world_size: Res<WorldSize>, mut performance_data: ResMut<Timers>) {
    performance_data.record_number("num_ranks", **world_size);
    info!("Running on {} MPI ranks", **world_size);
}

pub fn remove_components_system<C: Component>(
    mut commands: Commands,
    particles: Particles<Entity, With<C>>,
) {
    for entity in particles.iter() {
        commands.entity(entity).remove::<C>();
    }
}
