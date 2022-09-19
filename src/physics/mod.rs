mod gravity;
mod parameters;
mod time;

use bevy::app::AppExit;
use bevy::prelude::*;
use mpi::traits::Equivalence;

use self::gravity::gravity_system;
pub use self::gravity::mass_moments::MassMoments;
pub use self::gravity::plugin::GravityPlugin;
use self::parameters::Parameters;
pub use self::time::Time;
use crate::domain::ExchangeDataPlugin;
use crate::io::input::DatasetInputPlugin;
use crate::io::output::AttributeOutputPlugin;
use crate::io::output::DatasetOutputPlugin;
use crate::mass::Mass;
use crate::named::Named;
use crate::parameters::ParameterPlugin;
use crate::plugin_utils::get_parameters;
use crate::plugin_utils::panic_if_already_added;
use crate::position::Position;
use crate::quadtree::QuadTreeConfig;
use crate::units;
use crate::velocity::Velocity;

#[derive(Component)]
pub struct LocalParticle;

#[derive(Equivalence, Deref, DerefMut)]
pub struct Timestep(crate::units::Time);

pub struct PhysicsPlugin;

impl Named for PhysicsPlugin {
    fn name() -> &'static str {
        "physics_plugin"
    }
}

// Cannot wait for stageless
#[derive(StageLabel)]
pub enum PhysicsStages {
    Physics,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        panic_if_already_added::<Self>(app);
        app.add_plugin(ParameterPlugin::<Parameters>::new("physics"))
            .add_plugin(ParameterPlugin::<QuadTreeConfig>::new("tree"));
        let parameters = get_parameters::<Parameters>(app).clone();
        app.add_plugin(ExchangeDataPlugin::<Position>::default())
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
