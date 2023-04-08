use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Res;
use derive_custom::Named;

use super::super::Constructor;
use super::mpi_types::TetraIndexSend;
use super::MpiSearchData;
use super::MpiSearchResult;
use super::ParallelSearch;
use super::SendNum;
use crate::communication::ExchangeCommunicator;
use crate::components::Position;
use crate::dimension::ThreeD;
use crate::domain::Decomposition;
use crate::domain::GlobalExtent;
use crate::domain::IdEntityMap;
use crate::domain::QuadTree;
use crate::grid::ParticleType;
use crate::parameters::SimulationBox;
use crate::particle::HaloParticle;
use crate::prelude::CommunicationPlugin;
use crate::prelude::Communicator;
use crate::prelude::ParticleId;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::SimulationStartupStages;
use crate::simulation::RaxiomPlugin;
use crate::units::VecLength;
use crate::voronoi::constructor::halo_cache::HaloCache;
use crate::voronoi::utils::Extent;

#[derive(Named)]
pub struct ParallelVoronoiGridConstruction;

impl RaxiomPlugin for ParallelVoronoiGridConstruction {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_plugin(CommunicationPlugin::<MpiSearchData<ThreeD>>::exchange())
            .add_plugin(CommunicationPlugin::<MpiSearchResult<ThreeD>>::exchange())
            .add_plugin(CommunicationPlugin::<TetraIndexSend>::exchange())
            .add_plugin(CommunicationPlugin::<SendNum>::default())
            .add_startup_system_to_stage(
                SimulationStartupStages::InsertGrid,
                construct_grid_system,
            );
    }
}

fn construct_grid_system(
    mut commands: Commands,
    particles: Particles<(Entity, &ParticleId, &Position)>,
    mut data_comm: ExchangeCommunicator<MpiSearchData<ThreeD>>,
    mut result_comm: ExchangeCommunicator<MpiSearchResult<ThreeD>>,
    mut tetra_index_comm: ExchangeCommunicator<TetraIndexSend>,
    mut finished_comm: Communicator<SendNum>,
    tree: Res<QuadTree>,
    decomposition: Res<Decomposition>,
    global_extent: Res<GlobalExtent>,
    box_: Res<SimulationBox>,
    map: Res<IdEntityMap>,
) {
    let extent = Extent {
        min: global_extent.min.value_unchecked(),
        max: global_extent.max.value_unchecked(),
    };
    let search = ParallelSearch {
        data_comm: &mut *data_comm,
        result_comm: &mut *result_comm,
        finished_comm: &mut finished_comm,
        tetra_index_comm: &mut tetra_index_comm,
        global_extent: extent,
        tree: &tree,
        decomposition: &decomposition,
        box_: box_.clone(),
        halo_cache: HaloCache::default(),
    };
    let cons = Constructor::<ThreeD>::construct_from_iter(
        particles.iter().map(|(_, i, p)| (*i, p.value_unchecked())),
        search,
    );
    for (id, type_, cell) in cons.sweep_grid() {
        match type_ {
            ParticleType::Local(_) => {
                let entity = map.get_by_left(&id).unwrap();
                commands.entity(*entity).insert(cell);
            }
            ParticleType::Remote(remote) => {
                let has_local_neighbours =
                    cell.neighbours.iter().any(|(_, type_)| type_.is_local());
                // If this cell does not have local neighbours, it was imported by "accident"
                // during the delaunay construction and then turned out not to be relevant.
                // We don't need to spawn a halo particle in this case.
                if has_local_neighbours {
                    let pos = cons.get_position_for_particle_id(id);
                    let pos = VecLength::new_unchecked(pos);
                    commands.spawn((HaloParticle { rank: remote.rank }, Position(pos), remote.id));
                }
            }
            ParticleType::Boundary => unreachable!(),
        }
    }
}
