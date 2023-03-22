use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashSet;

use ordered_float::OrderedFloat;

use super::dimension::DimensionTetraData;
use super::Delaunay;
use super::DelaunayTriangulation;
use super::Point;
use super::Tetra;
use super::TetraIndex;
use crate::voronoi::delaunay::dimension::DimensionTetra;
use crate::voronoi::primitives::Vector;
use crate::voronoi::Dimension;

#[derive(PartialEq, Eq, Ord)]
struct CheckData {
    heuristic_distance: OrderedFloat<f64>,
    tetra: TetraIndex,
}

impl PartialOrd for CheckData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse here because the binary heap is a max heap
        Some(
            self.heuristic_distance
                .cmp(&other.heuristic_distance)
                .reverse(),
        )
    }
}

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
    Point<D>: Vector,
{
    let mut already_checked: HashSet<TetraIndex> = HashSet::default();
    let mut to_check: BinaryHeap<CheckData> = BinaryHeap::default();
    to_check.push(CheckData {
        tetra: first_to_check,
        heuristic_distance: OrderedFloat(0.0), // Heuristic doesn't matter for the first item anyways
    });
    while let Some(check) = to_check.pop() {
        let tetra = &t.tetras[check.tetra];
        if tetra_contains_point(t, tetra, point) {
            return Some(check.tetra);
        } else {
            to_check.extend(
                tetra
                    .faces()
                    .filter_map(|face| face.opposing)
                    .filter(|opp| !already_checked.contains(&opp.tetra))
                    .map(|opp| CheckData {
                        // We take the opposing point here because its
                        // 1. the simplest
                        // 2. also possibly the most meaningful
                        heuristic_distance: OrderedFloat(t.points[opp.point].distance(point)),
                        tetra: opp.tetra,
                    }),
            );
        }
        already_checked.insert(check.tetra);
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
