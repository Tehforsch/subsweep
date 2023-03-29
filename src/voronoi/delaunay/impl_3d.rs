use glam::DVec3;

use super::dimension::Dimension;
use super::dimension::DimensionTetra;
use super::dimension::DimensionTetraData;
use super::face_info::ConnectionData;
use super::face_info::FaceInfo;
use super::Delaunay;
use super::FaceIndex;
use super::FlipCheckData;
use super::PointIndex;
use super::TetraIndex;
use super::TetrasRequiringCheck;
use super::Triangulation;
use crate::voronoi::delaunay::dimension::DimensionFace;
use crate::voronoi::delaunay::PointKind;
use crate::voronoi::primitives::polygon3d::Polygon3d;
use crate::voronoi::primitives::tetrahedron::Tetrahedron;
use crate::voronoi::primitives::tetrahedron::TetrahedronData;
use crate::voronoi::primitives::triangle::IntersectionType;
use crate::voronoi::primitives::triangle::Triangle;
use crate::voronoi::primitives::triangle::TriangleData;
use crate::voronoi::ThreeD;

type Point = DVec3;
type Face = Triangle;
type FaceData = TriangleData<DVec3>;
type Tetra = Tetrahedron;
type TetraData = TetrahedronData;

impl Dimension for ThreeD {
    type Point = Point;
    type Face = Face;
    type FaceData = FaceData;
    type Tetra = Tetra;
    type TetraData = TetraData;
    type VoronoiFaceData = Polygon3d;
}

impl Triangulation<ThreeD> {
    fn insert_split_tetra(
        &mut self,
        p_a: PointIndex,
        p_b: PointIndex,
        p_c: PointIndex,
        p: PointIndex,
        f_a: FaceIndex,
        f_b: FaceIndex,
        f_c: FaceIndex,
        old_face: FaceInfo,
    ) -> TetraIndex {
        // Leave opposing data of the newly created faces
        // uninitialized for now, since we do not know the indices of
        // the other tetras before we have inserted them.
        self.insert_positively_oriented_tetra(Tetra {
            p1: p_a,
            p2: p_b,
            p3: p_c,
            p4: p,
            f1: FaceInfo {
                face: f_a,
                opposing: None,
            },
            f2: FaceInfo {
                face: f_b,
                opposing: None,
            },
            f3: FaceInfo {
                face: f_c,
                opposing: None,
            },
            f4: old_face,
        })
    }

    fn two_to_three_flip(
        &mut self,
        ta_index: TetraIndex,
        tb_index: TetraIndex,
        p1: PointIndex,
        p2: PointIndex,
        shared_face: FaceIndex,
    ) -> TetrasRequiringCheck {
        let ta = self.tetras.remove(ta_index).unwrap();
        let tb = self.tetras.remove(tb_index).unwrap();
        let shared_face = self.faces.remove(shared_face).unwrap();
        let f1 = self.faces.insert(Face {
            p1,
            p2,
            p3: shared_face.p1,
        });
        let f2 = self.faces.insert(Face {
            p1,
            p2,
            p3: shared_face.p2,
        });
        let f3 = self.faces.insert(Face {
            p1,
            p2,
            p3: shared_face.p3,
        });
        let mut make_tetra = |pa, pb, fa, fb, other_point| {
            let f1 = *ta.find_face_opposite(other_point);
            let f2 = *tb.find_face_opposite(other_point);
            self.insert_positively_oriented_tetra(Tetra {
                p1,
                p2,
                p3: pa,
                p4: pb,
                f1: f2,
                f2: f1,
                // Leave opposing uninitialized for now
                f3: FaceInfo {
                    face: fb,
                    opposing: None,
                },
                f4: FaceInfo {
                    face: fa,
                    opposing: None,
                },
            })
        };
        let t1 = make_tetra(shared_face.p2, shared_face.p3, f2, f3, shared_face.p1);
        let t2 = make_tetra(shared_face.p3, shared_face.p1, f3, f1, shared_face.p2);
        let t3 = make_tetra(shared_face.p1, shared_face.p2, f1, f2, shared_face.p3);
        // Set the connections between the newly created tetras
        self.tetras[t1].find_face_mut(f2).opposing = Some(ConnectionData {
            tetra: t3,
            point: shared_face.p1,
        });
        self.tetras[t1].find_face_mut(f3).opposing = Some(ConnectionData {
            tetra: t2,
            point: shared_face.p1,
        });
        self.tetras[t2].find_face_mut(f3).opposing = Some(ConnectionData {
            tetra: t1,
            point: shared_face.p2,
        });
        self.tetras[t2].find_face_mut(f1).opposing = Some(ConnectionData {
            tetra: t3,
            point: shared_face.p2,
        });
        self.tetras[t3].find_face_mut(f1).opposing = Some(ConnectionData {
            tetra: t2,
            point: shared_face.p3,
        });
        self.tetras[t3].find_face_mut(f2).opposing = Some(ConnectionData {
            tetra: t1,
            point: shared_face.p3,
        });
        [t1, t2, t3].into()
    }

