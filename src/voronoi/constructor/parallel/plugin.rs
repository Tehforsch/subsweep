use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Res;
use bevy::utils::StableHashMap;
use derive_custom::Named;

use super::super::Constructor;
use super::HaloExporter;
use super::MpiSearchData;
use super::MpiSearchResult;
use super::NumUndecided;
use super::ParallelSearch;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::components::Position;
use crate::domain::GlobalExtent;
use crate::domain::QuadTree;
use crate::domain::TopLevelIndices;
use crate::parameters::SimulationBox;
use crate::prelude::CommunicationPlugin;
use crate::prelude::Communicator;
use crate::prelude::ParticleId;
use crate::prelude::Particles;
use crate::prelude::Simulation;
use crate::prelude::SimulationStartupStages;
use crate::simulation::RaxiomPlugin;
use crate::voronoi::utils::Extent;
use crate::voronoi::ThreeD;

#[derive(Named)]
pub struct ParallelVoronoiGridConstruction;

impl RaxiomPlugin for ParallelVoronoiGridConstruction {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_plugin(CommunicationPlugin::<MpiSearchData<ThreeD>>::exchange())
            .add_plugin(CommunicationPlugin::<MpiSearchResult<ThreeD>>::exchange())
            .add_plugin(CommunicationPlugin::<NumUndecided>::default())
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
    mut finished_comm: Communicator<NumUndecided>,
    tree: Res<QuadTree>,
    indices: Res<TopLevelIndices>,
    global_extent: Res<GlobalExtent>,
    box_: Res<SimulationBox>,
) {
    let extent = Extent {
        min: global_extent.min.value_unchecked(),
        max: global_extent.max.value_unchecked(),
    };
    let already_sent = DataByRank::from_communicator(&*data_comm);
    let search = ParallelSearch {
        data_comm: &mut *data_comm,
        result_comm: &mut *result_comm,
        finished_comm: &mut *finished_comm,
        global_extent: extent,
        tree: &*tree,
        indices: &*indices,
        box_: box_.clone(),
        already_sent,
    };
    let halo_exporter = HaloExporter::new(search);
    let cons = Constructor::<ThreeD>::construct_from_iter(
        particles.iter().map(|(_, i, p)| (*i, p.value_unchecked())),
        halo_exporter,
    );
    let id_entity_map: StableHashMap<_, _> = particles
        .iter()
        .map(|(entity, id, _)| (id, entity))
        .collect();
    for (id, cell) in cons.sweep_grid() {
        let entity = id_entity_map[&id];
        commands.entity(entity).insert(cell);
    }
}
