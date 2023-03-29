use bevy::ecs::system::Commands;
use bevy::prelude::Res;

use super::MpiSearchData;
use super::MpiSearchResult;
use super::ParallelSearch;
use crate::communication::local_sim_building::build_local_communication_sim_with_custom_logic;
use crate::communication::ExchangeCommunicator;
use crate::components::Position;
use crate::domain::DomainDecompositionPlugin;
use crate::domain::GlobalExtent;
use crate::domain::QuadTree;
use crate::domain::TopLevelIndices;
use crate::parameters::DomainParameters;
use crate::parameters::DomainStage;
use crate::parameters::SimulationBox;
use crate::parameters::SimulationParameters;
use crate::prelude::CommunicationPlugin;
use crate::prelude::LocalParticle;
use crate::prelude::ParticleId;
use crate::prelude::Particles;
use crate::prelude::WorldRank;
use crate::simulation::Simulation;
use crate::simulation_plugin::SimulationStartupStages;
use crate::stages::SimulationStagesPlugin;
use crate::units::Length;
use crate::units::Time;
use crate::units::VecLength;
use crate::voronoi::constructor::halo_iteration::HaloExporter;
use crate::voronoi::test_utils::TestDimension;
use crate::voronoi::utils::Extent;
use crate::voronoi::Constructor;
use crate::voronoi::ThreeD;

#[test]
#[ignore]
fn parallel_voronoi_construction() {
    for num_ranks in 1..10 {
        println!("Running on {}", num_ranks);
        build_local_communication_sim_with_custom_logic(
            |sim: &mut Simulation| build_sim(sim),
            |mut sim| {
                sim.update();
            },
            num_ranks,
        );
    }
}

fn build_sim(sim: &mut Simulation) {
    let simulation_box = SimulationBox::cube_from_side_length(Length::meters(1.0));
    sim.add_parameter_file_contents("".into())
        .add_parameters_explicitly(DomainParameters {
            stage: DomainStage::Startup,
            ..Default::default()
        })
        .add_plugin(SimulationStagesPlugin)
        .add_required_component::<Position>()
        .add_plugin(DomainDecompositionPlugin)
        .add_plugin(CommunicationPlugin::<MpiSearchData<ThreeD>>::exchange())
        .add_plugin(CommunicationPlugin::<MpiSearchResult<ThreeD>>::exchange())
        .add_parameters_explicitly(simulation_box)
        .add_parameters_explicitly(SimulationParameters {
            final_time: Some(Time::zero()),
        })
        .add_startup_system_to_stage(
            SimulationStartupStages::InsertComponents,
            spawn_particles_system,
        )
        .add_startup_system_to_stage(SimulationStartupStages::InsertGrid, construct_grid_system);
}

fn spawn_particles_system(mut commands: Commands, rank: Res<WorldRank>) {
    for p in ThreeD::get_example_point_set(**rank as usize) {
        commands.spawn((LocalParticle, Position(VecLength::new_unchecked(p))));
    }
}

fn construct_grid_system(
    particles: Particles<(&ParticleId, &Position)>,
    mut data_comm: ExchangeCommunicator<MpiSearchData<ThreeD>>,
    mut result_comm: ExchangeCommunicator<MpiSearchResult<ThreeD>>,
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
