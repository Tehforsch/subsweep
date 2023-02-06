use bevy::prelude::Resource;
use derive_more::From;
use derive_more::Into;
use generational_arena::Index;

use self::face::Face;
use self::indexed_arena::IndexedArena;
use self::tetra::OtherTetraInfo;
use self::tetra::Tetra;
use self::tetra::TetraData;
use self::tetra::TetraFace;

mod face;
mod indexed_arena;
mod tetra;

#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct TetraIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct FaceIndex(Index);
#[derive(Debug, Clone, Copy, From, Into, PartialEq, Eq)]
pub struct PointIndex(Index);

#[cfg(feature = "2d")]
pub type Point = glam::DVec2;
#[cfg(feature = "3d")]
pub type Point = glam::DVec3;

type TetraList = IndexedArena<TetraIndex, Tetra>;
type FaceList = IndexedArena<FaceIndex, Face>;
type PointList = IndexedArena<PointIndex, Point>;

pub struct FlipCheckData {
    tetra: TetraIndex,
}

#[derive(Resource)]
pub struct DelaunayTriangulation {
    pub tetras: TetraList,
    pub faces: FaceList,
    pub points: PointList,
    pub to_check: Vec<FlipCheckData>,
}

impl DelaunayTriangulation {
    pub fn all_encompassing(points: &[Point]) -> DelaunayTriangulation {
        let initial_tetra_data = get_all_encompassing_tetra(points);
        let mut points = PointList::new();
        let p1 = points.insert(initial_tetra_data.p1);
        let p2 = points.insert(initial_tetra_data.p2);
        let p3 = points.insert(initial_tetra_data.p3);
        #[cfg(not(feature = "2d"))]
        let p4 = points.insert(initial_tetra_data.p4);
        let mut faces = FaceList::new();
        let f1 = TetraFace {
            face: faces.insert(Face { p1: p2, p2: p3 }),
            opposing: None,
        };
        let f2 = TetraFace {
            face: faces.insert(Face { p1: p3, p2: p1 }),
            opposing: None,
        };
        let f3 = TetraFace {
            face: faces.insert(Face { p1: p1, p2: p2 }),
            opposing: None,
        };
        let mut tetras = TetraList::new();
        tetras.insert(Tetra {
            p1,
            p2,
            p3,
            f1,
            f2,
            f3,
            #[cfg(not(feature = "2d"))]
            p4,
        });
        DelaunayTriangulation {
            tetras,
            faces: faces,
            to_check: vec![],
            points,
        }
    }

    pub fn construct(points: &[Point]) -> Self {
        let mut constructor = DelaunayTriangulation::all_encompassing(points);
        for p in points {
            constructor.insert(*p);
        }
        constructor
    }

