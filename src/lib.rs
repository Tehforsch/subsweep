#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params)]
#![feature(const_fn_floating_point_arithmetic)]
// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]

pub(crate) mod command_line_options;
#[cfg(feature = "mpi_test")]
pub mod communication;
#[cfg(not(feature = "mpi_test"))]
pub(crate) mod communication;
pub mod components;
pub(crate) mod config;
pub(crate) mod domain;
pub(crate) mod gravity;
pub mod hydrodynamics;
pub mod ics;
pub(crate) mod io;
pub(crate) mod memory;
pub(crate) mod parameter_plugin;
pub(crate) mod particle;
pub(crate) mod performance_parameters;
pub mod quadtree;
pub(crate) mod rand;
pub(crate) mod simulation;
pub(crate) mod simulation_box;
pub(crate) mod simulation_builder;
pub(crate) mod simulation_plugin;
pub(crate) mod stages;
pub(crate) mod timestep;
pub(crate) mod visualization;

pub mod named {
    pub use derive_custom::Named;
    pub use derive_traits::Named;
}

#[cfg(test)]
pub(crate) mod test_examples;
#[cfg(test)]
pub(crate) mod test_utils;

/// Debug printing utilities for MPI simulations
#[cfg(feature = "mpi")]
pub mod mpi_log;
/// Compile-time units and quantities for the simulation.
pub mod units;

/// Contains all the parameter types of the simulation.
pub mod parameters;
/// `use raxiom::prelude::*` to import some commonly used
/// plugins and components when building a simulation.
pub mod prelude;
