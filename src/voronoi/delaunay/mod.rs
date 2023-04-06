pub(crate) mod dimension;
pub(crate) mod face_info;
mod impl_2d;
mod impl_3d;
mod point_location;

use std::hash::Hash;

use bevy::prelude::Resource;
use derive_more::From;
use derive_more::Into;
use generational_arena::Index;

use self::dimension::DFace;
use self::dimension::DTetra;
use self::dimension::DTetraData;
use self::dimension::Dimension;
use self::face_info::ConnectionData;
use super::indexed_arena::IndexedArena;
use super::primitives::DVector;
use super::utils::get_extent;
use super::utils::Extent;
use crate::communication::Rank;
use crate::hash_map::BiMap;
use crate::hash_map::HashMap;

#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TetraIndex(pub Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct FaceIndex(pub Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq, Hash)]
pub struct PointIndex(pub Index);

type Point<D> = <D as Dimension>::Point;
type Face<D> = <D as Dimension>::Face;
type FaceData<D> = <D as Dimension>::FaceData;
type Tetra<D> = <D as Dimension>::Tetra;
type TetraData<D> = <D as Dimension>::TetraData;

type TetraList<D> = IndexedArena<TetraIndex, Tetra<D>>;
type FaceList<D> = IndexedArena<FaceIndex, Face<D>>;
type PointList<D> = IndexedArena<PointIndex, Point<D>>;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum PointKind {
    Inner,
    Outer,
    Halo(Rank),
}

type TetrasRequiringCheck = Vec<TetraIndex>;

#[derive(Clone)]
pub struct FlipCheckData {
    tetra: TetraIndex,
    face: FaceIndex,
}

// The magic of this is that the point involved in the check
// is always the newly inserted point. This data structure makes
// this explicit.
#[derive(Clone)]
struct FlipCheckStack {
    point: PointIndex,
    stack: Vec<TetraIndex>,
}

impl FlipCheckStack {
    fn pop(&mut self) -> Option<TetraIndex> {
        self.stack.pop()
    }

    fn extend(&mut self, t: Vec<TetraIndex>) {
        self.stack.extend(t);
    }
}

#[derive(Resource, Clone)]
pub struct Triangulation<D: Dimension> {
    pub tetras: TetraList<D>,
    pub faces: FaceList<D>,
    pub points: PointList<D>,
    pub(super) point_kinds: HashMap<PointIndex, PointKind>,
    last_insertion_tetra: Option<TetraIndex>,
}

pub trait Delaunay<D: Dimension> {
    fn make_positively_oriented_tetra(&mut self, tetra: Tetra<D>) -> Tetra<D>;
    fn split(&mut self, old_tetra_index: TetraIndex, point: PointIndex) -> TetrasRequiringCheck;
    fn flip(&mut self, check: FlipCheckData) -> TetrasRequiringCheck;
    fn insert_basic_tetra(&mut self, tetra: TetraData<D>);
}

