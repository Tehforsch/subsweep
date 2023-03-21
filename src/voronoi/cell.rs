use std::collections::HashSet;
use std::f64::consts::PI;

use super::delaunay::dimension::Dimension;
use super::delaunay::Delaunay;
use super::delaunay::FaceIndex;
use super::delaunay::PointIndex;
use super::primitives::Point2d;
use super::primitives::Point3d;
use super::primitives::Vector;
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
    pub data: D::VoronoiFaceData,
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

fn get_common_face<D: Dimension>(
    c: &Constructor<D>,
    t1: &TetraIndex,
    t2: &TetraIndex,
) -> Option<FaceIndex> {
    let t1_data = &c.triangulation.tetras[*t1];
    let t2_data = &c.triangulation.tetras[*t2];
    t1_data.get_common_face_with(t2_data)
}

fn get_normal<D: Dimension>(c: &Constructor<D>, p1: PointIndex, p2: PointIndex) -> Point<D> {
    let p1 = c.triangulation.points[p1];
    let p2 = c.triangulation.points[p2];
    (p2 - p1).normalize()
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
        let tetras_are_neighbours =
            |t1: &TetraIndex, t2: &TetraIndex| get_common_face(c, t1, t2).is_some();
        arrange_cyclic_by(tetras, tetras_are_neighbours)
            .map(|(t1, t2)| {
                let line = &c.triangulation.faces[get_common_face(c, t1, t2).unwrap()];
                let p2_index = line.other_point(p);
                let vp1 = c.tetra_to_voronoi_point_map[&t1];
                let vp2 = c.tetra_to_voronoi_point_map[&t2];
                let area = vp1.distance(vp2);
                let normal = get_normal(c, p, p2_index);
                VoronoiFace {
                    connection: c.get_connection(p2_index),
                    normal,
                    area,
                    data: (),
                }
            })
            .collect()
    }
}

impl DimensionCell for Cell<ThreeD> {
    type Dimension = ThreeD;

    fn contains(&self, point: Point3d) -> bool {
        self.faces
            .iter()
            .all(|face| face.normal.dot(point - face.data[0]) < 0.0)
    }

    fn size(&self) -> Float {
        (3.0 * self.volume() / (4.0 * PI)).cbrt()
    }

    fn volume(&self) -> Float {
        todo!()
    }

    fn get_faces(c: &Constructor<ThreeD>, p1: PointIndex) -> Vec<VoronoiFace<ThreeD>> {
        let tetras = &c.point_to_tetras_map[&p1];
        let connected_points: HashSet<PointIndex> = tetras
            .iter()
            .flat_map(|tetra| {
                c.triangulation.tetras[*tetra]
                    .points()
                    .filter(|p2| *p2 != p1)
            })
            .collect();
        connected_points
            .iter()
            .map(|p2| get_face_polygon_perpendicular_to_line(c, p1, *p2))
            .collect()
    }
}

fn get_face_polygon_perpendicular_to_line(
    c: &Constructor<ThreeD>,
    p1: PointIndex,
    p2: PointIndex,
) -> VoronoiFace<ThreeD> {
    let tetras_are_neighbours =
        |t1: &TetraIndex, t2: &TetraIndex| get_common_face(c, t1, t2).is_some();
    let tetras_with_p1 = &c.point_to_tetras_map[&p1];
    let tetras_with_both_points: Vec<TetraIndex> = tetras_with_p1
        .iter()
        .filter(|tetra| {
            let tetra = &c.triangulation.tetras[**tetra];
            debug_assert!(tetra.contains_point(p1));
            tetra.contains_point(p2)
        })
        .cloned()
        .collect();
    let mut area = 0.0;
    // Take any point out of the polygon for reference
    let r: Point3d = c.tetra_to_voronoi_point_map[&tetras_with_both_points[0]];
    let mut points = vec![];
    for (t1, t2) in arrange_cyclic_by(&tetras_with_both_points, tetras_are_neighbours) {
        let vp1 = c.tetra_to_voronoi_point_map[&t1];
        points.push(vp1);
        let vp2 = c.tetra_to_voronoi_point_map[&t2];
        area += 0.5 * (r - vp1).cross(r - vp2).length();
    }
    let normal = get_normal(c, p1, p2);
    VoronoiFace {
        connection: c.get_connection(p2),
        normal,
        area,
        data: points,
    }
}
