pub(crate) mod hydro_components;
mod parameters;
mod quadtree;

use std::f64::consts::PI;

use bevy::prelude::*;

use self::hydro_components::InternalEnergy;
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
use crate::units::helpers::VecQuantity;
use crate::units::Density;
use crate::units::Dimension;
use crate::units::Energy;
use crate::units::Length;
use crate::units::NumberDensity;
use crate::units::VecAcceleration;
use crate::units::VecLength;
use crate::units::NONE;

const GAMMA: f64 = 5.0 / 3.0;

fn kernel_function(r: Length, h: Length) -> f64 {
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

fn kernel_derivative_function(r: Length, h: Length) -> f64 {
    let ratio = *(r / h).value();
    if ratio < 0.5 {
        -2.0 * ratio + 3.0 * ratio.powi(2)
    } else if ratio < 1.0 {
        -(1.0 - ratio).powi(2)
    } else {
        0.0
    }
}

#[cfg(feature = "2d")]
fn kernel(r: Length, h: Length) -> NumberDensity {
    80.0 / (7.0 * PI * h.squared()) * kernel_function(r, h)
}

#[cfg(not(feature = "2d"))]
fn kernel(r: Length, h: Length) -> NumberDensity {
    8.0 / (PI * h.cubed()) * kernel_function(r, h)
}

#[cfg(feature = "2d")]
fn symmetric_kernel_derivative(
    r1: VecLength,
    r2: VecLength,
    h1: Length,
    h2: Length,
) -> VecQuantity<{ Dimension { length: -3, ..NONE } }> {
    let dist = r1 - r2;
    let length = dist.length();
    dist / length
        * (48.0 / (7.0 * PI * h1.powi::<3>()) * kernel_derivative_function(length, h1)
            + 48.0 / (7.0 * PI * h2.powi::<3>()) * kernel_derivative_function(length, h2))
}

#[cfg(not(feature = "2d"))]
fn symmetric_kernel_derivative(
    r1: VecLength,
    r2: VecLength,
    h1: Length,
    h2: Length,
) -> VecQuantity<{ Dimension { length: -4, ..NONE } }> {
    let dist = r1 - r2;
    let length = dist.length();
    dist / length
        * (48.0 / (PI * h1.powi::<4>()) * kernel_derivative_function(length, h1)
            + 48.0 / (PI * h2.powi::<4>()) * kernel_derivative_function(length, h2))
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
                set_smoothing_lengths_system.before(construct_quad_tree_system),
            )
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
                compute_energy_change_system.after(compute_pressure_and_density_system),
            )
            .add_system_to_stage(
                HydrodynamicsStages::Hydrodynamics,
                compute_forces_system.after(compute_energy_change_system),
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
            components::SmoothingLength::default(),
            components::InternalEnergy(Energy::joules(1e6)),
        ));
    }
}

fn compute_pressure_and_density_system(
    mut pressures: Particles<(
        &mut components::Pressure,
        &mut components::Density,
        &InternalEnergy,
        &SmoothingLength,
        &Position,
        &Mass,
    )>,
    masses: Query<&Mass, With<LocalParticle>>,
    tree: Res<QuadTree>,
    performance_parameters: Res<PerformanceParameters>,
) {
    pressures.par_for_each_mut(
        performance_parameters.batch_size(),
        |(mut pressure, mut density, internal_energy, smoothing_length, pos, mass)| {
            **density = Density::zero();
            for particle in get_particles_in_radius(&tree, pos, smoothing_length).iter() {
                let mass2 = masses.get(particle.entity).unwrap();
                let distance = particle.pos.distance(pos);
                **density += **mass2 * kernel(distance, **smoothing_length);
            }
            // P = (gamma - 1) * rho * u
            // u = energy / mass
            **pressure = (GAMMA - 1.0) * **density * **internal_energy / **mass
        },
    );
}

fn compute_energy_change_system(
    mut particles1: Particles<(
        &mut InternalEnergy,
        &Mass,
        &Velocity,
        &Position,
        &SmoothingLength,
        &components::Pressure,
        &components::Density,
    )>,
    particles2: Particles<(
        &Position,
        &components::Velocity,
        &components::Pressure,
        &components::Density,
        &components::Mass,
        &SmoothingLength,
    )>,
    tree: Res<QuadTree>,
    timestep: Res<Timestep>,
    performance_parameters: Res<PerformanceParameters>,
) {
    particles1.par_for_each_mut(
        performance_parameters.batch_size(),
        |(mut energy1, mass1, velocity1, position1, smoothing_length1, pressure1, density1)| {
            let mut d_energy =
                Energy::zero() / crate::units::Mass::one() / crate::units::Time::one();
            for particle in get_particles_in_radius(&tree, position1, smoothing_length1).iter() {
                let (position2, velocity2, pressure2, density2, mass2, smoothing_length2) =
                    particles2.get(particle.entity).unwrap();
                if **position1 == **position2 {
                    continue;
                }
                let relative_velocity = **velocity1 - **velocity2;
                let kernel_derivative = symmetric_kernel_derivative(
                    **position1,
                    **position2,
                    **smoothing_length1,
                    **smoothing_length2,
                );
                // TODO: viscosity
                d_energy += 0.5
                    * **mass2
                    * ((**pressure1 / density1.squared()) + (**pressure2 / density2.squared()))
                    * relative_velocity.dot(kernel_derivative);
            }
            **energy1 += d_energy * **timestep * **mass1;
        },
    );
}

fn compute_forces_system(
    mut particles1: Particles<(
        &mut Velocity,
        &Position,
        &SmoothingLength,
        &components::Pressure,
        &components::Density,
    )>,
    particles2: Particles<(
        &Position,
        &components::Pressure,
        &components::Density,
        &components::Mass,
        &SmoothingLength,
    )>,
    tree: Res<QuadTree>,
    timestep: Res<Timestep>,
    performance_parameters: Res<PerformanceParameters>,
) {
    particles1.par_for_each_mut(
        performance_parameters.batch_size(),
        |(mut velocity1, position1, smoothing_length1, pressure1, density1)| {
            let mut d_vel = VecAcceleration::zero();
            for particle in get_particles_in_radius(&tree, position1, smoothing_length1).iter() {
                let (position2, pressure2, density2, mass2, smoothing_length2) =
                    particles2.get(particle.entity).unwrap();
                if **position1 == **position2 {
                    continue;
                }
                let kernel_derivative = symmetric_kernel_derivative(
                    **position1,
                    **position2,
                    **smoothing_length1,
                    **smoothing_length2,
                );
                // TODO: viscosity
                d_vel += -0.5
                    * **mass2
                    * ((**pressure1 / density1.squared()) + (**pressure2 / density2.squared()))
                    * kernel_derivative;
            }
            **velocity1 += d_vel * **timestep;
        },
    );
}
