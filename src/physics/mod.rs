mod gravity;
mod parameters;
mod time;

use bevy::app::AppExit;
use bevy::prelude::*;
use mpi::traits::Equivalence;

pub use self::gravity::mass_moments::MassMoments;
use self::gravity::plugin::GravityPlugin;
use self::parameters::Parameters;
pub use self::time::Time;
use crate::domain::DomainDecompositionStages;
use crate::domain::ExchangeDataPlugin;
use crate::mass::Mass;
use crate::output::AttributePlugin;
use crate::output::DatasetPlugin;
use crate::parameters::ParameterPlugin;
use crate::plugin_utils::get_parameters;
use crate::position::Position;
use crate::quadtree::QuadTreeConfig;
use crate::units;
use crate::velocity::Velocity;

#[derive(Component)]
pub struct LocalParticle;

#[derive(Equivalence, Deref, DerefMut)]
pub struct Timestep(crate::units::Time);

pub struct PhysicsPlugin;

// Cannot wait for stageless
#[derive(StageLabel)]
pub enum PhysicsStages {
    QuadTreeConstruction,
    Gravity,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            DomainDecompositionStages::Decomposition,
            PhysicsStages::QuadTreeConstruction,
            SystemStage::parallel(),
        );
        app.add_stage_after(
            PhysicsStages::QuadTreeConstruction,
            PhysicsStages::Gravity,
            SystemStage::parallel(),
        );
        app.add_plugin(ParameterPlugin::<Parameters>::new("physics"))
            .add_plugin(ParameterPlugin::<QuadTreeConfig>::new("tree"));
        let parameters = get_parameters::<Parameters>(app).clone();
        app.add_plugin(ExchangeDataPlugin::<Position>::default())
            .add_plugin(ExchangeDataPlugin::<Velocity>::default())
            .add_plugin(ExchangeDataPlugin::<Mass>::default())
            .add_plugin(DatasetPlugin::<Position>::new("position"))
            .add_plugin(DatasetPlugin::<Velocity>::new("velocity"))
            .add_plugin(DatasetPlugin::<Mass>::new("mass"))
            .add_plugin(AttributePlugin::<Time>::new("time"))
            .add_plugin(GravityPlugin)
            .add_event::<StopSimulationEvent>()
            .insert_resource(Timestep(parameters.timestep))
            .insert_resource(Time(units::Time::second(0.00)))
            .add_system(integrate_motion_system)
            .add_system(time_system)
            .add_system(stop_simulation_system.after(time_system));
    }
}

pub struct StopSimulationEvent;

fn stop_simulation_system(
    parameters: Res<Parameters>,
    current_time: Res<Time>,
    mut stop_sim: EventWriter<StopSimulationEvent>,
    mut app_exit: EventWriter<AppExit>,
) {
    if let Some(time) = parameters.final_time {
        if **current_time >= time {
            stop_sim.send(StopSimulationEvent);
            app_exit.send(AppExit);
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
