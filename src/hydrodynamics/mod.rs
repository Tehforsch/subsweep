use std::f64::consts::PI;

use bevy::prelude::*;
use mpi::traits::Equivalence;

use self::hydro_components::InternalEnergy;
use self::hydro_components::Pressure;
use self::hydro_components::SmoothingLength;
use self::quadtree::bounding_boxes_overlap;
use self::quadtree::construct_quad_tree_system;
use self::quadtree::get_particles_in_radius;
use self::quadtree::QuadTree;
use crate::communication::CommunicationPlugin;
use crate::communication::Rank;
use crate::communication::SyncCommunicator;
use crate::components;
use crate::components::Mass;
use crate::components::Position;
use crate::components::Timestep;
use crate::components::Velocity;
use crate::domain;
use crate::domain::extent::Extent;
use crate::domain::TopLevelIndices;
use crate::named::Named;
use crate::performance_parameters::PerformanceParameters;
use crate::prelude::LocalParticle;
use crate::prelude::MVec;
use crate::prelude::Particles;
use crate::prelude::WorldRank;
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
use crate::units::BOLTZMANN_CONSTANT;
use crate::units::NONE;
use crate::units::PROTON_MASS;
use crate::visualization::DrawCircle;
use crate::visualization::DrawItem;
use crate::visualization::Pixels;
use crate::visualization::RColor;
use crate::visualization::VisualizationStage;

pub(crate) mod hydro_components;
mod parameters;
mod quadtree;

pub use self::parameters::HydrodynamicsParameters;
pub use self::parameters::InitialGasEnergy;

const GAMMA: f64 = 5.0 / 3.0;

// Could eventually become a more dynamic approach (similar to ExchangeDataPlugin)
// but for now this is probably fine
#[derive(Equivalence, Bundle)]
struct RemoteParticleData {
    pub position: Position,
    pub smoothing_length: SmoothingLength,
    pub density: components::Density,
    pub pressure: Pressure,
    pub mass: Mass,
    pub velocity: components::Velocity,
    pub internal_energy: InternalEnergy,
}

#[derive(Component)]
pub struct HaloParticle {
    pub rank: Rank,
}

/// A convenience type to query for halo particles.
pub type HaloParticles<'world, 'state, T, F = ()> =
    Query<'world, 'state, T, (With<HaloParticle>, F)>;

/// A convenience type to query for local and halo particles.
pub type HydroParticles<'world, 'state, T, F = ()> =
    Query<'world, 'state, T, (Or<(With<HaloParticle>, With<LocalParticle>)>, F)>;

fn kernel_function(r: Length, h: Length) -> f64 {
    // Spline Kernel, Monaghan & Lattanzio 1985
    let ratio = (r / h).value();
    if ratio < 0.5 {
        1.0 - 6.0 * ratio.powi(2) + 6.0 * ratio.powi(3)
    } else if ratio < 1.0 {
        2.0 * (1.0 - ratio).powi(3)
    } else {
        0.0
    }
}

fn kernel_derivative_function(r: Length, h: Length) -> f64 {
    let ratio = (r / h).value();
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
    Initial,
    Hydrodynamics,
}

#[derive(Named)]
pub struct HydrodynamicsPlugin;

impl RaxiomPlugin for HydrodynamicsPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        let initial_halo_exchange = halo_exchange_system.label("initial_halo_exchange");
        let density_pressure_halo_exchange =
            halo_exchange_system.label("density_pressure_halo_exchange");
        sim.add_parameter_type::<HydrodynamicsParameters>()
            .add_plugin(CommunicationPlugin::<RemoteParticleData>::sync())
            .insert_resource(QuadTree::make_empty_leaf_from_extent(Extent::default()))
            .add_system_to_stage(
                HydrodynamicsStages::Initial,
                set_smoothing_lengths_system.before("initial_halo_exchange"),
            )
            .add_system_to_stage(HydrodynamicsStages::Initial, initial_halo_exchange)
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
                density_pressure_halo_exchange.after(compute_pressure_and_density_system),
            )
            .add_system_to_stage(
                HydrodynamicsStages::Hydrodynamics,
                compute_energy_change_system
                    .after(compute_pressure_and_density_system)
                    .after("density_pressure_halo_exchange"),
            )
            .add_system_to_stage(
                HydrodynamicsStages::Hydrodynamics,
                compute_forces_system
                    .after(compute_energy_change_system)
                    .after("density_pressure_halo_exchange"),
            )
            .add_startup_system_to_stage(
                StartupStage::PostStartup,
                insert_pressure_and_density_system,
            )
            .add_system_to_stage(
                VisualizationStage::AddVisualization,
                show_halo_particles_system,
            )
            .add_derived_component::<components::Pressure>()
            .add_derived_component::<components::SmoothingLength>()
            .add_derived_component::<components::InternalEnergy>()
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

