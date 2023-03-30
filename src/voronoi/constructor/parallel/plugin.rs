use bevy::prelude::Res;
use derive_custom::Named;

use super::super::Constructor;
use super::HaloExporter;
use super::MpiSearchData;
use super::MpiSearchResult;
use super::NumUndecided;
use super::ParallelSearch;
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
use crate::prelude::WorldRank;
use crate::simulation::RaxiomPlugin;
use crate::voronoi::utils::Extent;
use crate::voronoi::ThreeD;

#[derive(Named)]
pub struct ParallelVoronoiGridConstruction;

impl RaxiomPlugin for ParallelVoronoiGridConstruction {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_required_component::<Position>()
            .add_plugin(CommunicationPlugin::<MpiSearchData<ThreeD>>::exchange())
            .add_plugin(CommunicationPlugin::<MpiSearchResult<ThreeD>>::exchange())
            .add_plugin(CommunicationPlugin::<NumUndecided>::default())
            .add_startup_system_to_stage(
                SimulationStartupStages::InsertGrid,
                construct_grid_system,
            );
    }
}

fn construct_grid_system(
    particles: Particles<(&ParticleId, &Position)>,
    mut data_comm: ExchangeCommunicator<MpiSearchData<ThreeD>>,
    mut result_comm: ExchangeCommunicator<MpiSearchResult<ThreeD>>,
    mut finished_comm: Communicator<NumUndecided>,
    tree: Res<QuadTree>,
    indices: Res<TopLevelIndices>,
    global_extent: Res<GlobalExtent>,
    box_: Res<SimulationBox>,
    rank: Res<WorldRank>,
) {
    let extent = Extent {
        min: global_extent.min.value_unchecked(),
        max: global_extent.max.value_unchecked(),
    };
    let search = ParallelSearch {
        data_comm: &mut *data_comm,
        result_comm: &mut *result_comm,
        finished_comm: &mut *finished_comm,
        global_extent: extent,
        tree: &*tree,
        indices: &*indices,
        box_: box_.clone(),
        rank: **rank,
    };
    let halo_exporter = HaloExporter::new(search);
    let cons = Constructor::<ThreeD>::construct_from_iter(
        particles.iter().map(|(i, p)| (*i, p.value_unchecked())),
        halo_exporter,
    );
    let voronoi = cons.voronoi();
}
