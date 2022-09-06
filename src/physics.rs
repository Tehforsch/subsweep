mod gravity;
mod parameters;

use bevy::prelude::*;
pub use gravity::QuadTree;
use hdf5::H5Type;
use mpi::traits::Equivalence;

use self::gravity::construct_quad_tree_system;
use self::gravity::gravity_system;
use self::parameters::Parameters;
use crate::communication::Rank;
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

#[derive(Component)]
pub struct RemoteParticle(pub Rank);

#[derive(Equivalence)]
pub(super) struct Timestep(crate::units::Time);

#[derive(H5Type, Clone)]
#[repr(C)]
pub struct Time(pub crate::units::Time);

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
            .add_system_to_stage(
                PhysicsStages::QuadTreeConstruction,
                construct_quad_tree_system,
            )
            .add_system_to_stage(PhysicsStages::Gravity, gravity_system)
            .add_system(integrate_motion_system)
            .add_system(time_system);
    }
}

fn integrate_motion_system(mut query: Query<(&mut Position, &Velocity)>, timestep: Res<Timestep>) {
    for (mut pos, velocity) in query.iter_mut() {
        pos.0 += velocity.0 * timestep.0;
    }
}

pub(super) fn time_system(mut time: ResMut<self::Time>, timestep: Res<Timestep>) {
    time.0 += timestep.0;
}
