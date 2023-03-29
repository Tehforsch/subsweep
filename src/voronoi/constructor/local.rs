use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResult;
use super::SearchData;
use crate::communication::DataByRank;
use crate::voronoi::utils::Extent;
use crate::voronoi::Dimension;
use crate::voronoi::Point;

pub struct Local;

impl<D: Dimension> RadiusSearch<D> for Local {
    fn unique_radius_search(&mut self, _: Vec<SearchData<D>>) -> DataByRank<Vec<SearchResult<D>>> {
        DataByRank::empty()
    }

    fn determine_global_extent(&self) -> Option<Extent<Point<D>>> {
        None
    }

    fn everyone_finished(&mut self, _: usize) -> bool {
        true
    }
}