    fn get_tetra_data(&self, tetra: &Tetra) -> TetraData {
        TetraData {
            p1: self.points[tetra.p1],
            p2: self.points[tetra.p2],
            p3: self.points[tetra.p3],
        }
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

    pub fn insert(&mut self, point: Point) {
        let t = self
            .find_containing_tetra(point)
            .expect("No tetra containing the point {point:?} found");
        let new_point_index = self.points.insert(point);
        self.split(t, new_point_index);
        while let Some(check) = self.to_check.pop() {
            self.flip_check(check);
        }
    }

    fn set_opposing_in_existing_tetra(
        &mut self,
        face: TetraFace,
        new_tetra: TetraIndex,
        new_point: PointIndex,
        old_tetra_index: TetraIndex,
    ) {
        if let Some(opposing) = face.opposing {
            let existing_tetra = &mut self.tetras[opposing.tetra];
            let corresponding_face = existing_tetra.find_face_mut(face.face);
            assert!(corresponding_face.opposing.unwrap().tetra == old_tetra_index);
            corresponding_face.opposing = Some(OtherTetraInfo {
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
        self.tetras[new_tetra].find_face_mut(face).opposing = Some(OtherTetraInfo { tetra, point });
    }

    fn create_positively_oriented_tetra(
        &self,
        p1: PointIndex,
        p2: PointIndex,
        p3: PointIndex,
        f1: TetraFace,
        f2: TetraFace,
        f3: TetraFace,
    ) -> Tetra {
        let tetra_data = TetraData {
            p1: self.points[p1],
            p2: self.points[p2],
            p3: self.points[p3],
        };
        if tetra_data.is_positively_oriented() {
            Tetra {
                p1,
                p2,
                p3,
                f1,
                f2,
                f3,
            }
        } else {
            Tetra {
                p1: p2,
                p2: p1,
                p3,
                f1: f2,
                f2: f1,
                f3,
            }
        }
    }

    fn make_tetra(
        &mut self,
        p: PointIndex,
        p_a: PointIndex,
        p_b: PointIndex,
        f1: FaceIndex,
        f2: FaceIndex,
        old_face: TetraFace,
    ) -> TetraIndex {
        // Leave opposing data of the newly created faces
        // uninitialized for now, since we do not know the indices of
        // the other tetras before we have inserted them.
        self.tetras.insert(self.create_positively_oriented_tetra(
            p_a,
            p_b,
            p,
            TetraFace {
                face: f1,
                opposing: None,
            },
            TetraFace {
                face: f2,
                opposing: None,
            },
            TetraFace {
                face: old_face.face,
                opposing: old_face.opposing,
            },
        ))
    }

    fn split(&mut self, old_tetra_index: TetraIndex, point: PointIndex) {
        let old_tetra = self.tetras.remove(old_tetra_index).unwrap();
        let f1 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p1,
        });
        let f2 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p2,
        });
        let f3 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p3,
        });
        let t1 = self.make_tetra(point, old_tetra.p2, old_tetra.p3, f3, f2, old_tetra.f1);
        let t2 = self.make_tetra(point, old_tetra.p3, old_tetra.p1, f1, f3, old_tetra.f2);
        let t3 = self.make_tetra(point, old_tetra.p1, old_tetra.p2, f2, f1, old_tetra.f3);
        self.set_opposing_in_new_tetra(t1, f3, t2, old_tetra.p1);
        self.set_opposing_in_new_tetra(t1, f2, t3, old_tetra.p1);
        self.set_opposing_in_new_tetra(t2, f3, t1, old_tetra.p1);
        self.set_opposing_in_new_tetra(t2, f1, t3, old_tetra.p1);
        self.set_opposing_in_new_tetra(t3, f1, t2, old_tetra.p1);
        self.set_opposing_in_new_tetra(t3, f2, t1, old_tetra.p1);
        self.set_opposing_in_existing_tetra(old_tetra.f1, t1, point, old_tetra_index);
        self.set_opposing_in_existing_tetra(old_tetra.f2, t2, point, old_tetra_index);
        self.set_opposing_in_existing_tetra(old_tetra.f3, t3, point, old_tetra_index);
        for (tetra, _face) in [(t1, old_tetra.f1), (t2, old_tetra.f2), (t3, old_tetra.f3)] {
            self.to_check.push(FlipCheckData { tetra });
        }
    }

    fn circumcircle_contains_other_points(&self, tetra: TetraIndex) -> bool {
        let tetra = &self.tetras[tetra];
        let tetra_data = self.get_tetra_data(tetra);
        let other_point_contained = self
            .points
            .iter()
            .find(|(point, _)| *point != tetra.p1 && *point != tetra.p2 && *point != tetra.p3)
            .map(|(_, point)| tetra_data.circumcircle_contains(*point))
            .is_some();
        other_point_contained
    }

    fn flip(&mut self, _check: FlipCheckData) {
        // todo!()
    }

    fn flip_check(&mut self, to_check: FlipCheckData) {
        if self.circumcircle_contains_other_points(to_check.tetra) {
            self.flip(to_check);
        }
    }
}

#[cfg(feature = "2d")]
fn get_all_encompassing_tetra(points: &[Point]) -> TetraData {
    let (min, max) = get_min_and_max(points).unwrap();
    // An overshooting factor for numerical safety
    let alpha = 1.00;
    let p1 = min - (max - min) * alpha;
    let p2 = Point::new(min.x, max.y + (max.y - min.y) * (1.0 + alpha));
    let p3 = Point::new(max.x + (max.x - min.x) * (1.0 + alpha), min.y);
    TetraData { p1, p2, p3 }
}

