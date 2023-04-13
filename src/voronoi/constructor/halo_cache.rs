use super::halo_iteration::SearchResult;
use crate::communication::Rank;
use crate::dimension::Dimension;
use crate::dimension::Point;
use crate::dimension::WrapType;
use crate::hash_map::HashSet;
use crate::prelude::ParticleId;

#[derive(Default, Clone)]
pub struct HaloCache<D: Dimension> {
    sent_previously: HashSet<(Rank, (ParticleId, WrapType<D>))>,
}

impl<D: Dimension> HaloCache<D> {
    pub fn get_new_haloes<'a>(
        &'a mut self,
        rank: Rank,
        iter: impl Iterator<Item = (Point<D>, ParticleId, WrapType<D>)> + 'a,
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
