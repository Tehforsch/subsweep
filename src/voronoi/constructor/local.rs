use super::halo_iteration::RadiusSearch;
use super::halo_iteration::SearchResult;
use super::Constructor;
use super::SearchData;
use crate::voronoi::delaunay::Delaunay;
use crate::voronoi::utils::Extent;
use crate::voronoi::Cell;
use crate::voronoi::CellIndex;
use crate::voronoi::Dimension;
use crate::voronoi::DimensionCell;
use crate::voronoi::Point;
use crate::voronoi::Triangulation;

pub type LocalConstructor<D> = Constructor<D, Local>;

impl<D: Dimension> LocalConstructor<D>
where
    D: Dimension,
    Triangulation<D>: Delaunay<D>,
    Cell<D>: DimensionCell<Dimension = D>,
{
    pub fn only_delaunay<'a>(iter: impl Iterator<Item = &'a Point<D>> + 'a) -> Triangulation<D>
    where
        Point<D>: 'static,
    {
        Triangulation::construct_no_key(iter)
    }

    pub fn new(points: impl Iterator<Item = (CellIndex, Point<D>)>) -> Self {
        Self::construct_from_iter(points, Local)
    }
}

pub struct Local;

impl<D: Dimension> RadiusSearch<D> for Local {
    fn unique_radius_search(&mut self, _: Vec<SearchData<D>>) -> Vec<SearchResult<D>> {
        vec![]
    }

    fn determine_global_extent(&self) -> Option<Extent<Point<D>>> {
        None
    }
}
