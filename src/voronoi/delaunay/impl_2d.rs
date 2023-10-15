use super::dimension::DDimension;
use super::dimension::DTetra;
use super::face_info::ConnectionData;
use super::face_info::FaceInfo;
use super::Delaunay;
use super::FaceIndex;
use super::PointIndex;
use super::PointKind;
use super::TetraIndex;
use super::TetrasRequiringCheck;
use super::Triangulation;
use crate::dimension::TwoD;
use crate::voronoi::delaunay::dimension::DFace;
use crate::voronoi::delaunay::dimension::DTetraData;
use crate::voronoi::delaunay::FlipCheckData;
use crate::voronoi::primitives::line::Line;
use crate::voronoi::primitives::line::LineData;
use crate::voronoi::primitives::triangle::TriangleData;
use crate::voronoi::primitives::triangle::TriangleWithFaces;
use crate::voronoi::primitives::Point2d;

type Face = Line;
// not needed for two d
type FaceData = LineData<Point2d>;
type Tetra = TriangleWithFaces;
type TetraData = TriangleData<Point2d>;

impl DDimension for TwoD {
    type Face = Face;
    type FaceData = FaceData;
    type Tetra = Tetra;
    type TetraData = TetraData;
    type VoronoiFaceData = ();
}

impl Triangulation<TwoD> {
    fn insert_split_tetra(
        &mut self,
        p_a: PointIndex,
        p_b: PointIndex,
        p: PointIndex,
        f_a: FaceIndex,
        f_a_flipped: bool,
        f_b: FaceIndex,
        f_b_flipped: bool,
        old_face: FaceInfo,
    ) -> TetraIndex {
        // Leave opposing data of the newly created faces
        // uninitialized for now, since we do not know the indices of
        // the other tetras before we have inserted them.
        self.insert_positively_oriented_tetra(Tetra {
            p1: p_a,
            p2: p_b,
            p3: p,
            f1: FaceInfo {
                face: f_a,
                opposing: None,
                flipped: f_a_flipped,
            },
            f2: FaceInfo {
                face: f_b,
                opposing: None,
                flipped: f_b_flipped,
            },
            f3: old_face,
        })
    }
}

impl Delaunay<TwoD> for Triangulation<TwoD> {
    fn assert_reasonable_tetra(&self, _tetra: &Tetra) {}

    fn make_positively_oriented_tetra(&mut self, tetra: Tetra) -> Tetra {
        let tetra_data = TetraData {
            p1: self.points[tetra.p1],
            p2: self.points[tetra.p2],
            p3: self.points[tetra.p3],
        };
        debug_assert!(self.faces[tetra.f1.face].contains_point(tetra.p2));
        debug_assert!(self.faces[tetra.f1.face].contains_point(tetra.p3));
        debug_assert!(self.faces[tetra.f2.face].contains_point(tetra.p1));
        debug_assert!(self.faces[tetra.f2.face].contains_point(tetra.p3));
        debug_assert!(self.faces[tetra.f3.face].contains_point(tetra.p1));
        debug_assert!(self.faces[tetra.f3.face].contains_point(tetra.p2));
        if tetra_data.is_positively_oriented(&self.extent) {
            tetra
        } else {
            Tetra {
                p1: tetra.p2,
                p2: tetra.p1,
                p3: tetra.p3,
                f1: tetra.f2,
                f2: tetra.f1,
                f3: tetra.f3,
            }
        }
    }