fn get_min_and_max(points: &[Point]) -> Option<(Point, Point)> {
    let mut min = None;
    let mut max = None;
    let update_min = |min: &mut Option<Point>, pos: Point| {
        if let Some(ref mut min) = min {
            *min = min.min(pos);
        } else {
            *min = Some(pos);
        }
    };
    let update_max = |max: &mut Option<Point>, pos: Point| {
        if let Some(ref mut max) = max {
            *max = max.max(pos);
        } else {
            *max = Some(pos);
        }
    };
    for p in points {
        update_min(&mut min, *p);
        update_max(&mut max, *p);
    }
    Some((min?, max?))
}

#[cfg(feature = "2d")]
#[cfg(test)]
mod tests {
    use super::face::Face;
    use super::tetra::Tetra;
    use super::tetra::TetraFace;
    use super::DelaunayTriangulation;
    use super::FaceList;
    use super::Point;
    use super::PointList;
    use super::TetraList;

    #[test]
    fn insertion_creates_sane_triangulation() {
        let mut triangulation = get_basic_triangle();
        for i in 0..10 {
            triangulation.insert(Point::new(0.5, 0.5 / 2f64.powf(i as f64)));
            assert_eq!(triangulation.points.len(), 4 + i);
            assert_eq!(triangulation.tetras.len(), 3 + 2 * i);
            assert_eq!(triangulation.faces.len(), 6 + 3 * i);
            check_opposing_faces_are_symmetric(&triangulation);
            check_opposing_faces_contain_valid_indices(&triangulation);
            check_faces_share_points_with_tetra(&triangulation);
            if i == 0 {
                // After the first insertion, we know that each tetra
                // should contain two faces which have an opposing
                // face (the `inner` ones).
                for (_, tetra) in triangulation.tetras.iter() {
                    assert_eq!(
                        tetra.iter_faces().filter_map(|face| face.opposing).count(),
                        2
                    );
                }
            }
        }
    }

    fn get_basic_triangle() -> DelaunayTriangulation {
        let mut points = PointList::new();
        let mut faces = FaceList::new();
        let mut tetras = TetraList::new();
        let p1 = points.insert(Point::new(0.0, 0.0));
        let p2 = points.insert(Point::new(2.0, 0.0));
        let p3 = points.insert(Point::new(0.0, 2.0));
        let f1 = faces.insert(Face { p1: p2, p2: p3 });
        let f2 = faces.insert(Face { p1: p3, p2: p1 });
        let f3 = faces.insert(Face { p1: p1, p2: p2 });
        tetras.insert(Tetra {
            p1,
            p2,
            p3,
            f1: TetraFace {
                face: f1,
                opposing: None,
            },
            f2: TetraFace {
                face: f2,
                opposing: None,
            },
            f3: TetraFace {
                face: f3,
                opposing: None,
            },
        });

        DelaunayTriangulation {
            tetras: tetras,
            faces: faces,
            points: points,
            to_check: vec![],
        }
    }

    fn check_opposing_faces_are_symmetric(triangulation: &DelaunayTriangulation) {
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
    }

    fn check_opposing_faces_contain_valid_indices(triangulation: &DelaunayTriangulation) {
        for (_, tetra) in triangulation.tetras.iter() {
            for face in tetra.iter_faces() {
                if let Some(opp) = face.opposing {
                    assert!(triangulation.tetras.contains(opp.tetra));
                }
            }
        }
    }

    fn check_faces_share_points_with_tetra(triangulation: &DelaunayTriangulation) {
        for (_, tetra) in triangulation.tetras.iter() {
            for face in tetra.iter_faces() {
                let face = &triangulation.faces[face.face];
                for p in [face.p1, face.p2] {
                    assert!(tetra.p1 == p || tetra.p2 == p || tetra.p3 == p);
                }
            }
        }
    }
}
