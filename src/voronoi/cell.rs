use std::f64::consts::PI;

use super::delaunay::dimension::Dimension;
use super::delaunay::dimension::DimensionTetraData;
use super::delaunay::Delaunay;
use super::delaunay::PointIndex;
use super::primitives::Point2d;
use super::primitives::Point3d;
use super::utils::periodic_windows_2;
use super::CellIndex;
use super::Constructor;
use super::DelaunayTriangulation;
use super::Point;
use super::ThreeD;
use super::TwoD;
use crate::prelude::Float;

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
            .map(|tetra| {
                let tetra_data = &c.triangulation.tetras[*tetra];
                c.triangulation
                    .get_tetra_data(&tetra_data)
                    .get_center_of_circumcircle()
            })
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
        todo!()
        // let tetras = &c.point_to_tetras_map[&p];
        // let mut prev_tetra: Option<TetraIndex> = None;
        // let mut tetra = tetras[0];
        // loop {
        //     let tetra_data = &c.triangulation.tetras[tetra];
        //     let face = tetra_data
        //         .faces()
        //         .find(|face| {
        //             if let Some(opp) = face.opposing {
        //                 let other_tetra_is_incident_with_cell = tetras.contains(&opp.tetra);
        //                 let other_tetra_is_prev_tetra = prev_tetra
        //                     .map(|prev_tetra| prev_tetra == opp.tetra)
        //                     .unwrap_or(false);
        //                 other_tetra_is_incident_with_cell && !other_tetra_is_prev_tetra
        //             } else {
        //                 false
        //             }
        //         })
        //         .unwrap();
        //     for other_point in t.faces[face.face].other_points(p) {
        //         connected_cells.push(
        //             map.get(&other_point)
        //                 .map(|i| CellConnection::ToInner(*i))
        //                 .unwrap_or(CellConnection::ToOuter),
        //         );
        //     }
        //     prev_tetra = Some(tetra);
        //     tetra = face.opposing.unwrap().tetra;
        //     if tetra == tetras[0] {
        //         break;
        //     }
        // }
        // iter_faces_two_d(t.points[point_index], &points, &connected_cells).collect()
        // fn iter_faces_two_d<'a>(
        //     center: Point2d,
        //     points: &'a [Point2d],
        //     connected_cells: &'a [CellConnection],
        // ) -> impl Iterator<Item = VoronoiFace<TwoD>> + 'a {
        //     connected_cells
        //         .iter()
        //         .zip(periodic_windows_2(points))
        //         .map(move |(c, (p1, p2))| {
        //             let area = p1.distance(*p2);
        //             let dir = *p1 - *p2;
        //             let mut normal = Point2d::new(dir.y, -dir.x).normalize();
        //             if (*p1 - center).dot(normal) < 0.0 {
        //                 normal = -normal;
        //             }
        //             VoronoiFace {
        //                 area,
        //                 normal,
        //                 connection: *c,
        //             }
        //         })
        // }
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
