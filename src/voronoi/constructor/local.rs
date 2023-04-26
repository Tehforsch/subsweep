use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResults;
use super::SearchData;
use crate::communication::DataByRank;
use crate::communication::Rank;
use crate::dimension::Point;
use crate::extent::Extent;
use crate::voronoi::DDimension;

pub struct Local;

impl<D: DDimension> RadiusSearch<D> for Local {
    fn radius_search(&mut self, _: Vec<SearchData<D>>) -> DataByRank<SearchResults<D>> {
        DataByRank::empty()
    }

    fn determine_global_extent(&self) -> Option<Extent<Point<D>>> {
        None
    }

    fn everyone_finished(&mut self, _: usize) -> bool {
        true
    }

    fn rank(&self) -> Rank {
        0
    }

    fn num_points(&mut self) -> usize {
        0
    }
}
