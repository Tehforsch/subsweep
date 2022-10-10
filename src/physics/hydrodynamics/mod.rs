mod parameters;
mod quadtree;

use std::f64::consts::PI;

use bevy::prelude::*;

pub use self::parameters::HydrodynamicsParameters;
use self::quadtree::construct_quad_tree_system;
use self::quadtree::get_particles_in_radius;
use self::quadtree::QuadTree;
use super::Timestep;
use crate::density;
use crate::domain::extent::Extent;
use crate::mass;
use crate::mass::Mass;
use crate::named::Named;
use crate::performance_parameters::PerformanceParameters;
use crate::position::Position;
use crate::prelude::LocalParticle;
use crate::pressure;
use crate::quadtree::LeafDataType;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::units::Density;
use crate::units::Pressure;
use crate::units::VecAcceleration;
use crate::velocity::Velocity;

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
            .add_derived_component::<pressure::Pressure>()
            .add_derived_component::<density::Density>();
    }
}

fn insert_pressure_and_density_system(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<LocalParticle>,
            Without<pressure::Pressure>,
            Without<density::Density>,
        ),
    >,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert_bundle((pressure::Pressure::default(), density::Density::default()));
    }
}
fn compute_pressure_and_density_system(
    mut pressures: Query<
        (
            &mut pressure::Pressure,
            &mut density::Density,
            &Position,
            &Mass,
        ),
        With<LocalParticle>,
    >,
    parameters: Res<HydrodynamicsParameters>,
    tree: Res<QuadTree>,
    performance_parameters: Res<PerformanceParameters>,
) {
    let cutoff_squared = parameters.smoothing_length.squared();
    let poly_6 = 4.0 / (PI * parameters.smoothing_length.powi::<8>());
    let rest_density = Density::kilogram_per_square_meter(1.0);
    let gas_const = Pressure::pascals(100000.0) / rest_density;
    pressures.par_for_each_mut(
        performance_parameters.batch_size(),
        |(mut pressure, mut density, pos1, mass)| {
            **density = Density::zero();
            for particle in
                get_particles_in_radius(&tree, pos1, &parameters.smoothing_length).iter()
            {
                {
                    let distance_squared = pos1.distance_squared(particle.pos());

                    if distance_squared < cutoff_squared {
                        **density +=
                            **mass * poly_6 * (cutoff_squared - distance_squared).powi::<3>();
                    }
                }
                **pressure = gas_const * (**density - rest_density);
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
            &pressure::Pressure,
            &density::Density,
        ),
        With<LocalParticle>,
    >,
    particles2: Query<
        (
            Entity,
            &Position,
            &pressure::Pressure,
            &density::Density,
            &mass::Mass,
        ),
        With<LocalParticle>,
    >,
    tree: Res<QuadTree>,
    timestep: Res<Timestep>,
    parameters: Res<HydrodynamicsParameters>,
    performance_parameters: Res<PerformanceParameters>,
) {
    let spiky_grad = -10.0 / (PI * parameters.smoothing_length.powi::<5>());
    particles1.par_for_each_mut(
        performance_parameters.batch_size(),
        |(entity1, mut vel, pos1, pressure1, density1)| {
            let mut acc = VecAcceleration::zero();
            for particle in
                get_particles_in_radius(&tree, pos1, &parameters.smoothing_length).iter()
            {
                let entity2 = particle.entity;
                if entity1 == entity2 {
                    continue;
                }
                let pos2 = particle.pos;
                let mass2 = particles2.get_component::<mass::Mass>(entity2).unwrap();
                let pressure2 = particles2
                    .get_component::<pressure::Pressure>(entity2)
                    .unwrap();
                let density2 = particles2
                    .get_component::<density::Density>(entity2)
                    .unwrap();

                let distance = pos2 - **pos1;
                let distance_normalized = distance.normalize();
                let length = distance.length();

                if length < parameters.smoothing_length {
                    acc += distance_normalized * **mass2 * (**pressure1 + **pressure2)
                        / (2.0 * **density2)
                        * spiky_grad
                        * (parameters.smoothing_length - length).cubed()
                        / **density1;
                }
            }
            **vel += acc * **timestep;
        },
    );
}
