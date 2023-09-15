use bevy_ecs::prelude::Res;
use bevy_ecs::prelude::Resource;
use log::debug;
use mpi::traits::Equivalence;
use ordered_float::OrderedFloat;

use crate::communication::MpiWorld;
use crate::components;
use crate::components::Position;
use crate::domain::DecompositionState;
use crate::domain::IntoKey;
use crate::prelude::Particles;
use crate::prelude::SimulationBox;
use crate::prelude::WorldRank;
use crate::units::Length;
use crate::units::SourceRate;
use crate::units::VecLength;

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
) {
    let mut source_comm = MpiWorld::<Source>::new();
    let all_sources = source_comm.all_gather_varcount(&sources.sources);
    for s in all_sources.iter() {
        let key = s.position.into_key(&*box_);
        let rank = decomposition.get_owning_rank(key);
        if rank == **world_rank {
            let closest = particles
                .iter_mut()
                .map(|(pos, source)| {
                    let dist = **pos - s.position;
                    (OrderedFloat(dist.length().value_unchecked()), source)
                })
                .min_by_key(|(dist, _)| *dist);
            let (_, mut source_term) = closest.unwrap();
            **source_term += s.rate;
        }
    }
    let total: SourceRate = all_sources.iter().map(|source| source.rate).sum();
    debug!("Total luminosity: {:+.2e}", total.in_photons_per_second());
}
