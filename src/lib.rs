#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params)]
#![feature(const_fn_floating_point_arithmetic)]
// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]
// This can sometimes make code less clear in my opinion.
#![allow(clippy::collapsible_else_if)]
// These are sometimes caused by automatically generated in the
// MPI Equivalence derive.
#![allow(clippy::unneeded_wildcard_pattern)]
#![allow(clippy::new_without_default)]
#![doc = include_str!("../README.md")]

mod chemistry;
mod command_line_options;
pub mod communication;
pub mod components;
pub mod cosmology;
pub mod dimension;
pub mod domain;
mod extent;
pub mod hash_map;
pub mod io;
/// Debug printing utilities for MPI simulations
pub mod mpi_log;
mod parameter_plugin;
/// Contains all the parameter types of the simulation.
pub mod parameters;
mod particle;
mod peano_hilbert;
mod performance_data;
pub mod prelude;
pub mod quadtree;
mod simulation;
mod simulation_box;
mod simulation_builder;
pub mod simulation_plugin;
pub mod source_systems;
pub mod stages;
pub mod sweep;
/// Compile-time units and quantities for the simulation.
pub mod units;
pub mod voronoi;

mod named {
    pub use derive_custom::Named;
    pub use derive_traits::Named;
}

#[cfg(test)]
pub(crate) mod test_examples;
#[cfg(test)]
pub(crate) mod test_utils;
