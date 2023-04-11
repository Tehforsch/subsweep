use super::halo_iteration::SearchResult;
use crate::communication::Rank;
use crate::dimension::Point;
use crate::hash_map::HashSet;
use crate::prelude::ParticleId;
use crate::voronoi::DDimension;

#[derive(Default, Clone)]
pub struct HaloCache {
    sent_previously: HashSet<(Rank, ParticleId)>,
}

impl HaloCache {
    pub fn get_new_haloes<'a, D: DDimension>(
        &'a mut self,
        rank: Rank,
        iter: impl Iterator<Item = (Point<D>, ParticleId)> + 'a,
    ) -> impl Iterator<Item = SearchResult<D>> + 'a {
        iter.filter(move |(_, id)| self.sent_previously.insert((rank, *id)))
            .map(|(point, id)| SearchResult { point, id })
    }
}
