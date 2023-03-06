use super::DelaunayTriangulation;
use super::FlipCheckData;
use crate::voronoi::face::Face;
use crate::voronoi::tetra::Tetra;
use crate::voronoi::tetra::TetraData;
use crate::voronoi::tetra::TetraFace;
use crate::voronoi::FaceIndex;
use crate::voronoi::PointIndex;
use crate::voronoi::TetraIndex;

impl DelaunayTriangulation {
    pub fn get_tetra_data(&self, tetra: &Tetra) -> TetraData {
        TetraData {
            p1: self.points[tetra.p1],
            p2: self.points[tetra.p2],
            p3: self.points[tetra.p3],
            p4: self.points[tetra.p4],
        }
    }

    fn make_tetra(
        &mut self,
        p_a: PointIndex,
        p_b: PointIndex,
        p_c: PointIndex,
        p: PointIndex,
        f_a: FaceIndex,
        f_b: FaceIndex,
        f_c: FaceIndex,
        old_face: TetraFace,
    ) -> TetraIndex {
        // Leave opposing data of the newly created faces
        // uninitialized for now, since we do not know the indices of
        // the other tetras before we have inserted them.
        self.insert_positively_oriented_tetra(
            p_a,
            p_b,
            p_c,
            p,
            TetraFace {
                face: f_a,
                opposing: None,
            },
            TetraFace {
                face: f_b,
                opposing: None,
            },
            TetraFace {
                face: f_c,
                opposing: None,
            },
            old_face,
        )
    }

    fn insert_positively_oriented_tetra(
        &mut self,
        p1: PointIndex,
        p2: PointIndex,
        p3: PointIndex,
        p4: PointIndex,
        f1: TetraFace,
        f2: TetraFace,
        f3: TetraFace,
        f4: TetraFace,
    ) -> TetraIndex {
        let tetra_data = TetraData {
            p1: self.points[p1],
            p2: self.points[p2],
            p3: self.points[p3],
            p4: self.points[p4],
        };
        for (f, (pa, pb, pc)) in [
            (f1.face, (p2, p3, p4)),
            (f2.face, (p1, p3, p4)),
            (f3.face, (p1, p2, p4)),
            (f4.face, (p1, p2, p3)),
        ] {
            debug_assert!(self.faces[f].contains_point(pa));
            debug_assert!(self.faces[f].contains_point(pb));
            debug_assert!(self.faces[f].contains_point(pc));
        }
        let tetra = if tetra_data.is_positively_oriented() {
            Tetra {
                p1,
                p2,
                p3,
                p4,
                f1,
                f2,
                f3,
                f4,
            }
        } else {
            Tetra {
                p1: p2,
                p2: p1,
                p3,
                p4,
                f1: f2,
                f2: f1,
                f3,
                f4,
            }
        };
        debug_assert!(self.get_tetra_data(&tetra).is_positively_oriented());
        self.tetras.insert(tetra)
    }

    pub(super) fn split(&mut self, old_tetra_index: TetraIndex, point: PointIndex) {
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
        let t1 = self.make_tetra(
            old_tetra.p2,
            old_tetra.p3,
            old_tetra.p4,
            point,
            f6,
            f5,
            f4,
            old_tetra.f1,
        );
        let t2 = self.make_tetra(
            old_tetra.p1,
            old_tetra.p3,
            old_tetra.p4,
            point,
            f6,
            f3,
            f2,
            old_tetra.f2,
        );
        let t3 = self.make_tetra(
            old_tetra.p1,
            old_tetra.p2,
            old_tetra.p4,
            point,
            f5,
            f3,
            f1,
            old_tetra.f3,
        );
        let t4 = self.make_tetra(
            old_tetra.p1,
            old_tetra.p2,
            old_tetra.p3,
            point,
            f4,
            f2,
            f1,
            old_tetra.f4,
        );
        // self.set_opposing_in_new_tetra(t1, f3, t2, old_tetra.p1);
        todo!();
        // self.set_opposing_in_existing_tetra(old_tetra.f1, t1, point, old_tetra_index);
        // todo!()
        for (tetra, face) in [
            (t1, old_tetra.f1),
            (t2, old_tetra.f2),
            (t3, old_tetra.f3),
            (t4, old_tetra.f4),
        ] {
            self.to_check.push(FlipCheckData {
                tetra,
                face: face.face,
            });
        }
    }

    pub(super) fn flip(&mut self, _check: FlipCheckData) {
        todo!()
    }

    pub fn insert_basic_tetra(&mut self, _tetra: TetraData) {
        todo!()
    }
}
