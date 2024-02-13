use bevy_ecs::prelude::Commands;
use bevy_ecs::prelude::Entity;
use bevy_ecs::prelude::Res;
use derive_custom::subsweep_parameters;
use derive_custom::Named;
use log::debug;
use log::warn;

use super::super::Constructor;
use super::ParallelSearch;
use crate::communication::Rank;
use crate::components::Position;
use crate::dimension::ActiveDimension;
use crate::domain::DecompositionState;
use crate::domain::IdEntityMap;
use crate::domain::QuadTree;
use crate::parameters::SimulationBox;
use crate::parameters::SweepParameters;
use crate::particle::HaloParticle;
use crate::prelude::ParticleId;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::StartupStages;
use crate::simulation::SubsweepPlugin;
use crate::sweep::grid::Cell;
use crate::sweep::grid::ParticleType;
use crate::units::Length;
use crate::units::VecLength;
use crate::voronoi::constructor::halo_cache::HaloCache;
use crate::voronoi::CellIndex;

#[subsweep_parameters("grid")]
pub struct GridParameters {
    /// The initial search radius for halo iteration during grid construction.
    pub initial_search_radius: Option<Length>,
}

#[derive(Named)]
pub struct ParallelVoronoiGridConstruction;

impl SubsweepPlugin for ParallelVoronoiGridConstruction {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(StartupStages::InsertGrid, construct_grid_system)
            .add_parameter_type::<GridParameters>();
    }
}

fn warn_if_halo_fraction_too_high(
    num_local_particles: usize,
    num_haloes: usize,
    num_relevant_haloes: usize,
) {
    const HALO_FRACTION_WARNING_THRESHOLD: f64 = 0.05;
    let halo_fraction = num_haloes as f64 / num_local_particles as f64;
    let relevant_halo_fraction = num_relevant_haloes as f64 / num_local_particles as f64;
    if halo_fraction > HALO_FRACTION_WARNING_THRESHOLD {
        warn!(
            "High halo fraction: {:.1}% ({:.1}% of those are relevant)",
            halo_fraction * 100.0,
            relevant_halo_fraction * 100.0
        );
    } else {
        debug!("Halo fraction: {:.1}%", halo_fraction * 100.0);
    }
}

pub fn construct_grid_system(
    mut commands: Commands,
    particles: Particles<(Entity, &ParticleId, &Position)>,
    tree: Res<QuadTree>,
    decomposition: Res<DecompositionState>,
    box_: Res<SimulationBox>,
    map: Res<IdEntityMap>,
    sweep_parameters: Res<SweepParameters>,
    grid_parameters: Res<GridParameters>,
) {
    let num_points_local = particles.iter().count();
    let search = ParallelSearch::new(
        &tree,
        &decomposition,
        box_.clone(),
        HaloCache::default(),
        num_points_local,
    );
    let cons = Constructor::<ActiveDimension>::construct_from_iter(
        particles.iter().map(|(_, i, p)| (*i, p.value_unchecked())),
        search,
        grid_parameters
            .initial_search_radius
            .map(|r| r.value_unchecked()),
    );
    let mut num_haloes = 0;
    let mut num_relevant_haloes = 0;
    let mut num_local_particles = 0;
    let mut add_halo =
        |commands: &mut Commands, cell_index: CellIndex, cell: Cell, rank: Rank, id: ParticleId| {
            num_haloes += 1;
            let has_local_neighbours = cell.neighbours.iter().any(|(_, type_)| type_.is_local());
            // If this cell does not have local neighbours, it was imported by "accident"
            // during the delaunay construction and then turned out not to be relevant.
            // We don't need to spawn a halo particle in this case.
            if has_local_neighbours {
                num_relevant_haloes += 1;
                let pos = cons.get_position_for_cell(cell_index);
                let pos = VecLength::new_unchecked(pos);
                commands.spawn((HaloParticle { rank }, Position(pos), id));
            }
        };
    for (cell_index, cell) in cons.sweep_grid(sweep_parameters.periodic) {
        match cell_index {
            ParticleType::Local(id) => {
                num_local_particles += 1;
                let entity = map.get_by_left(&id).unwrap();
                commands.entity(*entity).insert(cell);
            }
            ParticleType::Remote(remote) => {
                add_halo(&mut commands, cell_index, cell, remote.rank, remote.id);
            }
            ParticleType::Boundary => {}
            ParticleType::LocalPeriodic(_) => {}
            ParticleType::RemotePeriodic(remote) => {
                add_halo(&mut commands, cell_index, cell, remote.rank, remote.id);
            }
        }
    }
    warn_if_halo_fraction_too_high(num_local_particles, num_haloes, num_relevant_haloes);
}
