use bevy_ecs::prelude::Commands;
use bevy_ecs::prelude::Entity;
use bevy_ecs::prelude::Res;
use derive_custom::Named;
use hdf5::File;
use hdf5::H5Type;
use log::debug;
use log::warn;
use mpi::traits::Equivalence;

use super::super::Constructor;
use super::ParallelSearch;
use crate::communication::communicator::Communicator;
use crate::communication::Rank;
use crate::communication::MPI_UNIVERSE;
use crate::components::Position;
use crate::dimension::ActiveDimension;
use crate::domain::DecompositionState;
use crate::domain::IdEntityMap;
use crate::domain::QuadTree;
use crate::io::file_distribution::get_rank_output_assignment_for_rank;
use crate::parameters::SimulationBox;
use crate::parameters::SweepParameters;
use crate::particle::HaloParticle;
use crate::prelude::ParticleId;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::StartupStages;
use crate::prelude::WorldRank;
use crate::prelude::WorldSize;
use crate::simulation::SubsweepPlugin;
use crate::sweep::grid::Cell;
use crate::sweep::grid::ParticleType;
use crate::units::VecLength;
use crate::voronoi::constructor::halo_cache::HaloCache;
use crate::voronoi::CellIndex;

#[derive(Named)]
pub struct ParallelVoronoiGridConstruction;

impl SubsweepPlugin for ParallelVoronoiGridConstruction {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(StartupStages::InsertGrid, construct_grid_system);
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
    rank: Res<WorldRank>,
    world_size: Res<WorldSize>,
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
    let data: Vec<_> = cons
        .data
        .triangulation
        .tetras
        .iter()
        .filter_map(|(_, tetra)| {
            let map_p = |p| match cons.data.get_particle_type(p) {
                ParticleType::Local(p) => Some(ParticleData {
                    index: p.index,
                    rank: **rank,
                }),
                ParticleType::Remote(p) => Some(ParticleData {
                    index: p.id.index,
                    rank: p.rank,
                }),
                ParticleType::Boundary => None,
                ParticleType::LocalPeriodic(p) => Some(ParticleData {
                    index: p.id.index,
                    rank: p.id.rank,
                }),
                ParticleType::RemotePeriodic(p) => Some(ParticleData {
                    index: p.id.index,
                    rank: p.rank,
                }),
            };
            let p1 = map_p(tetra.p1)?;
            let p2 = map_p(tetra.p2)?;
            let p3 = map_p(tetra.p3)?;
            let p4 = map_p(tetra.p4)?;
            Some(Tetra { p1, p2, p3, p4 })
        })
        .collect();
    #[derive(Equivalence, Clone)]
    struct NumTetras(usize);
    let num_tetras_local = NumTetras(data.len());
    let num_tetras_per_rank = Communicator::<NumTetras>::new().all_gather(&num_tetras_local);
    let num_tetras_per_rank: Vec<_> = num_tetras_per_rank.into_iter().map(|x| x.0).collect();
    let rank_assignment = get_rank_output_assignment_for_rank(&num_tetras_per_rank, 1, **rank);
    if rank.is_main() {
        let f = File::create("output/grid.hdf5").unwrap();
        f.new_dataset::<Tetra>()
            .shape(num_tetras_per_rank.iter().sum::<usize>())
            .create("tetras")
            .unwrap();
    }
    for i in 0..**world_size {
        if i as i32 == **rank {
            for region in rank_assignment.regions.iter() {
                let d = File::open_rw("output/grid.hdf5").unwrap();
                d.dataset("tetras")
                    .unwrap()
                    .write_slice(&data, region.start..region.end)
                    .unwrap();
            }
        }
        MPI_UNIVERSE.barrier();
    }
}

#[derive(H5Type, Debug, Clone, Copy)]
#[repr(packed)]
struct ParticleData {
    index: u32,
    rank: i32,
}

#[derive(H5Type, Debug)]
#[repr(packed)]
struct Tetra {
    p1: ParticleData,
    p2: ParticleData,
    p3: ParticleData,
    p4: ParticleData,
}
