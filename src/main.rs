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
mod mass;
mod particle;
mod physics;
mod position;
pub mod units;
mod velocity;
mod visualization;

use args::CommandLineOptions;
use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::log::Level;
use bevy::log::LogPlugin;
use bevy::log::LogSettings;
use bevy::prelude::App;
use bevy::prelude::Commands;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::MinimalPlugins;
use bevy::prelude::Res;
use bevy::time::Time;
use communication::Communicator;
use communication::ExchangeCommunicator;
use communication::Identified;
use communication::SizedCommunicator;
use communication::SyncCommunicator;
use domain::Domain;
use domain::DomainDistribution;
use glam::Vec2;
use mass::Mass;
use mpi::Rank;
use particle::LocalParticleBundle;
use physics::ParticleExchangeData;
use physics::PhysicsPlugin;
use position::Position;
use units::f32::kilograms;
use units::vec2::meter;
use units::vec2::meters_per_second;
use velocity::Velocity;
use visualization::remote::ParticleVisualizationExchangeData;
use visualization::VisualizationPlugin;

fn spawn_particles_system(mut commands: Commands, domain: Res<Domain>, rank: Res<Rank>) {
    if *rank != 0 {
        return;
    }
    for i in [0.5] {
        let pos = domain.upper_left + (domain.lower_right - domain.upper_left) * i;
        commands.spawn().insert_bundle(LocalParticleBundle::new(
            Position(pos),
            Velocity(meters_per_second(Vec2::new(1.0, 0.0))),
            Mass(kilograms(1.0)),
        ));
    }
}

fn log_setup(verbosity: usize) -> LogSettings {
    match verbosity {
        0 => LogSettings {
            level: Level::INFO,
            ..Default::default()
        },
        1 => LogSettings {
            level: Level::DEBUG,
            filter: "bevy_ecs::world=info,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error,symphonia_format_ogg=error,symphonia_core=error".to_string(),
        },
        2 => LogSettings {
            level: Level::DEBUG,
            filter: "bevy_ecs::world=debug,bevy_app::plugin_group=info,bevy_app::app=info,winit=error,bevy_render=error,naga=error,wgpu=error,symphonia_format_ogg=error,symphonia_core=error".to_string(),
        },
        3 => LogSettings {
            level: Level::DEBUG,
            ..Default::default()
        },
        4 => LogSettings {
            level: Level::TRACE,
            ..Default::default()
        },
        v => unimplemented!("Unsupported verbosity level: {}", v)
    }
}

fn build_and_run_app(
    opts: &CommandLineOptions,
    communicator1: Communicator<ParticleExchangeData>,
    communicator2: Communicator<Identified<ParticleVisualizationExchangeData>>,
) {
    let mut app = App::new();
    let rank = communicator1.rank();
    let domain_distribution = get_domain_distribution();
    let domain = domain_distribution.domains[&rank].clone();
    if rank == 0 {
        app.insert_resource(log_setup(opts.verbosity));
        if opts.visualize {
            app.add_plugins(DefaultPlugins);
        } else {
            app.add_plugins(MinimalPlugins).add_plugin(LogPlugin);
        }
    } else {
        app.add_plugins(MinimalPlugins);
    }
    if opts.visualize {
        app.add_plugin(VisualizationPlugin {
            main_rank: rank == 0,
        });
    } else {
        // Only show execution order ambiguities when running without render plugins
        app.insert_resource(ReportExecutionOrderAmbiguities);
    }
    app.add_plugin(PhysicsPlugin(get_domain_distribution()))
        .insert_resource(domain)
        .insert_non_send_resource(ExchangeCommunicator::new(communicator1))
        .insert_non_send_resource(SyncCommunicator::new(communicator2))
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
    let mut communicators1 = get_local_communicators(opts.num_threads);
    let mut communicators2 = get_local_communicators(opts.num_threads);
    for rank in (1..opts.num_threads).chain(once(0)) {
        let communicator1 = communicators1.remove(&(rank as Rank)).unwrap();
        let communicator2 = communicators2.remove(&(rank as Rank)).unwrap();
        if rank == 0 {
            build_and_run_app(&opts, communicator1, communicator2);
        } else {
            thread::spawn(move || {
                build_and_run_app(&opts, communicator1, communicator2);
            });
        }
    }
}

#[cfg(not(feature = "local"))]
fn main() {
    use clap::Parser;

    let opts = CommandLineOptions::parse();
    let (_universe, world) = Communicator::<ParticleExchangeData>::initialize();
    let world2 = world.clone_for_different_type();
    build_and_run_app(&opts, world, world2);
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
