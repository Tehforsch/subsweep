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
pub(crate) mod parameter_plugin;
pub mod particle;
pub(crate) mod performance_parameters;
pub(crate) mod physics;
pub(crate) mod position;
pub(crate) mod pressure;
pub(crate) mod quadtree;
pub(crate) mod simulation;
pub(crate) mod simulation_builder;
pub(crate) mod stages;
pub mod units;
pub(crate) mod velocity;
pub(crate) mod visualization;

#[cfg(feature = "mpi")]
pub mod mpi_log;

pub mod parameters;
pub mod prelude;
