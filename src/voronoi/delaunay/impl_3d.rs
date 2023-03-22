use glam::DVec3;

use super::dimension::Dimension;
use super::dimension::DimensionTetra;
use super::dimension::DimensionTetraData;
use super::face_info::ConnectionData;
use super::face_info::FaceInfo;
use super::Delaunay;
use super::DelaunayTriangulation;
use super::FaceIndex;
use super::FlipCheckData;
use super::PointIndex;
use super::TetraIndex;
use super::TetrasRequiringCheck;
use crate::voronoi::delaunay::dimension::DimensionFace;
use crate::voronoi::primitives::polygon3d::Polygon3d;
use crate::voronoi::primitives::tetrahedron::Tetrahedron;
use crate::voronoi::primitives::tetrahedron::TetrahedronData;
use crate::voronoi::primitives::triangle::IntersectionType;
use crate::voronoi::primitives::triangle::Triangle;
use crate::voronoi::primitives::triangle::TriangleData;
use crate::voronoi::utils::periodic_windows_2;
use crate::voronoi::utils::periodic_windows_3;
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

impl DelaunayTriangulation<ThreeD> {
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
        t1_index: TetraIndex,
        t2_index: TetraIndex,
        p1: PointIndex,
        p2: PointIndex,
        shared_face: FaceIndex,
    ) -> TetrasRequiringCheck {
        let t1 = self.tetras.remove(t1_index).unwrap();
        let t2 = self.tetras.remove(t2_index).unwrap();
        let shared_face = self.faces.remove(shared_face).unwrap();
        let points = [shared_face.p1, shared_face.p2, shared_face.p3];
        let new_faces: Vec<_> = points
            .into_iter()
            .map(|p| {
                let new_face = self.faces.insert(Face { p1, p2, p3: p });
                (new_face, p)
            })
            .collect();
        let new_tetras: Vec<_> = periodic_windows_3(&new_faces)
            .map(|((fa, pa), (fb, pb), (_, other_point))| {
                let f1 = t1.find_face_opposite(*other_point).clone();
                let f2 = t2.find_face_opposite(*other_point).clone();
                let t = self.insert_positively_oriented_tetra(Tetra {
                    p1,
                    p2,
                    p3: *pa,
                    p4: *pb,
                    f1: f2,
                    f2: f1,
                    // Leave opposing uninitialized for now
                    f3: FaceInfo {
                        face: *fb,
                        opposing: None,
                    },
                    f4: FaceInfo {
                        face: *fa,
                        opposing: None,
                    },
                });
                (t, *fa, *fb, *pa, *pb)
            })
            .collect();
        // Set the connections between the newly created tetras
        for ((t_left, _, f_left, _, p_left), (t, _, _, _, _), (t_right, f_right, _, p_right, _)) in
            periodic_windows_3(&new_tetras)
        {
            for (tetra, face, point) in [(t_left, f_left, p_left), (t_right, f_right, p_right)] {
                self.tetras[*t].find_face_mut(*face).opposing = Some(ConnectionData {
                    tetra: *tetra,
                    point: *point,
                });
            }
        }
        [new_tetras[0].0, new_tetras[1].0, new_tetras[2].0].into()
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

        let new_tetras_with_uninitialized_faces: Vec<_> =
            periodic_windows_2(&[shared_edge_p1, shared_edge_p2])
                .map(|(pa, pb)| {
                    let f1 = t2.find_face_opposite(*pb).clone();
                    let f2 = t1.find_face_opposite(*pb).clone();
                    let f3 = t3.find_face_opposite(*pb).clone();
                    let new_tetra = self.insert_positively_oriented_tetra(Tetra {
                        p1,
                        p2,
                        p3,
                        p4: *pa,
                        f1,
                        f2,
                        f3,
                        f4: FaceInfo {
                            face: new_face,
                            opposing: None,
                        },
                    });
                    (
                        new_tetra,
                        // Remember these to make the initialization of the connection data easier afterwards
                        new_face, *pb,
                    )
                })
                .collect();
        // Fix the connection data in the newly created tetra
        for ((t, uninitialized_face, _), (t_other, _, p_other)) in
            periodic_windows_2(&new_tetras_with_uninitialized_faces)
        {
            self.tetras[*t].find_face_mut(*uninitialized_face).opposing = Some(ConnectionData {
                tetra: *t_other,
                point: *p_other,
            })
        }
        [
            new_tetras_with_uninitialized_faces[0].0,
            new_tetras_with_uninitialized_faces[1].0,
        ]
        .into()
    }
}

