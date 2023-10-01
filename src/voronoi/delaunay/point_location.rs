use std::cmp::Ordering;
use std::collections::BinaryHeap;

use ordered_float::OrderedFloat;

use super::dimension::DTetraData;
use super::Delaunay;
use super::Point;
use super::Tetra;
use super::TetraIndex;
use super::Triangulation;
use crate::hash_map::HashSet;
use crate::voronoi::delaunay::dimension::DTetra;
use crate::voronoi::DDimension;

#[derive(PartialEq, Eq)]
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

impl Ord for CheckData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn tetra_contains_point<D>(t: &Triangulation<D>, tetra: &Tetra<D>, point: Point<D>) -> bool
where
    D: DDimension,
    Triangulation<D>: Delaunay<D>,
{
    let tetra_data = t.get_tetra_data(tetra);
    tetra_data.contains(point, &t.extent)
}

fn find_breadth_first<D>(
    t: &Triangulation<D>,
    point: D::Point,
    first_to_check: TetraIndex,
) -> Option<TetraIndex>
where
    D: DDimension,
    Triangulation<D>: Delaunay<D>,
{
    let mut already_checked: HashSet<TetraIndex> = HashSet::default();
    let mut to_check: BinaryHeap<CheckData> = BinaryHeap::default();
    to_check.push(CheckData {
        tetra: first_to_check,
        heuristic_distance: OrderedFloat(0.0), // Heuristic doesn't matter for the first item anyways
    });
    already_checked.insert(first_to_check);
    let mut ts = vec![];
    while let Some(check) = to_check.pop() {
        let tetra = &t.tetras[check.tetra];
        ts.push(t.get_tetra_data(tetra));
        if tetra_contains_point(t, tetra, point) {
            return Some(check.tetra);
        } else {
            for face in tetra.faces() {
                if let Some(opp) = face.opposing {
                    if already_checked.insert(opp.tetra) {
                        let heuristic_distance = OrderedFloat(
                            t.get_tetra_data(&t.tetras[opp.tetra])
                                .distance_to_point(point),
                        );
                        to_check.push(CheckData {
                            heuristic_distance,
                            tetra: opp.tetra,
                        });
                    }
                }
            }
        }
    }
    None
}

pub fn find_containing_tetra<D>(t: &Triangulation<D>, point: D::Point) -> Option<TetraIndex>
where
    D: DDimension,
    Triangulation<D>: Delaunay<D>,
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
