use bevy_ecs::event::EventWriter;
use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::Resource;
use derive_custom::Named;
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;
use log::debug;
use mpi::traits::Equivalence;
use serde::Serialize;

use crate::communication::MpiWorld;
use crate::components;
use crate::components::Position;
use crate::domain::DecompositionState;
use crate::domain::IntoKey;
use crate::prelude::Float;
use crate::prelude::Particles;
use crate::prelude::SimulationBox;
use crate::prelude::WorldRank;
use crate::units::Length;
use crate::units::SourceRate;
use crate::units::VecLength;

#[derive(Debug, Clone, Equivalence, Named, Serialize)]
#[name = "total_luminosity"]
pub struct TotalLuminosity(pub SourceRate);

#[derive(Debug, Equivalence, Clone, PartialOrd, PartialEq)]
pub struct DistanceToSourceData(Length);

#[derive(Clone, Debug, Equivalence)]
pub struct Source {
    pub position: VecLength,
    pub rate: SourceRate,
}

#[derive(Resource, Default, Debug)]
pub struct Sources {
    pub sources: Vec<Source>,
}

pub fn set_source_terms_system(
    mut particles: Particles<(&Position, &mut components::Source)>,
    sources: Res<Sources>,
    decomposition: Res<DecompositionState>,
    box_: Res<SimulationBox>,
    world_rank: Res<WorldRank>,
    mut writer: EventWriter<TotalLuminosity>,
) {
    let mut source_comm = MpiWorld::<Source>::new();
    let all_sources = source_comm.all_gather_varcount(&sources.sources);
    let mut particles: Vec<_> = particles.iter_mut().collect();
    let tree: KdTree<Float, 3> = (&particles
        .iter()
        .map(|(pos, _)| pos_to_tree_coord(pos))
        .collect::<Vec<_>>())
        .into();
    for s in all_sources.iter() {
        let key = s.position.into_key(&*box_);
        let rank = decomposition.get_owning_rank(key);
        if rank == **world_rank {
            let (_, index) = tree.nearest_one(&pos_to_tree_coord(&s.position), &squared_euclidean);
            let (_, ref mut source_term) = &mut particles[index];
            ***source_term += s.rate;
        }
    }
    let total: SourceRate = all_sources.iter().map(|source| source.rate).sum();
    writer.send(TotalLuminosity(total));
    debug!("Total luminosity: {:+.2e}", total.in_photons_per_second());
}

fn pos_to_tree_coord(pos: &VecLength) -> [f64; 3] {
    [
        pos.x().value_unchecked(),
        pos.y().value_unchecked(),
        pos.z().value_unchecked(),
    ]
}
