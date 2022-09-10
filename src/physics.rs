mod gravity;
mod parameters;
mod time;

use bevy::prelude::*;
use mpi::traits::Equivalence;

pub use self::gravity::mass_moments::MassMoments;
use self::parameters::Parameters;
pub use self::time::Time;
use crate::domain::quadtree::QuadTreeConfig;
use crate::domain::DomainDecompositionStages;
use crate::domain::ExchangeDataPlugin;
use crate::mass::Mass;
use crate::output::AttributePlugin;
use crate::output::DatasetPlugin;
use crate::parameters::ParameterPlugin;
use crate::position::Position;
use crate::units;
use crate::velocity::Velocity;

#[derive(Component)]
pub struct LocalParticle;

#[derive(Equivalence, Deref, DerefMut)]
pub(super) struct Timestep(crate::units::Time);

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
            .add_plugin(ParameterPlugin::<QuadTreeConfig>::new("tree"))
            .add_plugin(ExchangeDataPlugin::<Position>::default())
            .add_plugin(ExchangeDataPlugin::<Velocity>::default())
            .add_plugin(ExchangeDataPlugin::<Mass>::default())
            .add_plugin(DatasetPlugin::<Position>::new("position"))
            .add_plugin(DatasetPlugin::<Velocity>::new("velocity"))
            .add_plugin(DatasetPlugin::<Mass>::new("mass"))
            .add_plugin(AttributePlugin::<Time>::new("time"))
            .insert_resource(Timestep(units::Time::second(0.01)))
            .insert_resource(Time(units::Time::second(0.00)))
            .add_system(integrate_motion_system)
            .add_system(time_system);
    }
}

fn integrate_motion_system(mut query: Query<(&mut Position, &Velocity)>, timestep: Res<Timestep>) {
    for (mut pos, velocity) in query.iter_mut() {
        **pos += **velocity * timestep.0;
    }
}

pub(super) fn time_system(mut time: ResMut<self::Time>, timestep: Res<Timestep>) {
    **time += **timestep;
}