fn halo_exchange_system(
    mut commands: Commands,
    particles: Particles<
        (
            Entity,
            &Position,
            &SmoothingLength,
            &components::Density,
            &Pressure,
            &Mass,
            &InternalEnergy,
            &components::Velocity,
        ),
        Without<HaloParticle>,
    >,
    mut halo_particles: HaloParticles<(
        &mut Position,
        &mut SmoothingLength,
        &mut components::Density,
        &mut Pressure,
        &mut Mass,
        &mut InternalEnergy,
        &mut components::Velocity,
    )>,
    mut communicator: SyncCommunicator<RemoteParticleData>,
    indices: Res<TopLevelIndices>,
    tree: Res<domain::QuadTree>,
    world_rank: Res<WorldRank>,
) {
    for (entity, pos, smoothing_length, density, pressure, mass, internal_energy, velocity) in
        particles.iter()
    {
        for (rank, index) in indices
            .iter()
            .flat_map(|(rank, indices)| indices.iter().map(|index| (*rank, index)))
        {
            if rank == **world_rank {
                continue;
            }
            let tree = &tree[index];
            if bounding_boxes_overlap(
                pos,
                &(MVec::ONE * **smoothing_length),
                &tree.extent.center,
                &tree.extent.side_lengths(),
            ) {
                communicator.send_sync(
                    rank,
                    entity,
                    RemoteParticleData {
                        position: pos.clone(),
                        smoothing_length: smoothing_length.clone(),
                        density: density.clone(),
                        pressure: pressure.clone(),
                        mass: mass.clone(),
                        internal_energy: internal_energy.clone(),
                        velocity: velocity.clone(),
                    },
                );
            }
        }
    }
    let spawn_particle = |rank: Rank, data: RemoteParticleData| {
        commands
            .spawn()
            .insert_bundle(data)
            .insert(HaloParticle { rank })
            .id()
    };
    let mut sync = communicator.receive_sync(spawn_particle);
    sync.despawn_deleted(&mut commands);
    for (_, data) in sync.updated.drain_all() {
        for (entity, new_data) in data.into_iter() {
            let mut particle = halo_particles.get_mut(entity).unwrap();
            *particle.0 = new_data.position;
            *particle.1 = new_data.smoothing_length;
            *particle.2 = new_data.density;
            *particle.3 = new_data.pressure;
            *particle.4 = new_data.mass;
            *particle.5 = new_data.internal_energy;
            *particle.6 = new_data.velocity;
        }
    }
}

fn show_halo_particles_system(
    mut commands: Commands,
    undrawn_halo_particles: HaloParticles<(Entity, &Position), Without<DrawCircle>>,
    mut drawn_halo_particles: HaloParticles<(&Position, &mut DrawCircle), With<DrawCircle>>,
) {
    for (entity, pos) in undrawn_halo_particles.iter() {
        commands.entity(entity).insert(DrawCircle {
            position: **pos,
            radius: Pixels(10.0),
            color: RColor::RED,
        });
    }
    for (pos, mut circle) in drawn_halo_particles.iter_mut() {
        circle.set_translation(pos);
    }
}

fn insert_pressure_and_density_system(
    mut commands: Commands,
    particles: Particles<
        (Entity, &Mass),
        (Without<components::Pressure>, Without<components::Density>),
    >,
    parameters: Res<HydrodynamicsParameters>,
) {
    for (entity, mass) in particles.iter() {
        let energy = match parameters.initial_gas_energy {
            InitialGasEnergy::TemperatureAndMolecularWeight {
                temperature,
                molecular_weight,
            } => {
                temperature * (BOLTZMANN_CONSTANT / PROTON_MASS) * (1.0 / (GAMMA - 1.0))
                    / molecular_weight
                    * **mass
            }
            InitialGasEnergy::Energy(energy) => energy * **mass,
        };
        commands.entity(entity).insert_bundle((
            components::Pressure::default(),
            components::Density::default(),
            components::SmoothingLength::default(),
            components::InternalEnergy(energy),
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
    masses: HydroParticles<&Mass>,
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
        &Timestep,
    )>,
    particles2: HydroParticles<(
        &Position,
        &components::Velocity,
        &components::Pressure,
        &components::Density,
        &components::Mass,
        &SmoothingLength,
    )>,
    tree: Res<QuadTree>,
    performance_parameters: Res<PerformanceParameters>,
) {
    particles1.par_for_each_mut(
        performance_parameters.batch_size(),
        |(
            mut energy1,
            mass1,
            velocity1,
            position1,
            smoothing_length1,
            pressure1,
            density1,
            timestep,
        )| {
            let mut d_energy = Energy::zero()
                / crate::units::Mass::one_unchecked()
                / crate::units::Time::one_unchecked();
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
        &Timestep,
    )>,
    particles2: HydroParticles<(
        &Position,
        &components::Pressure,
        &components::Density,
        &components::Mass,
        &SmoothingLength,
    )>,
    tree: Res<QuadTree>,
    performance_parameters: Res<PerformanceParameters>,
) {
    particles1.par_for_each_mut(
        performance_parameters.batch_size(),
        |(mut velocity1, position1, smoothing_length1, pressure1, density1, timestep)| {
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
                if density2.value_unchecked() == 0.0 && pressure2.value_unchecked() == 0.0 {
                    panic!()
                }
            }
            **velocity1 += d_vel * **timestep;
        },
    );
}