impl Delaunay<ThreeD> for DelaunayTriangulation<ThreeD> {
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
        let opposing = t1.find_face(check.face).opposing.clone().unwrap();
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
        self.outer_points.push(p1);
        self.outer_points.push(p2);
        self.outer_points.push(p3);
        self.outer_points.push(p4);
        self.insert_tetra_and_faces(p1, p2, p3, p4);
    }
}

impl DelaunayTriangulation<ThreeD> {
    fn insert_tetra_and_faces(
        &mut self,
        p1: PointIndex,
        p2: PointIndex,
        p3: PointIndex,
        p4: PointIndex,
    ) -> TetraIndex {
        let f1 = self.faces.insert(Face {
            p1: p2,
            p2: p3,
            p3: p4,
        });
        let f2 = self.faces.insert(Face {
            p1: p1,
            p2: p3,
            p3: p4,
        });
        let f3 = self.faces.insert(Face {
            p1: p1,
            p2: p2,
            p3: p4,
        });
        let f4 = self.faces.insert(Face {
            p1: p1,
            p2: p2,
            p3: p3,
        });
        self.insert_positively_oriented_tetra(Tetra {
            p1,
            p2,
            p3,
            p4,
            f1: FaceInfo {
                face: f1,
                opposing: None,
            },
            f2: FaceInfo {
                face: f2,
                opposing: None,
            },
            f3: FaceInfo {
                face: f3,
                opposing: None,
            },
            f4: FaceInfo {
                face: f4,
                opposing: None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Tetra;
    use crate::voronoi::delaunay::dimension::DimensionFace;
    use crate::voronoi::delaunay::dimension::DimensionTetra;
    use crate::voronoi::delaunay::face_info::ConnectionData;
    use crate::voronoi::delaunay::face_info::FaceInfo;
    use crate::voronoi::delaunay::tests::check_opposing_faces_are_symmetric;
    use crate::voronoi::delaunay::FaceList;
    use crate::voronoi::delaunay::PointIndex;
    use crate::voronoi::delaunay::PointList;
    use crate::voronoi::delaunay::TetraIndex;
    use crate::voronoi::delaunay::TetraList;
    use crate::voronoi::primitives::triangle::Triangle;
    use crate::voronoi::primitives::Point3d;
    use crate::voronoi::DelaunayTriangulation;
    use crate::voronoi::ThreeD;

    fn insert_tetra_with_neighbours(
        t: &mut DelaunayTriangulation<ThreeD>,
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
                    opposing: Some(connection_data.clone()),
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
        let mut point_list = PointList::<ThreeD>::new();
        let points = [
            Point3d::new(-0.3, -0.3, -1.0),
            Point3d::new(-1.0, -1.0, 0.0),
            Point3d::new(-1.0, 1.0, 0.0),
            Point3d::new(1.0, -1.0, 0.0),
            Point3d::new(-0.3, -0.3, 1.0),
        ];
        let points: Vec<_> = points.into_iter().map(|p| point_list.insert(p)).collect();

        let mut triangulation = DelaunayTriangulation::<ThreeD> {
            tetras: TetraList::<ThreeD>::new(),
            faces: FaceList::<ThreeD>::new(),
            points: point_list,
            outer_points: vec![],
            last_insertion_tetra: None,
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
        triangulation.two_to_three_flip(t1, t2, points[0], points[4], shared_face);
        assert_eq!(triangulation.tetras.len(), 3);
        assert_eq!(triangulation.points.len(), 5);
        assert_eq!(triangulation.faces.len(), 9);
        sanity_checks(&triangulation);
    }

    #[test]
    fn three_to_two_flip() {
        let mut point_list = PointList::<ThreeD>::new();
        let points = [
            Point3d::new(-0.3, -0.3, -1.0),
            Point3d::new(-1.0, -1.0, 0.0),
            Point3d::new(-1.0, 1.0, 0.0),
            Point3d::new(1.0, -1.0, 0.0),
            Point3d::new(-0.3, -0.3, 1.0),
        ];
        let points: Vec<_> = points.into_iter().map(|p| point_list.insert(p)).collect();

        let mut triangulation = DelaunayTriangulation::<ThreeD> {
            tetras: TetraList::<ThreeD>::new(),
            faces: FaceList::<ThreeD>::new(),
            points: point_list,
            outer_points: vec![],
            last_insertion_tetra: None,
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
        triangulation.three_to_two_flip(
            t1, t2, t3, points[1], points[3], points[2], points[0], points[4],
        );

        assert_eq!(triangulation.tetras.len(), 2);
        assert_eq!(triangulation.points.len(), 5);
        assert_eq!(triangulation.faces.len(), 7);
        sanity_checks(&triangulation);
    }

    fn sanity_checks(t: &DelaunayTriangulation<ThreeD>) {
        check_opposing_faces_are_symmetric(t);
    }
}
