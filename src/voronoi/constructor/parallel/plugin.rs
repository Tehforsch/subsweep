use bevy::prelude::debug;
use bevy::prelude::warn;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Res;
use derive_custom::Named;

use super::super::Constructor;
use super::ParallelSearch;
use crate::components::Position;
use crate::dimension::ActiveDimension;
use crate::domain::Decomposition;
use crate::domain::IdEntityMap;
use crate::domain::QuadTree;
use crate::grid::ParticleType;
use crate::parameters::SimulationBox;
use crate::particle::HaloParticle;
use crate::prelude::ParticleId;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::SimulationStartupStages;
use crate::simulation::RaxiomPlugin;
use crate::units::VecLength;
use crate::voronoi::constructor::halo_cache::HaloCache;

#[derive(Named)]
pub struct ParallelVoronoiGridConstruction;

impl RaxiomPlugin for ParallelVoronoiGridConstruction {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(SimulationStartupStages::InsertGrid, construct_grid_system);
    }
}

fn warn_if_halo_fraction_too_high(
    num_local_particles: usize,
    num_haloes: usize,
    num_relevant_haloes: usize,
) {
    const HALO_FRACTION_WARNING_TRESHOLD: f64 = 0.05;
    let halo_fraction = num_haloes as f64 / num_local_particles as f64;
    let relevant_halo_fraction = num_relevant_haloes as f64 / num_local_particles as f64;
    if halo_fraction > HALO_FRACTION_WARNING_TRESHOLD {
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
    decomposition: Res<Decomposition>,
    box_: Res<SimulationBox>,
    map: Res<IdEntityMap>,
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
    for (cell_index, cell) in cons.sweep_grid() {
        match cell_index {
            ParticleType::Local(id) => {
                num_local_particles += 1;
                let entity = map.get_by_left(&id).unwrap();
                commands.entity(*entity).insert(cell);
            }
            ParticleType::Remote(remote) => {
                num_haloes += 1;
                let has_local_neighbours =
                    cell.neighbours.iter().any(|(_, type_)| type_.is_local());
                // If this cell does not have local neighbours, it was imported by "accident"
                // during the delaunay construction and then turned out not to be relevant.
                // We don't need to spawn a halo particle in this case.
                if has_local_neighbours {
                    num_relevant_haloes += 1;
                    let pos = cons.get_position_for_cell(cell_index);
                    let pos = VecLength::new_unchecked(pos);
                    commands.spawn((HaloParticle { rank: remote.rank }, Position(pos), remote.id));
                }
            }
            ParticleType::Boundary => {}
            ParticleType::LocalPeriodic(_) => {}
            ParticleType::RemotePeriodic(_) => {}
        }
    }
    warn_if_halo_fraction_too_high(num_local_particles, num_haloes, num_relevant_haloes);
}