    fn three_to_two_flip(
        &mut self,
        t1_index: TetraIndex,
        t2_index: TetraIndex,
        t3_index: TetraIndex,
        p1: PointIndex,
        p2: PointIndex,
        p3: PointIndex,
        shared_edge_p1: PointIndex,
        shared_edge_p2: PointIndex,
    ) -> TetrasRequiringCheck {
        let t1 = self.tetras.remove(t1_index).unwrap();
        let t2 = self.tetras.remove(t2_index).unwrap();
        let t3 = self.tetras.remove(t3_index).unwrap();
        // We need to remove the 3 inner faces shared by (t1, t2), (t2, t3) and (t3, t1) respectively
        // and then add a new face
        let f1 = t3.find_face_opposite(p1).face;
        let f2 = t3.find_face_opposite(p2).face;
        let f3 = t1.find_face_opposite(p1).face;
        debug_assert_eq!(f3, t2.find_face_opposite(p2).face);
        self.faces.remove(f1);
        self.faces.remove(f2);
        self.faces.remove(f3);
        let new_face = self.faces.insert(Face { p1, p2, p3 });

        let mut make_new_tetra = |contained_point, opposite_point| {
            let f1 = *t2.find_face_opposite(opposite_point);
            let f2 = *t1.find_face_opposite(opposite_point);
            let f3 = *t3.find_face_opposite(opposite_point);
            self.insert_positively_oriented_tetra(Tetra {
                p1,
                p2,
                p3,
                p4: contained_point,
                f1,
                f2,
                f3,
                f4: FaceInfo {
                    face: new_face,
                    opposing: None,
                },
            })
        };
        let ta = make_new_tetra(shared_edge_p1, shared_edge_p2);
        let tb = make_new_tetra(shared_edge_p2, shared_edge_p1);
        // Fix the connection data in the newly created tetra
        self.tetras[ta].find_face_mut(new_face).opposing = Some(ConnectionData {
            tetra: tb,
            point: shared_edge_p2,
        });
        self.tetras[tb].find_face_mut(new_face).opposing = Some(ConnectionData {
            tetra: ta,
            point: shared_edge_p1,
        });
        [ta, tb].into()
    }
}

impl Delaunay<ThreeD> for Triangulation<ThreeD> {
    fn make_positively_oriented_tetra(&mut self, tetra: Tetra) -> Tetra {
        let tetra_data = TetraData {
            p1: self.points[tetra.p1],
            p2: self.points[tetra.p2],
            p3: self.points[tetra.p3],
            p4: self.points[tetra.p4],
        };
        for (f, (pa, pb, pc)) in [
            (tetra.f1.face, (tetra.p2, tetra.p3, tetra.p4)),
            (tetra.f2.face, (tetra.p1, tetra.p3, tetra.p4)),
            (tetra.f3.face, (tetra.p1, tetra.p2, tetra.p4)),
            (tetra.f4.face, (tetra.p1, tetra.p2, tetra.p3)),
        ] {
            debug_assert!(self.faces[f].contains_point(pa));
            debug_assert!(self.faces[f].contains_point(pb));
            debug_assert!(self.faces[f].contains_point(pc));
        }
        if tetra_data.is_positively_oriented().unwrap() {
            tetra
        } else {
            Tetra {
                p1: tetra.p2,
                p2: tetra.p1,
                p3: tetra.p3,
                p4: tetra.p4,
                f1: tetra.f2,
                f2: tetra.f1,
                f3: tetra.f3,
                f4: tetra.f4,
            }
        }
    }