impl<D: Dimension> Triangulation<D>
where
    Triangulation<D>: Delaunay<D>,
{
    fn construct<T: Hash + Clone + Eq>(
        mut points: Vec<(T, Point<D>)>,
        extent: &Extent<Point<D>>,
    ) -> (Self, BiMap<T, PointIndex>) {
        points.sort_by_key(|(_, p)| p.get_peano_hilbert_key(extent.min, extent.max));
        let mut triangulation = Self::all_encompassing(extent);
        let indices = points
            .iter()
            .map(|(name, p)| (name.clone(), triangulation.insert(*p, PointKind::Inner)))
            .collect();
        (triangulation, indices)
    }

    pub fn construct_from_iter_custom_extent<T: Hash + Clone + Eq>(
        iter: impl Iterator<Item = (T, Point<D>)>,
        extent: &Extent<Point<D>>,
    ) -> (Self, BiMap<T, PointIndex>) {
        let points: Vec<_> = iter.collect();
        Self::construct(points, extent)
    }

    pub fn construct_from_iter<T: Hash + Clone + Eq>(
        iter: impl Iterator<Item = (T, Point<D>)>,
    ) -> (Self, BiMap<T, PointIndex>) {
        let points: Vec<_> = iter.collect();
        let extent = get_extent(points.iter().map(|(_, p)| *p)).unwrap();
        Self::construct(points, &extent)
    }

    pub fn construct_no_key<'a>(points: impl Iterator<Item = &'a Point<D>> + 'a) -> Self
    where
        Point<D>: 'static,
    {
        let (t, _) = Self::construct_from_iter(points.into_iter().map(|p| ((), *p)));
        t
    }

    fn all_encompassing(extent: &Extent<Point<D>>) -> Self {
        let initial_tetra_data = TetraData::<D>::all_encompassing(extent);
        Triangulation::from_basic_tetra(initial_tetra_data)
    }

    fn from_basic_tetra(tetra: TetraData<D>) -> Self {
        let mut triangulation = Triangulation {
            tetras: TetraList::<D>::default(),
            faces: FaceList::<D>::default(),
            points: PointList::<D>::default(),
            last_insertion_tetra: None,
            point_kinds: HashMap::default(),
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
        point_location::find_containing_tetra(self, point)
    }

    /// Iterate over the inner points of the triangulation, i.e. every
    /// point that is not on the boundary of the all-encompassing
    /// tetra.  This only gives valid results if the
    /// triangulation was constructed via incremental insertion, not
    /// if it has been manually constructed from tetras, as is done in
    /// some of the test code.
    pub fn iter_non_boundary_points(&self) -> impl Iterator<Item = PointIndex> + '_ {
        self.points.iter().map(|(i, _)| i).filter(|p| {
            let kind = self.point_kinds[p];
            matches!(kind, PointKind::Inner | PointKind::Halo(_))
        })
    }

    fn insert_positively_oriented_tetra(&mut self, tetra: Tetra<D>) -> TetraIndex {
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
                            point,
                        },
                    )
                })
            })
            .collect();
        for (tetra, face, connection) in new_connections.into_iter() {
            self.tetras[tetra].find_face_mut(face).opposing = Some(connection);
        }
    }

    pub fn insert(&mut self, point: Point<D>, kind: PointKind) -> PointIndex {
        let t = self
            .find_containing_tetra(point)
            .expect("No tetra containing the point {point:?} found");
        let new_point_index = self.points.insert(point);
        self.point_kinds.insert(new_point_index, kind);
        let new_tetras = self.split(t, new_point_index);
        self.perform_flip_checks(new_point_index, new_tetras);
        new_point_index
    }

    fn perform_flip_checks(&mut self, new_point_index: PointIndex, tetras: TetrasRequiringCheck) {
        let mut stack = FlipCheckStack {
            point: new_point_index,
            stack: tetras,
        };
        while let Some(tetra) = stack.pop() {
            if !self.tetras.contains(tetra) {
                // In 3-to-2 flips, tetras are removed that might still be on the stack.
                // In this case we can just ignore this check.
                continue;
            }
            let check = FlipCheckData {
                tetra,
                face: self.tetras[tetra].find_face_opposite(stack.point).face,
            };
            self.flip_check(&mut stack, check);
            self.last_insertion_tetra = Some(tetra);
        }
    }

    fn flip_check(&mut self, stack: &mut FlipCheckStack, to_check: FlipCheckData) {
        if self.face_is_invalid(&to_check) {
            let new_tetras_to_check = self.flip(to_check);
            stack.extend(new_tetras_to_check);
        }
    }

    fn face_is_invalid(&self, to_check: &FlipCheckData) -> bool {
        let tetra = self.tetras.get(to_check.tetra);
        if let Some(tetra) = tetra {
            if let Some(opp) = tetra.find_face(to_check.face).opposing {
                return self.circumcircle_contains_point(tetra, opp.point);
            }
        } else {
            // If the tetra has been deleted by now: Skip this check
        }
        false
    }

    fn circumcircle_contains_point(&self, tetra: &Tetra<D>, point: PointIndex) -> bool {
        let tetra_data = self.get_tetra_data(tetra);
        tetra_data
            .circumcircle_contains(self.points[point])
            .unwrap()
    }
}

