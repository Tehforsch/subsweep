use bevy::utils::StableHashSet;
use ordered_float::OrderedFloat;

use super::halo_iteration::SearchResult;
use crate::communication::Rank;
use crate::prelude::ParticleId;
use crate::voronoi::primitives::DVector;
use crate::voronoi::Dimension;
use crate::voronoi::Point;

#[derive(Default, Clone)]
pub struct HaloCache {
    sent_previously: StableHashSet<(Rank, ParticleId)>,
    sent_now: StableHashSet<(Rank, ParticleId)>,
}

pub enum CachedSearchResult<D: Dimension> {
    NewPoint(SearchResult<D>),
    NewPointThatHasJustBeenExported,
    NothingNew,
}

impl HaloCache {
    /// Given a rank and the origin point of the search, find the
    /// point in iter which is closest to the origin point and which
    /// hasn't been sent to this rank in some previous halo
    /// iteration. If there is no such point (that is, all haloes in
    /// the search radius have been sent to this rank already),
    pub fn get_closest_new<D: Dimension>(
        &mut self,
        rank: Rank,
        search_origin: Point<D>,
        iter: impl Iterator<Item = (Point<D>, ParticleId)>,
    ) -> CachedSearchResult<D> {
        let closest = iter
            .filter(|(_, id)| !self.sent_previously.contains(&(rank, *id)))
            .min_by_key(|(pos, _)| OrderedFloat(search_origin.distance(*pos)));
        match closest {
            Some((point, id)) => {
                if self.sent_now.insert((rank, id)) {
                    CachedSearchResult::NewPoint(SearchResult { point, id })
                } else {
                    CachedSearchResult::NewPointThatHasJustBeenExported
                }
            }
            None => CachedSearchResult::NothingNew,
        }
    }

    pub fn flush(&mut self) {
        self.sent_previously.extend(self.sent_now.drain())
    }
}
