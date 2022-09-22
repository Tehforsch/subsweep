#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params, hash_drain_filter)]
#![feature(const_fn_floating_point_arithmetic)]
// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]

pub mod command_line_options;
pub mod communication;
pub(crate) mod density;
pub(crate) mod domain;
pub(crate) mod io;
pub(crate) mod mass;
pub(crate) mod named;
pub(crate) mod parameters;
pub mod particle;
pub(crate) mod physics;
pub(crate) mod position;
pub(crate) mod pressure;
pub(crate) mod quadtree;
pub mod simulation;
pub mod simulation_builder;
pub(crate) mod stages;
pub mod units;
pub(crate) mod velocity;
pub(crate) mod visualization;

#[cfg(feature = "mpi")]
pub mod mpi_log;

pub mod prelude {
    pub use super::communication::WorldRank;
    pub use super::communication::WorldSize;
    pub use super::mass::Mass;
    pub use super::named::*;
    pub use super::physics::hydrodynamics::HydrodynamicsPlugin;
    pub use super::physics::GravityPlugin;
    pub use super::physics::LocalParticle;
    pub use super::physics::Timestep;
    pub use super::position::Position;
    pub use super::simulation_builder::SimulationBuilder;
    pub use super::velocity::Velocity;
    pub use super::visualization::parameters::VisualizationParameters;
    pub use super::visualization::CameraTransform;
    pub use super::visualization::DrawCircle;
    pub use super::visualization::DrawRect;
}
