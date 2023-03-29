use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResult;
use super::SearchData;
use crate::voronoi::utils::Extent;
use crate::voronoi::Dimension;
use crate::voronoi::Point;

pub struct Local;

impl<D: Dimension> RadiusSearch<D> for Local {
    fn unique_radius_search(&mut self, _: Vec<SearchData<D>>) -> Vec<SearchResult<D>> {
        vec![]
    }

    fn determine_global_extent(&self) -> Option<Extent<Point<D>>> {
        None
    }
}
