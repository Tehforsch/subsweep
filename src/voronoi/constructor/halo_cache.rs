use super::halo_iteration::SearchResult;
use crate::communication::Rank;
use crate::dimension::Point;
use crate::hash_map::HashSet;
use crate::prelude::ParticleId;
use crate::simulation_box::PeriodicWrapType3d;
use crate::voronoi::DDimension;

#[derive(Default, Clone)]
pub struct HaloCache {
    sent_previously: HashSet<(Rank, (ParticleId, PeriodicWrapType3d))>,
}

impl HaloCache {
    pub fn get_new_haloes<'a, D: DDimension>(
        &'a mut self,
        rank: Rank,
        iter: impl Iterator<Item = (Point<D>, ParticleId, PeriodicWrapType3d)> + 'a,
    ) -> impl Iterator<Item = SearchResult<D>> + 'a {
        iter.filter(move |(_, id, wrap_type)| {
            self.sent_previously.insert((rank, (*id, *wrap_type)))
        })
        .map(|(point, id, periodic_wrap_type)| SearchResult {
            point,
            id,
            periodic_wrap_type,
        })
    }
}
