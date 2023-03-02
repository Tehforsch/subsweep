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
        todo!()
    }

    pub(super) fn split(&mut self, old_tetra_index: TetraIndex, point: PointIndex) {
        todo!()
    }

    pub(super) fn flip(&mut self, check: FlipCheckData) {
        todo!()
    }

    #[cfg(feature = "3d")]
    pub fn insert_basic_tetra(&mut self, _tetra: TetraData) {
        todo!()
    }
}