    fn split(&mut self, old_tetra_index: TetraIndex, point: PointIndex) -> Vec<TetraIndex> {
        let old_tetra = self.tetras.remove(old_tetra_index).unwrap();
        let f1 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p1,
            p3: old_tetra.p2,
        });
        let f2 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p1,
            p3: old_tetra.p3,
        });
        let f3 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p1,
            p3: old_tetra.p4,
        });
        let f4 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p2,
            p3: old_tetra.p3,
        });
        let f5 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p2,
            p3: old_tetra.p4,
        });
        let f6 = self.faces.insert(Face {
            p1: point,
            p2: old_tetra.p3,
            p3: old_tetra.p4,
        });
        let t1 = self.insert_split_tetra(
            old_tetra.p2,
            old_tetra.p3,
            old_tetra.p4,
            point,
            f6,
            f5,
            f4,
            old_tetra.f1,
        );
        let t2 = self.insert_split_tetra(
            old_tetra.p1,
            old_tetra.p3,
            old_tetra.p4,
            point,
            f6,
            f3,
            f2,
            old_tetra.f2,
        );
        let t3 = self.insert_split_tetra(
            old_tetra.p1,
            old_tetra.p2,
            old_tetra.p4,
            point,
            f5,
            f3,
            f1,
            old_tetra.f3,
        );
        let t4 = self.insert_split_tetra(
            old_tetra.p1,
            old_tetra.p2,
            old_tetra.p3,
            point,
            f4,
            f2,
            f1,
            old_tetra.f4,
        );
        self.set_opposing_in_new_tetra(t1, f6, t2, old_tetra.p1);
        self.set_opposing_in_new_tetra(t1, f5, t3, old_tetra.p1);
        self.set_opposing_in_new_tetra(t1, f4, t4, old_tetra.p1);

        self.set_opposing_in_new_tetra(t2, f6, t1, old_tetra.p2);
        self.set_opposing_in_new_tetra(t2, f3, t3, old_tetra.p2);
        self.set_opposing_in_new_tetra(t2, f2, t4, old_tetra.p2);

        self.set_opposing_in_new_tetra(t3, f5, t1, old_tetra.p3);
        self.set_opposing_in_new_tetra(t3, f3, t2, old_tetra.p3);
        self.set_opposing_in_new_tetra(t3, f1, t4, old_tetra.p3);

        self.set_opposing_in_new_tetra(t4, f4, t1, old_tetra.p4);
        self.set_opposing_in_new_tetra(t4, f2, t2, old_tetra.p4);
        self.set_opposing_in_new_tetra(t4, f1, t3, old_tetra.p4);

        [t1, t2, t3, t4].into()
    }

    fn flip(&mut self, check: FlipCheckData) -> TetrasRequiringCheck {
        // Two tetrahedra are flagged for flipping. There are three possible cases here, depending on the
        // intersection of the shared face (triangle) and the line between the two points opposite of the shared face.
        // 1. If the intersection point lies inside the triangle, we do a 2-to-3-flip, in which the two tetrahedra are replaced by three
        // 2. If the intersection point lies outside one of the edges, we take into account the neighbouring tetrahedron
        //    along that edge and do a 3-to-2 flip in which the three tetrahedra are converted to two.
        // 3. If the intersection point lies outside two edges, the flip can be skipped. This seems like magic
        //    but it can be shown that flipping the remaining violating edges will restore delaunayhood.
        // For more information see Springel (2009), doi:10.1111/j.1365-2966.2009.15715.x
        let t1 = &self.tetras[check.tetra];
        let shared_face = &self.faces[check.face];
        let opposing = t1.find_face(check.face).opposing.unwrap();
        let t2 = &self.tetras[opposing.tetra];
        // Obtain the two points opposite of the shared face
        let p1 = t1.find_point_opposite(check.face);
        let p2 = t2.find_point_opposite(check.face);
        let intersection_type = self
            .get_face_data(shared_face)
            .get_line_intersection_type(self.points[p1], self.points[p2])
            .unwrap_or_else(|_| {
                todo!("Handle case of degenerate intersection type (4-to-4 flip?)")
            });
        match intersection_type {
            IntersectionType::Inside => {
                self.two_to_three_flip(check.tetra, opposing.tetra, p1, p2, check.face)
            }
            IntersectionType::OutsideOneEdge(edge) => {
                let opposite_point = shared_face.get_point_opposite(edge);
                let (shared_face_p1, shared_face_p2) = shared_face.get_points_of(edge);
                let t3 = t1
                    .find_face_opposite(opposite_point)
                    .opposing
                    .unwrap()
                    .tetra;
                if t2
                    .find_face_opposite(opposite_point)
                    .opposing
                    .unwrap()
                    .tetra
                    == t3
                {
                    self.three_to_two_flip(
                        check.tetra,
                        opposing.tetra,
                        t3,
                        p1,
                        p2,
                        opposite_point,
                        shared_face_p1,
                        shared_face_p2,
                    )
                } else {
                    [].into()
                    // This is not documented in Springel 2009, but the Arepo code
                    // does nothing here.
                }
            }
            IntersectionType::OutsideTwoEdges(_, _) => [].into(),
        }
    }

    fn insert_basic_tetra(&mut self, tetra: TetraData) {
        debug_assert_eq!(self.tetras.len(), 0);
        let p1 = self.points.insert(tetra.p1);
        let p2 = self.points.insert(tetra.p2);
        let p3 = self.points.insert(tetra.p3);
        let p4 = self.points.insert(tetra.p4);
        self.point_kinds.insert(p1, PointKind::Outer);
        self.point_kinds.insert(p2, PointKind::Outer);
        self.point_kinds.insert(p3, PointKind::Outer);
        self.point_kinds.insert(p4, PointKind::Outer);
        self.insert_tetra_and_faces(p1, p2, p3, p4);
    }
}

