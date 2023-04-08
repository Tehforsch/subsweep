use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResults;
use super::SearchData;
use crate::communication::DataByRank;
use crate::voronoi::utils::Extent;
use crate::voronoi::DDimension;
use crate::voronoi::Point;

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
}
