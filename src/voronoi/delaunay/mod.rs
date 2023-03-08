#[cfg(feature = "2d")]
mod impl_2d;
#[cfg(feature = "3d")]
mod impl_3d;

use bevy::prelude::Resource;

use super::tetra::ConnectionData;
use super::tetra::Tetra;
use super::tetra::TetraData;
use super::tetra::TetraFace;
use super::FaceIndex;
use super::FaceList;
use super::Point;
use super::PointIndex;
use super::PointList;
use super::TetraIndex;
use super::TetraList;

#[derive(Clone)]
pub struct FlipCheckData {
    tetra: TetraIndex,
    face: FaceIndex,
}

#[derive(Resource, Clone)]
pub struct DelaunayTriangulation {
    pub tetras: TetraList,
    pub faces: FaceList,
    pub points: PointList,
    pub to_check: Vec<FlipCheckData>,
}

impl DelaunayTriangulation {
    pub fn all_encompassing(points: &[Point]) -> DelaunayTriangulation {
        let initial_tetra_data = TetraData::all_encompassing(points);
        DelaunayTriangulation::from_basic_tetra(initial_tetra_data)
    }

    pub fn construct(points: &[Point]) -> (DelaunayTriangulation, Vec<PointIndex>) {
        let mut triangulation = DelaunayTriangulation::all_encompassing(points);
        let indices = points.iter().map(|p| triangulation.insert(*p)).collect();
        (triangulation, indices)
    }

    pub fn construct_from_iter(
        iter: impl Iterator<Item = Point>,
    ) -> (DelaunayTriangulation, Vec<PointIndex>) {
        let positions: Vec<_> = iter.collect();
        Self::construct(&positions)
    }

    pub fn find_containing_tetra(&self, point: Point) -> Option<TetraIndex> {
        self.tetras
            .iter()
            .find(|(_, tetra)| {
                let tetra_data = self.get_tetra_data(tetra);
                tetra_data.contains(point)
            })
            .map(|(index, _)| index)
    }

    pub fn insert(&mut self, point: Point) -> PointIndex {
        let t = self
            .find_containing_tetra(point)
            .expect("No tetra containing the point {point:?} found");
        let new_point_index = self.points.insert(point);
        self.split(t, new_point_index);
        while let Some(check) = self.to_check.pop() {
            self.flip_check(check);
        }
        new_point_index
    }

    pub fn insert_positively_oriented_tetra(&mut self, tetra: Tetra) -> TetraIndex {
        let tetra = self.make_positively_oriented_tetra(tetra);
        debug_assert!(self.get_tetra_data(&tetra).is_positively_oriented());
        self.tetras.insert(tetra)
    }

    fn set_opposing_in_existing_tetra(
        &mut self,
        old_tetra: TetraIndex,
        shared_face: TetraFace,
        new_tetra: TetraIndex,
        new_point: PointIndex,
    ) {
        if let Some(opposing) = shared_face.opposing {
            let existing_tetra = &mut self.tetras[opposing.tetra];
            let corresponding_face = existing_tetra.find_face_mut(shared_face.face);
            assert!(corresponding_face.opposing.unwrap().tetra == old_tetra);
            corresponding_face.opposing = Some(ConnectionData {
                tetra: new_tetra,
                point: new_point,
            });
        }
    }

    fn set_opposing_in_new_tetra(
        &mut self,
        new_tetra: TetraIndex,
        face: FaceIndex,
        tetra: TetraIndex,
        point: PointIndex,
    ) {
        self.tetras[new_tetra].find_face_mut(face).opposing = Some(ConnectionData { tetra, point });
    }

    fn circumcircle_contains_other_points(&self, tetra: TetraIndex) -> bool {
        let tetra = &self.tetras[tetra];
        let tetra_data = self.get_tetra_data(tetra);
        let other_point_contained = self
            .points
            .iter()
            .filter(|(point, _)| *point != tetra.p1 && *point != tetra.p2 && *point != tetra.p3)
            .find(|(_, point)| tetra_data.circumcircle_contains(**point))
            .is_some();
        other_point_contained
    }

    fn flip_check(&mut self, to_check: FlipCheckData) {
        if self.circumcircle_contains_other_points(to_check.tetra) {
            self.flip(to_check);
        }
    }