impl Triangulation<ThreeD> {
    fn insert_tetra_and_faces(
        &mut self,
        pa: PointIndex,
        pb: PointIndex,
        pc: PointIndex,
        pd: PointIndex,
    ) -> TetraIndex {
        let fa = self.faces.insert(Face {
            p1: pb,
            p2: pc,
            p3: pd,
        });
        let fb = self.faces.insert(Face {
            p1: pa,
            p2: pc,
            p3: pd,
        });
        let fc = self.faces.insert(Face {
            p1: pa,
            p2: pb,
            p3: pd,
        });
        let fd = self.faces.insert(Face {
            p1: pa,
            p2: pb,
            p3: pc,
        });
        self.insert_positively_oriented_tetra(Tetra {
            p1: pa,
            p2: pb,
            p3: pc,
            p4: pd,
            f1: FaceInfo {
                face: fa,
                opposing: None,
            },
            f2: FaceInfo {
                face: fb,
                opposing: None,
            },
            f3: FaceInfo {
                face: fc,
                opposing: None,
            },
            f4: FaceInfo {
                face: fd,
                opposing: None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use bevy::utils::HashMap;

    use super::Tetra;
    use crate::voronoi::delaunay::dimension::DimensionFace;
    use crate::voronoi::delaunay::dimension::DimensionTetra;
    use crate::voronoi::delaunay::face_info::ConnectionData;
    use crate::voronoi::delaunay::face_info::FaceInfo;
    use crate::voronoi::delaunay::tests::check_opposing_faces_are_symmetric;
    use crate::voronoi::delaunay::tests::check_opposing_point_is_in_other_tetra;
    use crate::voronoi::delaunay::FaceList;
    use crate::voronoi::delaunay::PointIndex;
    use crate::voronoi::delaunay::PointList;
    use crate::voronoi::delaunay::TetraIndex;
    use crate::voronoi::delaunay::TetraList;
    use crate::voronoi::primitives::triangle::Triangle;
    use crate::voronoi::primitives::Point3d;
    use crate::voronoi::ThreeD;
    use crate::voronoi::Triangulation;

    fn insert_tetra_with_neighbours(
        t: &mut Triangulation<ThreeD>,
        neighbours: &[(TetraIndex, PointIndex)],
        p1: PointIndex,
        p2: PointIndex,
        p3: PointIndex,
        p4: PointIndex,
    ) -> TetraIndex {
        let mut insert_face = |face: Triangle| {
            let corresponding_neighbour = neighbours
                .iter()
                .map(|(tetra, point)| {
                    (
                        t.tetras[*tetra].find_face_opposite(*point).face,
                        ConnectionData {
                            tetra: *tetra,
                            point: *point,
                        },
                    )
                })
                .find(|(neighbour_face, _)| {
                    t.faces[*neighbour_face]
                        .points()
                        .all(|p| face.contains_point(p))
                });
            if let Some((face, connection_data)) = corresponding_neighbour {
                FaceInfo {
                    face: face,
                    opposing: Some(connection_data),
                }
            } else {
                FaceInfo {
                    face: t.faces.insert(face),
                    opposing: None,
                }
            }
        };
        let f1 = insert_face(Triangle {
            p1: p2,
            p2: p3,
            p3: p4,
        });
        let f2 = insert_face(Triangle {
            p1: p1,
            p2: p3,
            p3: p4,
        });
        let f3 = insert_face(Triangle {
            p1: p1,
            p2: p2,
            p3: p4,
        });
        let f4 = insert_face(Triangle {
            p1: p1,
            p2: p2,
            p3: p3,
        });
        t.insert_positively_oriented_tetra(Tetra {
            p1,
            p2,
            p3,
            p4,
            f1,
            f2,
            f3,
            f4,
        })
    }

    #[test]
    fn two_to_three_flip() {
        let mut point_list = PointList::<ThreeD>::default();
        let points = [
            Point3d::new(-0.3, -0.3, -1.0),
            Point3d::new(-1.0, -1.0, 0.0),
            Point3d::new(-1.0, 1.0, 0.0),
            Point3d::new(1.0, -1.0, 0.0),
            Point3d::new(-0.3, -0.3, 1.0),
        ];
        let points: Vec<_> = points.into_iter().map(|p| point_list.insert(p)).collect();

        let mut triangulation = Triangulation::<ThreeD> {
            tetras: TetraList::<ThreeD>::default(),
            faces: FaceList::<ThreeD>::default(),
            points: point_list,
            last_insertion_tetra: None,
            point_kinds: HashMap::default(),
        };
        let t1 = insert_tetra_with_neighbours(
            &mut triangulation,
            &[],
            points[0],
            points[1],
            points[2],
            points[3],
        );
        let t2 = insert_tetra_with_neighbours(
            &mut triangulation,
            &[(t1, points[0])],
            points[1],
            points[2],
            points[3],
            points[4],
        );
        let shared_face = triangulation.tetras[t1].find_face_opposite(points[0]).face;
        let tetras = triangulation.two_to_three_flip(t1, t2, points[0], points[4], shared_face);
        assert_eq!(triangulation.tetras.len(), 3);
        assert_eq!(triangulation.points.len(), 5);
        assert_eq!(triangulation.faces.len(), 9);
        let find_tetra_with = |p1, p2| {
            *tetras
                .iter()
                .find(|t| {
                    triangulation.tetras[**t].contains_point(p1)
                        && triangulation.tetras[**t].contains_point(p2)
                })
                .unwrap()
        };
        let t12 = find_tetra_with(points[1], points[2]);
        let t23 = find_tetra_with(points[2], points[3]);
        let t31 = find_tetra_with(points[3], points[1]);
        assert_eq!(
            triangulation.tetras[t12]
                .find_face_opposite(points[1])
                .opposing
                .unwrap(),
            ConnectionData {
                tetra: t23,
                point: points[3]
            }
        );
        assert_eq!(
            triangulation.tetras[t12]
                .find_face_opposite(points[2])
                .opposing
                .unwrap(),
            ConnectionData {
                tetra: t31,
                point: points[3]
            }
        );
        assert_eq!(
            triangulation.tetras[t23]
                .find_face_opposite(points[2])
                .opposing
                .unwrap(),
            ConnectionData {
                tetra: t31,
                point: points[1]
            }
        );
        assert_eq!(
            triangulation.tetras[t23]
                .find_face_opposite(points[3])
                .opposing
                .unwrap(),
            ConnectionData {
                tetra: t12,
                point: points[1]
            }
        );
        assert_eq!(
            triangulation.tetras[t31]
                .find_face_opposite(points[3])
                .opposing
                .unwrap(),
            ConnectionData {
                tetra: t12,
                point: points[2]
            }
        );
        assert_eq!(
            triangulation.tetras[t31]
                .find_face_opposite(points[1])
                .opposing
                .unwrap(),
            ConnectionData {
                tetra: t23,
                point: points[2]
            }
        );
        sanity_checks(&triangulation);
    }

    #[test]
    fn three_to_two_flip() {
        let mut point_list = PointList::<ThreeD>::default();
        let points = [
            Point3d::new(-0.3, -0.3, -1.0),
            Point3d::new(-1.0, -1.0, 0.0),
            Point3d::new(-1.0, 1.0, 0.0),
            Point3d::new(1.0, -1.0, 0.0),
            Point3d::new(-0.3, -0.3, 1.0),
        ];
        let points: Vec<_> = points.into_iter().map(|p| point_list.insert(p)).collect();

        let mut triangulation = Triangulation::<ThreeD> {
            tetras: TetraList::<ThreeD>::default(),
            faces: FaceList::<ThreeD>::default(),
            points: point_list,
            last_insertion_tetra: None,
            point_kinds: HashMap::default(),
        };
        let t1 = insert_tetra_with_neighbours(
            &mut triangulation,
            &[],
            points[0],
            points[4],
            points[1],
            points[2],
        );
        let t2 = insert_tetra_with_neighbours(
            &mut triangulation,
            &[(t1, points[1])],
            points[0],
            points[4],
            points[2],
            points[3],
        );
        let t3 = insert_tetra_with_neighbours(
            &mut triangulation,
            &[(t1, points[2]), (t2, points[2])],
            points[0],
            points[4],
            points[3],
            points[1],
        );
        let tetras = triangulation.three_to_two_flip(
            t1, t2, t3, points[1], points[3], points[2], points[0], points[4],
        );
        let ta = tetras[0];
        let tb = tetras[1];

        assert_eq!(
            triangulation.tetras[ta]
                .find_face_opposite(points[0])
                .opposing
                .unwrap()
                .tetra,
            tb
        );
        assert_eq!(
            triangulation.tetras[tb]
                .find_face_opposite(points[4])
                .opposing
                .unwrap()
                .tetra,
            ta
        );

        assert_eq!(triangulation.tetras.len(), 2);
        assert_eq!(triangulation.points.len(), 5);
        assert_eq!(triangulation.faces.len(), 7);
        sanity_checks(&triangulation);
    }

    fn sanity_checks(t: &Triangulation<ThreeD>) {
        check_opposing_faces_are_symmetric(t);
        check_opposing_point_is_in_other_tetra(t);
    }
}
