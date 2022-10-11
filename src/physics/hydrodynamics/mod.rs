pub(crate) mod hydro_components;
mod parameters;
mod quadtree;

use std::f64::consts::PI;

use bevy::prelude::*;

use self::hydro_components::SmoothingLength;
pub use self::parameters::HydrodynamicsParameters;
use self::quadtree::construct_quad_tree_system;
use self::quadtree::get_particles_in_radius;
use self::quadtree::QuadTree;
use super::Timestep;
use crate::components;
use crate::components::Mass;
use crate::components::Position;
use crate::components::Velocity;
use crate::domain::extent::Extent;
use crate::named::Named;
use crate::performance_parameters::PerformanceParameters;
use crate::prelude::LocalParticle;
use crate::prelude::Particles;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units::Density;
use crate::units::Length;
use crate::units::NumberDensity;
use crate::units::VecAcceleration;

const GAMMA: f64 = 5.0 / 3.0;

fn kernel(r: Length, h: Length) -> f64 {
    // Spline Kernel, Monaghan & Lattanzio 1985
    let ratio = *(r / h).value();
    if ratio < 0.5 {
        1.0 - 6.0 * ratio.powi(2) + 6.0 * ratio.powi(3)
    } else if ratio < 1.0 {
        2.0 * (1.0 - ratio).powi(3)
    } else {
        0.0
    }
}

#[cfg(feature = "2d")]
fn kernel_function(r: Length, h: Length) -> NumberDensity {
    80.0 / (7.0 * PI * h.squared()) * kernel(r, h)
}

#[cfg(not(feature = "2d"))]
fn kernel_function(r: Length, h: Length) -> NumberDensity {
    8.0 / (PI * h.cubed()) * kernel(r, h)
}

#[derive(StageLabel)]
pub enum HydrodynamicsStages {
    Hydrodynamics,
}

#[derive(Named)]
pub struct HydrodynamicsPlugin;

impl RaxiomPlugin for HydrodynamicsPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<HydrodynamicsParameters>()
            .insert_resource(QuadTree::make_empty_leaf_from_extent(Extent::default()))
            .add_system_to_stage(
                HydrodynamicsStages::Hydrodynamics,
                construct_quad_tree_system,
            )
            .add_system_to_stage(
                HydrodynamicsStages::Hydrodynamics,
                compute_pressure_and_density_system.after(construct_quad_tree_system),
            )
            .add_system_to_stage(
                HydrodynamicsStages::Hydrodynamics,
                compute_forces_system
                    .after(compute_pressure_and_density_system)
                    .after(construct_quad_tree_system),
            )
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                insert_pressure_and_density_system,
            )
            .add_derived_component::<components::Pressure>()
            .add_derived_component::<components::Density>();
    }
}

fn set_smoothing_lengths_system(
    parameters: Res<HydrodynamicsParameters>,
    mut query: Particles<&mut SmoothingLength>,
) {
    for mut p in query.iter_mut() {
        **p = parameters.min_smoothing_length;
    }
}

fn insert_pressure_and_density_system(
    mut commands: Commands,
    particles: Particles<Entity, (Without<components::Pressure>, Without<components::Density>)>,
) {
    for entity in particles.iter() {
        commands.entity(entity).insert_bundle((
            components::Pressure::default(),
            components::Density::default(),
        ));
    }
}

fn compute_pressure_and_density_system(
    mut pressures: Query<
        (
            &mut components::Pressure,
            &mut components::Density,
            &SmoothingLength,
            &Position,
            &Mass,
        ),
        With<LocalParticle>,
    >,
    masses: Query<&Mass, With<LocalParticle>>,
    tree: Res<QuadTree>,
    performance_parameters: Res<PerformanceParameters>,
) {
    pressures.par_for_each_mut(
        performance_parameters.batch_size(),
        |(mut pressure, mut density, smoothing_length, pos, mass)| {
            **density = Density::zero();
            for particle in get_particles_in_radius(&tree, pos, &smoothing_length).iter() {
                let mass2 = masses.get(particle.entity).unwrap();
                let distance = particle.pos.distance(pos);
                todo!("{:?} {:?} {:?}", mass2, distance, density)
                // **density += **mass2 * kernel2d(distance, **smoothing_length);
                // // P = A * rho^gamma
                // **pressure = **entropy * density.powi::<-3>() * density.powi::<5>();
            }
        },
    );
}

fn compute_forces_system(
    mut particles1: Query<
        (
            Entity,
            &mut Velocity,
            &Position,
            &SmoothingLength,
            &components::Pressure,
            &components::Density,
        ),
        With<LocalParticle>,
    >,
    particles2: Query<
        (
            Entity,
            &Position,
            &components::Pressure,
            &components::Density,
            &components::Mass,
        ),
        With<LocalParticle>,
    >,
    tree: Res<QuadTree>,
    timestep: Res<Timestep>,
    parameters: Res<HydrodynamicsParameters>,
    performance_parameters: Res<PerformanceParameters>,
) {
    particles1.par_for_each_mut(
        performance_parameters.batch_size(),
        |(entity1, mut vel, pos1, smoothing_length1, pressure1, density1)| {
            let mut acc = VecAcceleration::zero();
            for particle in get_particles_in_radius(&tree, pos1, &smoothing_length1).iter() {}
        },
    );
}