#[cfg(test)]
#[generic_tests::define]
pub(super) mod tests {
    use super::dimension::DFace;
    use super::dimension::DTetra;
    use super::dimension::Dimension;
    use super::Delaunay;
    use super::PointKind;
    use super::Triangulation;
    use crate::voronoi::test_utils::TestDimension;
    use crate::voronoi::utils::get_extent;
    use crate::voronoi::ThreeD;
    use crate::voronoi::TwoD;

    #[instantiate_tests(<TwoD>)]
    mod two_d {}

    #[instantiate_tests(<ThreeD>)]
    mod three_d {}

    pub fn perform_triangulation_check_on_each_level_of_construction<D>(
        check: impl Fn(&Triangulation<D>, usize) -> (),
    ) where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        let points = D::get_example_point_set(0);
        let extent = get_extent(points.iter().copied()).unwrap();
        let mut triangulation = Triangulation::all_encompassing(&extent);
        for (num_points_inserted, point) in points.iter().enumerate() {
            check(&triangulation, num_points_inserted);
            triangulation.insert(*point, PointKind::Inner);
        }
        check(&triangulation, points.len());
    }

    #[test]
    fn correct_number_of_objects<D>()
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction(
            |triangulation, num_inserted_points| {
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
            },
        );
    }

    #[test]
    fn first_insertion_creates_correct_number_of_opposing_faces<D>()
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction::<D>(
            |triangulation, num_points_inserted| {
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
            },
        );
    }

    /// This checks that the "opposing" point in any face of a tetra t
    /// is not part of the t itself (which is a trivial requirement, but
    /// necessary nonetheless)
    pub fn check_opposing_point_is_in_other_tetra<D>(triangulation: &Triangulation<D>)
    where
        D: Dimension,
        Triangulation<D>: Delaunay<D>,
    {
        for (_, tetra) in triangulation.tetras.iter() {
            for face in tetra.faces() {
                if let Some(opp) = face.opposing {
                    assert!(!tetra.contains_point(opp.point));
                }
            }
        }
    }

    #[test]
    fn opposing_point_is_in_other_tetra<D>()
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction::<D>(|triangulation, _| {
            check_opposing_point_is_in_other_tetra(triangulation)
        });
    }

    pub fn check_opposing_faces_are_symmetric<D>(triangulation: &Triangulation<D>)
    where
        D: Dimension,
        Triangulation<D>: Delaunay<D>,
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
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction::<D>(|triangulation, _| {
            check_opposing_faces_are_symmetric(triangulation)
        });
    }

    #[test]
    fn opposing_faces_contain_valid_indices<D>()
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction::<D>(|triangulation, _| {
            for (_, tetra) in triangulation.tetras.iter() {
                for face in tetra.faces() {
                    if let Some(opp) = face.opposing {
                        assert!(triangulation.tetras.contains(opp.tetra));
                    }
                }
            }
        });
    }

    pub fn check_faces_share_points_with_tetra<D>(triangulation: &Triangulation<D>)
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
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
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction::<D>(|triangulation, _| {
            check_faces_share_points_with_tetra(triangulation);
        });
    }

    #[test]
    fn global_delaunayhood<D>()
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction::<D>(|triangulation, _| {
            for (_, tetra) in triangulation.tetras.iter() {
                for (p, _) in triangulation.points.iter() {
                    if !tetra.contains_point(p) {
                        assert!(!triangulation.circumcircle_contains_point(tetra, p));
                    }
                }
            }
        });
    }

    #[test]
    fn local_delaunayhood<D>()
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction::<D>(|triangulation, _| {
            for (_, tetra) in triangulation.tetras.iter() {
                for face in tetra.faces() {
                    if let Some(opp) = face.opposing {
                        assert!(!triangulation.circumcircle_contains_point(tetra, opp.point));
                    }
                }
            }
        });
    }

    #[test]
    fn inner_points_contains_right_number_of_points<D>()
    where
        D: Dimension + TestDimension,
        Triangulation<D>: Delaunay<D>,
    {
        perform_triangulation_check_on_each_level_of_construction::<D>(
            |triangulation, num_inserted| {
                let num_inner_points = triangulation.iter_non_boundary_points().count();
                assert_eq!(num_inner_points, num_inserted);
            },
        );
    }
}
