use std::f64::consts::PI;

use super::delaunay::dimension::Dimension;
use super::delaunay::Delaunay;
use super::delaunay::PointIndex;
use super::primitives::Point2d;
use super::primitives::Point3d;
use super::utils::arrange_cyclic_by;
use super::utils::periodic_windows_2;
use super::CellIndex;
use super::Constructor;
use super::DelaunayTriangulation;
use super::Point;
use super::ThreeD;
use super::TwoD;
use crate::prelude::Float;
use crate::voronoi::delaunay::TetraIndex;
use crate::voronoi::DimensionTetra;

pub trait DimensionCell: Sized {
    type Dimension: Dimension;
    fn size(&self) -> Float;
    fn volume(&self) -> Float;
    fn contains(&self, point: Point<Self::Dimension>) -> bool;
    fn get_faces(
        c: &Constructor<Self::Dimension>,
        p: PointIndex,
    ) -> Vec<VoronoiFace<Self::Dimension>>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CellConnection {
    ToInner(CellIndex),
    ToOuter,
}

pub struct Cell<D: Dimension> {
    pub delaunay_point: PointIndex,
    pub index: CellIndex,
    pub points: Vec<Point<D>>,
    pub faces: Vec<VoronoiFace<D>>,
}

pub struct VoronoiFace<D: Dimension> {
    pub connection: CellConnection,
    pub normal: Point<D>,
    pub area: Float,
}

fn sign(p1: Point2d, p2: Point2d, p3: Point2d) -> Float {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

impl Cell<TwoD> {
    pub fn point_windows(&self) -> impl Iterator<Item = (&Point2d, &Point2d)> {
        periodic_windows_2(&self.points)
    }
}

impl<D: Dimension> Cell<D>
where
    DelaunayTriangulation<D>: Delaunay<D>,
    Cell<D>: DimensionCell<Dimension = D>,
{
    pub fn new(c: &Constructor<'_, D>, p: PointIndex) -> Self {
        let points = c.point_to_tetras_map[&p]
            .iter()
            .map(|tetra| c.tetra_to_voronoi_point_map[tetra])
            .collect();
        Self {
            delaunay_point: p,
            points,
            index: c.point_to_cell_map[&p],
            faces: Self::get_faces(c, p),
        }
    }
}

impl DimensionCell for Cell<TwoD> {
    type Dimension = TwoD;

    fn contains(&self, point: Point2d) -> bool {
        let has_negative = self
            .point_windows()
            .map(|(p1, p2)| sign(point, *p1, *p2))
            .any(|s| s < 0.0);
        let has_positive = self
            .point_windows()
            .map(|(p1, p2)| sign(point, *p1, *p2))
            .any(|s| s > 0.0);

        !(has_negative && has_positive)
    }

    fn size(&self) -> Float {
        (self.volume() / PI).sqrt()
    }

    fn volume(&self) -> Float {
        0.5 * self
            .point_windows()
            .map(|(p1, p2)| p1.x * p2.y - p2.x * p1.y)
            .sum::<Float>()
            .abs()
    }

    fn get_faces(c: &Constructor<TwoD>, p: PointIndex) -> Vec<VoronoiFace<TwoD>> {
        let tetras = &c.point_to_tetras_map[&p];
        let mut faces = vec![];
        let get_common_face = |t1: &TetraIndex, t2: &TetraIndex| {
            let t1_data = &c.triangulation.tetras[*t1];
            let t2_data = &c.triangulation.tetras[*t2];
            t1_data.get_common_face_with(t2_data)
        };
        let tetras_are_neighbours =
            |t1: &TetraIndex, t2: &TetraIndex| get_common_face(t1, t2).is_some();
        for (t1, t2) in arrange_cyclic_by(tetras, tetras_are_neighbours) {
            let line = &c.triangulation.faces[get_common_face(t1, t2).unwrap()];
            let p2_index = line.other_point(p);
            let connection = c
                .point_to_cell_map
                .get(&p2_index)
                .map(|i| CellConnection::ToInner(*i))
                .unwrap_or(CellConnection::ToOuter);
            let p1 = c.triangulation.points[p];
            let p2 = c.triangulation.points[p2_index];
            let mut normal = (p1 - p2).normalize();
            if (p2 - p1).dot(normal) < 0.0 {
                normal = -normal;
            }
            let vp1 = c.tetra_to_voronoi_point_map[&t1];
            let vp2 = c.tetra_to_voronoi_point_map[&t2];
            let area = vp1.distance(vp2);
            faces.push(VoronoiFace {
                connection,
                normal,
                area,
            });
        }
        faces
    }
}

impl DimensionCell for Cell<ThreeD> {
    type Dimension = ThreeD;

    fn contains(&self, _point: Point3d) -> bool {
        todo!()
    }

    fn size(&self) -> Float {
        (3.0 * self.volume() / (4.0 * PI)).cbrt()
    }

    fn volume(&self) -> Float {
        todo!()
    }

    fn get_faces(c: &Constructor<ThreeD>, p: PointIndex) -> Vec<VoronoiFace<ThreeD>> {
        todo!()
        //             let mut connected_cells = HashSet::new();
        //             let tetras = &point_to_tetra_map[&point_index];
        //             for tetra in tetras.iter() {
        //                 let tetra_data = &t.tetras[*tetra];
        //                 points.push(t.get_tetra_data(&tetra_data).get_center_of_circumcircle());
        //                 let face = tetra_data.find_face_opposite(point_index);
        //                 for other_point in t.faces[face.face].points() {
        //                     connected_cells.insert(
        //                         map.get(&other_point)
        //                             .map(|i| CellConnection::ToInner(*i))
        //                             .unwrap_or(CellConnection::ToOuter),
        //                     );
        //                 }
        //             }
    }
}
