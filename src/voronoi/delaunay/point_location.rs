use std::collections::HashSet;

use super::dimension::DimensionTetraData;
use super::Delaunay;
use super::DelaunayTriangulation;
use super::Point;
use super::Tetra;
use super::TetraIndex;
use crate::voronoi::delaunay::dimension::DimensionTetra;
use crate::voronoi::Dimension;

fn tetra_contains_point<D>(t: &DelaunayTriangulation<D>, tetra: &Tetra<D>, point: Point<D>) -> bool
where
    D: Dimension,
    DelaunayTriangulation<D>: Delaunay<D>,
{
    let tetra_data = t.get_tetra_data(tetra);
    tetra_data
        .contains(point)
        .unwrap_or_else(|_| todo!("Point wants to be inserted onto an edge."))
}

fn find_breadth_first<D>(
    t: &DelaunayTriangulation<D>,
    point: D::Point,
    first_to_check: TetraIndex,
) -> Option<TetraIndex>
where
    D: Dimension,
    DelaunayTriangulation<D>: Delaunay<D>,
{
    let mut already_checked: HashSet<TetraIndex> = HashSet::default();
    let mut to_check = vec![first_to_check];
    while let Some(tetra_index) = to_check.pop() {
        let tetra = &t.tetras[tetra_index];
        if tetra_contains_point(t, tetra, point) {
            return Some(tetra_index);
        } else {
            to_check.extend(
                tetra
                    .faces()
                    .filter_map(|face| face.opposing.map(|opp| opp.tetra))
                    .filter(|tetra| !already_checked.contains(tetra)),
            );
        }
        already_checked.insert(tetra_index);
    }
    None
}

pub fn find_containing_tetra<D>(t: &DelaunayTriangulation<D>, point: D::Point) -> Option<TetraIndex>
where
    D: Dimension,
    DelaunayTriangulation<D>: Delaunay<D>,
{
    if let Some(last_insertion_tetra) = t.last_insertion_tetra {
        find_breadth_first(t, point, last_insertion_tetra)
    } else {
        t.tetras
            .iter()
            .find(|(_, tetra)| tetra_contains_point(t, tetra, point))
            .map(|(index, _)| index)
    }
}
