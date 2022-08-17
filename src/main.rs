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
mod physics;
mod position;
pub mod units;
mod velocity;
mod visualization;

use bevy::prelude::App;
use bevy::prelude::Commands;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use bevy::prelude::Res;
use communication::Communicator;
use communication::ExchangeCommunicator;
use communication::SizedCommunicator;
use domain::Domain;
use domain::DomainDistribution;
use glam::Vec2;
use mpi::Rank;
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

fn build_and_run_app(communicator: Communicator<ParticleExchangeData>) {
    let mut app = App::new();
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
        &mut app,
        domain_distribution,
        ExchangeCommunicator::new(communicator),
    );
    app.insert_resource(domain)
        .insert_resource(rank)
        .add_startup_system(spawn_particles_system);
    app.run();
}

#[cfg(feature = "local")]
fn main() {
    use std::iter::once;
    use std::thread;

    use args::CommandLineOptions;
    use clap::Parser;
    use communication::get_local_communicators;

    let opts = CommandLineOptions::parse();
    let mut communicators = get_local_communicators(opts.num_threads);
    for rank in (1..opts.num_threads).chain(once(0)) {
        let communicator = communicators.remove(&(rank as Rank)).unwrap();
        if rank == 0 {
            build_and_run_app(communicator);
        } else {
            thread::spawn(move || {
                build_and_run_app(communicator);
            });
        }
    }
}

#[cfg(not(feature = "local"))]
fn main() {
    let (_universe, world) = Communicator::<ParticleExchangeData>::initialize();
    build_and_run_app(world);
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
