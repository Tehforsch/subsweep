use super::DelaunayTriangulation;
use super::FlipCheckData;
use crate::voronoi::tetra::Tetra;
use crate::voronoi::tetra::TetraData;
use crate::voronoi::tetra::TetraFace;
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

    fn _insert_positively_oriented_tetra(
        &mut self,
        _p1: PointIndex,
        _p2: PointIndex,
        _p3: PointIndex,
        _p4: PointIndex,
        _f1: TetraFace,
        _f2: TetraFace,
        _f3: TetraFace,
        _f4: TetraFace,
    ) -> TetraIndex {
        todo!()
    }

    pub(super) fn split(&mut self, _old_tetra_index: TetraIndex, _point: PointIndex) {
        todo!()
    }

    pub(super) fn flip(&mut self, _check: FlipCheckData) {
        todo!()
    }

    pub fn insert_basic_tetra(&mut self, _tetra: TetraData) {
        todo!()
    }
}
