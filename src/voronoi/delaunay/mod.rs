pub(crate) mod dimension;
pub(crate) mod face_info;

mod impl_2d;
mod impl_3d;

use bevy::prelude::Resource;
use derive_more::From;
use derive_more::Into;
use generational_arena::Index;

use self::dimension::Dimension;
use self::dimension::DimensionFace;
use self::dimension::DimensionTetra;
use self::dimension::DimensionTetraData;
use self::face_info::ConnectionData;
use super::indexed_arena::IndexedArena;

#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct TetraIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct FaceIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq, Hash)]
pub struct PointIndex(Index);

type Point<D> = <D as Dimension>::Point;
type Face<D> = <D as Dimension>::Face;
type FaceData<D> = <D as Dimension>::FaceData;
type Tetra<D> = <D as Dimension>::Tetra;
type TetraData<D> = <D as Dimension>::TetraData;

type TetraList<D> = IndexedArena<TetraIndex, Tetra<D>>;
type FaceList<D> = IndexedArena<FaceIndex, Face<D>>;
type PointList<D> = IndexedArena<PointIndex, Point<D>>;

#[derive(Clone)]
pub struct FlipCheckData {
    tetra: TetraIndex,
    face: FaceIndex,
}

#[derive(Resource, Clone)]
pub struct DelaunayTriangulation<D: Dimension> {
    pub tetras: TetraList<D>,
    pub faces: FaceList<D>,
    pub points: PointList<D>,
    pub to_check: Vec<FlipCheckData>,
}

pub trait Delaunay<D: Dimension> {
    fn make_positively_oriented_tetra(&mut self, tetra: Tetra<D>) -> Tetra<D>;
    fn split(&mut self, old_tetra_index: TetraIndex, point: PointIndex);
    fn flip(&mut self, check: FlipCheckData);
    fn insert_basic_tetra(&mut self, tetra: TetraData<D>);
}

