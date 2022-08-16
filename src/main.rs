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
mod domain;
mod mpi_world;
mod physics;
mod position;
pub mod units;
mod velocity;
mod visualization;

use std::thread;

use args::CommandLineOptions;
use args::RunType;
use bevy::prelude::App;
use bevy::prelude::Commands;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use bevy::prelude::Res;
use clap::Parser;
use communication::Communicator;
use communication::ExchangeCommunicator;
use domain::Domain;
use domain::DomainDistribution;
use glam::Vec2;
use mpi::Rank;
use mpi_world::MpiWorld;
use physics::ParticleExchangeData;
use physics::PhysicsPlugin;
use position::Position;
use units::vec2::meter;
use units::vec2::meters_per_second;
use velocity::Velocity;
use visualization::VisualizationPlugin;

fn spawn_particles_system(mut commands: Commands, domain: Res<Domain>, rank: Res<Rank>) {
    if *rank != 0 {
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

fn build_and_run_app<C: Communicator<ParticleExchangeData> + Clone + 'static>(
    app: &mut App,
    communicator: C,
) {
    let rank = communicator.rank();
    let domain_distribution = get_domain_distribution();
    let domain = domain_distribution.domains[&rank].clone();
    if rank == 0 {
        app.add_plugins(DefaultPlugins)
            .add_plugin(VisualizationPlugin);
    } else {
        app.add_plugins(MinimalPlugins);
    }
    PhysicsPlugin::add_to_app(
        app,
        domain_distribution,
        ExchangeCommunicator::new(communicator),
    );
    app.insert_resource(domain)
        .insert_resource(rank)
        .add_startup_system(spawn_particles_system);
    app.run();
}

fn main() {
    let opts = CommandLineOptions::parse();
    match opts.run_type {
        RunType::Mpi => {
            let (_universe, world) = MpiWorld::initialize();
            let mut app = App::new();
            build_and_run_app(&mut app, world);
        }
        RunType::Local(op) => {
            for rank in 1..op.num_threads {
                thread::spawn(move || {
                    let mut app = App::new();
                    todo!()
                    // build_and_run_app(&mut app, todo!());
                });
            }
            let mut app = App::new();
            todo!()
            // build_and_run_app(&mut app, todo!());
        }
    };
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