    fn split(&mut self, old_tetra_index: TetraIndex, point: PointIndex) -> TetrasRequiringCheck {
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
        let t1 = self.insert_split_tetra(
            old_tetra.p2,
            old_tetra.p3,
            point,
            f3,
            true,
            f2,
            false,
            old_tetra.f1,
        );
        let t2 = self.insert_split_tetra(
            old_tetra.p3,
            old_tetra.p1,
            point,
            f1,
            true,
            f3,
            false,
            old_tetra.f2,
        );
        let t3 = self.insert_split_tetra(
            old_tetra.p1,
            old_tetra.p2,
            point,
            f2,
            true,
            f1,
            false,
            old_tetra.f3,
        );
        self.set_opposing_in_new_tetra(t1, f3, t2, old_tetra.p1);
        self.set_opposing_in_new_tetra(t1, f2, t3, old_tetra.p1);
        self.set_opposing_in_new_tetra(t2, f3, t1, old_tetra.p2);
        self.set_opposing_in_new_tetra(t2, f1, t3, old_tetra.p2);
        self.set_opposing_in_new_tetra(t3, f1, t2, old_tetra.p3);
        self.set_opposing_in_new_tetra(t3, f2, t1, old_tetra.p3);
        [t1, t2, t3].into()
    }

    fn flip(&mut self, check: FlipCheckData) -> TetrasRequiringCheck {
        let old_tetra = self.tetras.remove(check.tetra).unwrap();
        let old_face = self.faces.remove(check.face).unwrap();
        let face = old_tetra.find_face(check.face);
        let opposing = face.opposing.unwrap();
        let opposing_old_tetra = self.tetras.remove(opposing.tetra).unwrap();
        let opposing_point = opposing.point;
        let check_point = old_tetra.find_point_opposite(check.face);
        let (p1, p2) = if face.flipped {
            (old_face.p2, old_face.p1)
        } else {
            (old_face.p1, old_face.p2)
        };
        let new_face = self.faces.insert(Face {
            p1: check_point,
            p2: opposing_point,
        });

        let f1_a = *opposing_old_tetra.find_face_opposite(p2);
        let f1_b = *old_tetra.find_face_opposite(p2);
        let f2_a = *opposing_old_tetra.find_face_opposite(p1);
        let f2_b = *old_tetra.find_face_opposite(p1);

        let t1 = Tetra {
            p1: check_point,
            p2: p1,
            p3: opposing_point,
            // Leave uninitialized for now
            f1: f1_a,
            f2: FaceInfo {
                face: new_face,
                opposing: None,
                flipped: true,
            },
            f3: f1_b,
        };
        let t2 = Tetra {
            p1: p2,
            p2: check_point,
            p3: opposing_point,
            // Leave uninitialized for now
            f1: FaceInfo {
                face: new_face,
                opposing: None,
                flipped: false,
            },
            f2: f2_a,
            f3: f2_b,
        };
        let t1 = self.insert_positively_oriented_tetra(t1);
        let t2 = self.insert_positively_oriented_tetra(t2);
        // Set previously uninitialized opposing data, now that we know the tetra indices
        self.tetras[t1].find_face_mut(new_face).opposing = Some(ConnectionData {
            tetra: t2,
            point: p2,
        });
        self.tetras[t2].find_face_mut(new_face).opposing = Some(ConnectionData {
            tetra: t1,
            point: p1,
        });
        [t1, t2].into()
    }

    fn insert_basic_tetra(&mut self, tetra: TetraData) {
        let pa = self.points.insert(tetra.p1);
        let pb = self.points.insert(tetra.p2);
        let pc = self.points.insert(tetra.p3);
        self.point_kinds.insert(pa, PointKind::Outer);
        self.point_kinds.insert(pb, PointKind::Outer);
        self.point_kinds.insert(pc, PointKind::Outer);
        let fa = self.faces.insert(Face { p1: pb, p2: pc });
        let fb = self.faces.insert(Face { p1: pc, p2: pa });
        let fc = self.faces.insert(Face { p1: pa, p2: pb });
        self.insert_positively_oriented_tetra(Tetra {
            p1: pa,
            p2: pb,
            p3: pc,
            f1: FaceInfo {
                face: fa,
                opposing: None,
                flipped: false,
            },
            f2: FaceInfo {
                face: fb,
                opposing: None,
                flipped: false,
            },
            f3: FaceInfo {
                face: fc,
                opposing: None,
                flipped: false,
            },
        });
    }
}