impl<D: Dimension> DelaunayTriangulation<D>
where
    DelaunayTriangulation<D>: Delaunay<D>,
{
    pub fn all_encompassing(points: &[Point<D>]) -> Self {
        let initial_tetra_data = TetraData::<D>::all_encompassing(points);
        DelaunayTriangulation::from_basic_tetra(initial_tetra_data)
    }

    pub fn construct(points: &[Point<D>]) -> (Self, Vec<PointIndex>) {
        let mut triangulation = DelaunayTriangulation::all_encompassing(points);
        let indices = points.iter().map(|p| triangulation.insert(*p)).collect();
        (triangulation, indices)
    }

    pub fn construct_from_iter(iter: impl Iterator<Item = Point<D>>) -> (Self, Vec<PointIndex>) {
        let positions: Vec<_> = iter.collect();
        Self::construct(&positions)
    }

    fn from_basic_tetra(tetra: TetraData<D>) -> Self {
        let mut triangulation = DelaunayTriangulation {
            tetras: TetraList::<D>::new(),
            faces: FaceList::<D>::new(),
            points: PointList::<D>::new(),
            to_check: vec![],
        };
        triangulation.insert_basic_tetra(tetra);
        triangulation
    }

    pub fn get_tetra_data(&self, tetra: &Tetra<D>) -> TetraData<D> {
        tetra.points().map(|p| self.points[p]).collect()
    }

    pub fn get_face_data(&self, face: &Face<D>) -> FaceData<D> {
        face.points().map(|p| self.points[p]).collect()
    }

    pub fn find_containing_tetra(&self, point: Point<D>) -> Option<TetraIndex> {
        self.tetras
            .iter()
            .find(|(_, tetra)| {
                let tetra_data = self.get_tetra_data(tetra);
                tetra_data
                    .contains(point)
                    .unwrap_or_else(|_| todo!("Point wants to be inserted onto an edge."))
            })
            .map(|(index, _)| index)
    }

    pub fn insert(&mut self, point: Point<D>) -> PointIndex {
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

    fn update_connections_in_existing_tetra(&mut self, tetra_index: TetraIndex) {
        let tetra = &self.tetras[tetra_index];
        let new_connections: Vec<_> = tetra
            .faces_and_points()
            .filter_map(|(face, point)| {
                face.opposing.map(|opposing| {
                    (
                        opposing.tetra,
                        face.face,
                        ConnectionData {
                            tetra: tetra_index,
                            point: point,
                        },
                    )
                })
            })
            .collect();
        for (tetra, face, connection) in new_connections.into_iter() {
            self.tetras[tetra].find_face_mut(face).opposing = Some(connection);
        }
    }

    pub fn insert_positively_oriented_tetra(&mut self, tetra: Tetra<D>) -> TetraIndex {
        let tetra = self.make_positively_oriented_tetra(tetra);
        debug_assert!(self
            .get_tetra_data(&tetra)
            .is_positively_oriented()
            .unwrap());
        let tetra_index = self.tetras.insert(tetra);
        self.update_connections_in_existing_tetra(tetra_index);
        tetra_index
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
        let tetra = self.tetras.get(tetra);
        if let Some(tetra) = tetra {
            let tetra_data = self.get_tetra_data(tetra);
            let other_point_contained = self
                .points
                .iter()
                .filter(|(point, _)| !tetra.contains_point(*point))
                .find(|(_, point)| tetra_data.circumcircle_contains(**point).unwrap())
                .is_some();
            other_point_contained
        } else {
            // If the tetra has been deleted by now: Skip this check
            false
        }
    }

    fn flip_check(&mut self, to_check: FlipCheckData) {
        if self.circumcircle_contains_other_points(to_check.tetra) {
            self.flip(to_check);
        }
    }
}

#[cfg(test)]
#[generic_tests::define]
pub(super) mod tests {
    use super::dimension::Dimension;
    use super::dimension::DimensionFace;
    use super::dimension::DimensionTetra;
    use super::Delaunay;
    use super::DelaunayTriangulation;
    use crate::voronoi::primitives::Point2d;
    use crate::voronoi::primitives::Point3d;
    use crate::voronoi::ThreeD;
    use crate::voronoi::TwoD;

    #[instantiate_tests(<TwoD>)]
    mod two_d {}

    #[instantiate_tests(<ThreeD>)]
    mod three_d {}

    pub trait TestableDimension: Dimension {
        fn num() -> usize;
        fn get_example_point_set() -> Vec<Self::Point>;
        fn basic_tetra() -> Self::TetraData;

        fn number_of_tetras(num_inserted_points: usize) -> Option<usize>;
        fn number_of_faces(num_inserted_points: usize) -> Option<usize>;
        fn number_of_points(num_inserted_points: usize) -> Option<usize>;
    }

    impl TestableDimension for TwoD {
        fn number_of_tetras(num_inserted_points: usize) -> Option<usize> {
            Some(1 + 2 * num_inserted_points)
        }

        fn number_of_faces(num_inserted_points: usize) -> Option<usize> {
            Some(3 + 3 * num_inserted_points)
        }

        fn number_of_points(num_inserted_points: usize) -> Option<usize> {
            Some(3 + num_inserted_points)
        }

        fn num() -> usize {
            2
        }

        fn get_example_point_set() -> Vec<Self::Point> {
            vec![
                Point2d::new(0.5, 0.5),
                Point2d::new(0.25, 0.5),
                Point2d::new(0.5, 0.25),
                Point2d::new(0.125, 0.4),
                Point2d::new(0.3, 0.125),
                Point2d::new(0.8, 0.15),
                Point2d::new(0.9, 0.8),
            ]
        }

        fn basic_tetra() -> Self::TetraData {
            Self::TetraData {
                p1: Point2d::new(0.0, 0.0),
                p2: Point2d::new(2.0, 0.0),
                p3: Point2d::new(0.0, 2.0),
            }
        }
    }

    impl TestableDimension for ThreeD {
        fn num() -> usize {
            3
        }

        // In 3d we don't know how many tetras/faces there should be at any given level
        // because of 2-to-3 flips and 3-to-2 flips
        fn number_of_tetras(_: usize) -> Option<usize> {
            None
        }

        fn number_of_faces(_: usize) -> Option<usize> {
            None
        }

        fn number_of_points(num_inserted_points: usize) -> Option<usize> {
            Some(4 + num_inserted_points)
        }

        fn basic_tetra() -> Self::TetraData {
            Self::TetraData {
                p1: Point3d::new(0.0, 0.0, 0.0),
                p2: Point3d::new(2.0, 0.0, 0.0),
                p3: Point3d::new(0.0, 2.0, 0.0),
                p4: Point3d::new(0.0, 0.0, 2.0),
            }
        }

        fn get_example_point_set() -> Vec<Point3d> {
            vec![
                Point3d::new(0.5, 0.5, 0.5),
                Point3d::new(0.25, 0.55, 0.3),
                Point3d::new(0.5, 0.25, 0.4),
                Point3d::new(0.125, 0.53, 0.2),
                Point3d::new(0.5, 0.125, 0.35),
                Point3d::new(0.8, 0.1, 0.23),
                Point3d::new(0.1, 0.8, 0.7),
            ]
        }
    }

    fn perform_check_on_each_level_of_construction<D>(
        check: fn(&DelaunayTriangulation<D>, usize) -> (),
    ) where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        let mut triangulation = DelaunayTriangulation::from_basic_tetra(D::basic_tetra());
        let points = D::get_example_point_set();
        for (num_points_inserted, point) in points.iter().enumerate() {
            check(&triangulation, num_points_inserted);
            triangulation.insert(*point);
        }
        check(&triangulation, points.len());
    }

    #[test]
    fn correct_number_of_objects<D>()
    where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        perform_check_on_each_level_of_construction(|triangulation, num_inserted_points| {
            let assert_correct_number = |correct_value, value| {
                if let Some(correct_value) = correct_value {
                    assert_eq!(correct_value, value);
                }
            };
            assert_correct_number(
                D::number_of_tetras(num_inserted_points),
                triangulation.tetras.len(),
            );
            assert_correct_number(
                D::number_of_faces(num_inserted_points),
                triangulation.faces.len(),
            );
            assert_correct_number(
                D::number_of_points(num_inserted_points),
                triangulation.points.len(),
            );
        });
    }

    #[test]
    fn first_insertion_creates_correct_number_of_opposing_faces<D>()
    where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        perform_check_on_each_level_of_construction::<D>(|triangulation, num_points_inserted| {
            if num_points_inserted == 1 {
                // After the first insertion, we know that each tetra
                // should contain d faces which have an opposing
                // face (the `inner` ones).
                for (_, tetra) in triangulation.tetras.iter() {
                    assert_eq!(
                        tetra.faces().filter_map(|face| face.opposing).count(),
                        D::num()
                    );
                }
            }
        });
    }

    pub fn check_opposing_faces_are_symmetric<D>(triangulation: &DelaunayTriangulation<D>)
    where
        D: Dimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        for (i, t) in triangulation.tetras.iter() {
            for (face, opposing) in t
                .faces()
                .filter_map(|face| face.opposing.map(|opp| (face, opp)))
            {
                let opposing_tetra = &triangulation.tetras[opposing.tetra];
                assert!(opposing_tetra
                    .faces()
                    .filter_map(|face| face.opposing.map(|opp| (face, opp)))
                    .any(|(opposing_face, opposing_opposing)| {
                        opposing_opposing.tetra == i && face.face == opposing_face.face
                    }));
            }
        }
    }

    #[test]
    fn opposing_faces_are_symmetric<D>()
    where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        perform_check_on_each_level_of_construction::<D>(|triangulation, _| {
            check_opposing_faces_are_symmetric(triangulation)
        });
    }

    #[test]
    fn opposing_faces_contain_valid_indices<D>()
    where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        perform_check_on_each_level_of_construction::<D>(|triangulation, _| {
            for (_, tetra) in triangulation.tetras.iter() {
                for face in tetra.faces() {
                    if let Some(opp) = face.opposing {
                        assert!(triangulation.tetras.contains(opp.tetra));
                    }
                }
            }
        });
    }

    pub fn check_faces_share_points_with_tetra<D>(triangulation: &DelaunayTriangulation<D>)
    where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        for (_, tetra) in triangulation.tetras.iter() {
            for face in tetra.faces() {
                let face = &triangulation.faces[face.face];
                let num_shared_points = tetra.points().filter(|p| face.contains_point(*p)).count();
                assert_eq!(num_shared_points, D::num());
            }
        }
    }

    #[test]
    fn faces_share_points_with_tetra<D>()
    where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        perform_check_on_each_level_of_construction::<D>(|triangulation, _| {
            check_faces_share_points_with_tetra(triangulation);
        });
    }

    #[test]
    fn circumcircles_contain_no_additional_points<D>()
    where
        D: Dimension + TestableDimension,
        DelaunayTriangulation<D>: Delaunay<D>,
    {
        perform_check_on_each_level_of_construction::<D>(|triangulation, _| {
            for (tetra, _) in triangulation.tetras.iter() {
                assert!(!triangulation.circumcircle_contains_other_points(tetra));
            }
        });
    }
}
