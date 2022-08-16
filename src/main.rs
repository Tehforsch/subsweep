#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params)]
// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]

mod args;
mod communication;
mod config;
mod mpi_world;
mod physics;
mod position;
pub mod units;
mod velocity;
mod visualization;

use args::CommandLineOptions;
use args::RunType;
use bevy::prelude::App;
use bevy::prelude::Commands;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use bevy::prelude::Res;
use clap::Parser;
use glam::Vec2;
use mpi::Rank;
use mpi_world::initialize_mpi_and_add_world_resource;
use mpi_world::MpiWorld;
use physics::Domain;
use physics::DomainDistribution;
use physics::PhysicsPlugin;
use position::Position;
use units::vec2::meter;
use units::vec2::meters_per_second;
use velocity::Velocity;
use visualization::VisualizationPlugin;

fn spawn_particles_system(mut commands: Commands, domain: Res<Domain>, world: Res<MpiWorld>) {
    if world.rank() != 0 {
        return;
    }
    for i in [0.5] {
        let pos = domain.upper_left + (domain.lower_right - domain.upper_left) * i;
        commands
            .spawn()
            .insert(Position(pos))
            .insert(Velocity(meters_per_second(Vec2::new(1.0, 0.0))));
    }
}

fn build_app(app: &mut App, rank: Rank) {
    let domain_distribution = get_domain_distribution();
    let domain = domain_distribution.domains[&rank].clone();
    if rank == 0 {
        app.add_plugins(DefaultPlugins)
            .add_plugin(VisualizationPlugin);
    } else {
        app.add_plugins(MinimalPlugins);
    }
    app.insert_resource(domain)
        .add_plugin(PhysicsPlugin(domain_distribution))
        .add_startup_system(spawn_particles_system);
}

fn main() {
    let opts = CommandLineOptions::parse();
    let mut app = App::new();
    let rank = match opts.run_type {
        RunType::Mpi => initialize_mpi_and_add_world_resource(&mut app),
        RunType::Local => todo!(),
    };
    build_app(&mut app, rank);
    app.run();
}

fn get_domain_distribution() -> DomainDistribution {
    DomainDistribution {
        domains: [
            (
                0,
                Domain {
                    upper_left: meter(Vec2::new(0.0, 0.0)),
                    lower_right: meter(Vec2::new(0.5, 0.5)),
                },
            ),
            (
                1,
                Domain {
                    upper_left: meter(Vec2::new(0.5, 0.0)),
                    lower_right: meter(Vec2::new(1.0, 0.5)),
                },
            ),
            (
                2,
                Domain {
                    upper_left: meter(Vec2::new(0.5, 0.5)),
                    lower_right: meter(Vec2::new(1.0, 1.0)),
                },
            ),
            (
                3,
                Domain {
                    upper_left: meter(Vec2::new(0.0, 0.5)),
                    lower_right: meter(Vec2::new(0.5, 1.0)),
                },
            ),
        ]
        .into_iter()
        .collect(),
    }
}