    fn from_basic_tetra(tetra: TetraData) -> DelaunayTriangulation {
        let mut triangulation = DelaunayTriangulation {
            tetras: TetraList::new(),
            faces: FaceList::new(),
            points: PointList::new(),
            to_check: vec![],
        };
        triangulation.insert_basic_tetra(tetra);
        triangulation
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::super::Point;
    use super::DelaunayTriangulation;
    use crate::config::NUM_DIMENSIONS;
    use crate::voronoi::tetra::TetraData;

    #[cfg(feature = "2d")]
    fn get_example_point_set() -> Vec<Point> {
        vec![
            Point::new(0.5, 0.5),
            Point::new(0.25, 0.5),
            Point::new(0.5, 0.25),
            Point::new(0.125, 0.5),
            Point::new(0.5, 0.125),
            Point::new(0.8, 0.1),
            Point::new(0.1, 0.8),
        ]
    }

    #[cfg(feature = "2d")]
    fn basic_tetra() -> TetraData {
        TetraData {
            p1: Point::new(0.0, 0.0),
            p2: Point::new(2.0, 0.0),
            p3: Point::new(0.0, 2.0),
        }
    }

    #[cfg(feature = "3d")]
    fn basic_tetra() -> TetraData {
        TetraData {
            p1: Point::new(0.0, 0.0, 0.0),
            p2: Point::new(2.0, 0.0, 0.0),
            p3: Point::new(0.0, 2.0, 0.0),
            p4: Point::new(0.0, 0.0, 2.0),
        }
    }

    #[cfg(feature = "3d")]
    fn get_example_point_set() -> Vec<Point> {
        vec![
            Point::new(0.5, 0.5, 0.5),
            Point::new(0.25, 0.5, 0.3),
            Point::new(0.5, 0.25, 0.4),
            Point::new(0.125, 0.5, 0.2),
            Point::new(0.5, 0.125, 0.35),
            Point::new(0.8, 0.1, 0.23),
            Point::new(0.1, 0.8, 0.7),
        ]
    }

    pub fn perform_check_on_each_level_of_construction(
        check: fn(&DelaunayTriangulation, usize) -> (),
    ) {
        let mut triangulation = DelaunayTriangulation::from_basic_tetra(basic_tetra());
        let points = get_example_point_set();
        for (num_points_inserted, point) in points.iter().enumerate() {
            check(&triangulation, num_points_inserted);
            triangulation.insert(*point);
        }
        check(&triangulation, points.len());
    }

    #[cfg(feature = "2d")]
    #[test]
    fn correct_number_of_objects() {
        perform_check_on_each_level_of_construction(|triangulation, num_points_inserted| {
            assert_eq!(triangulation.points.len(), 3 + num_points_inserted);
            assert_eq!(triangulation.tetras.len(), 1 + 2 * num_points_inserted);
            assert_eq!(triangulation.faces.len(), 3 + 3 * num_points_inserted);
        });
    }

    #[cfg(feature = "3d")]
    #[test]
    fn correct_number_of_objects() {
        perform_check_on_each_level_of_construction(|triangulation, num_points_inserted| {
            // In 3d we don't know how many faces there should be at any given level
            // because of 2-to-3 flips and 3-to-2 flips
            assert_eq!(triangulation.points.len(), 4 + num_points_inserted);
        });
    }

    #[test]
    fn first_insertion_creates_correct_number_of_opposing_faces() {
        perform_check_on_each_level_of_construction(|triangulation, num_points_inserted| {
            if num_points_inserted == 1 {
                // After the first insertion, we know that each tetra
                // should contain NUM_DIM faces which have an opposing
                // face (the `inner` ones).
                for (_, tetra) in triangulation.tetras.iter() {
                    assert_eq!(
                        tetra.iter_faces().filter_map(|face| face.opposing).count(),
                        NUM_DIMENSIONS
                    );
                }
            }
        });
    }

    #[test]
    fn opposing_faces_are_symmetric() {
        perform_check_on_each_level_of_construction(|triangulation, _| {
            for (i, t) in triangulation.tetras.iter() {
                for (face, opposing) in t
                    .iter_faces()
                    .filter_map(|face| face.opposing.map(|opp| (face, opp)))
                {
                    let opposing_tetra = &triangulation.tetras[opposing.tetra];
                    assert!(opposing_tetra
                        .iter_faces()
                        .filter_map(|face| face.opposing.map(|opp| (face, opp)))
                        .any(|(opposing_face, opposing_opposing)| {
                            opposing_opposing.tetra == i && face.face == opposing_face.face
                        }));
                }
            }
        });
    }

    #[test]
    fn opposing_faces_contain_valid_indices() {
        perform_check_on_each_level_of_construction(|triangulation, _| {
            for (_, tetra) in triangulation.tetras.iter() {
                for face in tetra.iter_faces() {
                    if let Some(opp) = face.opposing {
                        assert!(triangulation.tetras.contains(opp.tetra));
                    }
                }
            }
        });
    }

    #[test]
    fn faces_share_points_with_tetra() {
        perform_check_on_each_level_of_construction(|triangulation, _| {
            for (_, tetra) in triangulation.tetras.iter() {
                for face in tetra.iter_faces() {
                    let face = &triangulation.faces[face.face];
                    let num_shared_points = tetra
                        .iter_points()
                        .filter(|p| face.contains_point(**p))
                        .count();
                    assert_eq!(num_shared_points, NUM_DIMENSIONS);
                }
            }
        });
    }

    #[test]
    fn circumcircles_contain_no_additional_points() {
        perform_check_on_each_level_of_construction(|triangulation, _| {
            for (tetra, _) in triangulation.tetras.iter() {
                assert!(!triangulation.circumcircle_contains_other_points(tetra));
            }
        });
    }
}
