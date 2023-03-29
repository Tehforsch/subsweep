use std::collections::HashSet;
use std::f64::consts::PI;

use super::constructor::Constructor;
use super::delaunay::dimension::Dimension;
use super::delaunay::FaceIndex;
use super::delaunay::PointIndex;
use super::primitives::polygon3d::Polygon3d;
use super::primitives::Point2d;
use super::primitives::Point3d;
use super::primitives::Vector;
use super::utils::arrange_cyclic_by;
use super::utils::periodic_windows_2;
use super::CellIndex;
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
    fn new(constructor: &Constructor<Self::Dimension>, point: PointIndex) -> Self;
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
    pub center: Point<D>,
    pub is_infinite: bool,
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

impl Cell<TwoD> {
    fn tetras_are_neighbours(c: &Constructor<TwoD>, t1: &TetraIndex, t2: &TetraIndex) -> bool {
        get_common_face(c, t1, t2).is_some()
    }

    pub fn point_windows(&self) -> impl Iterator<Item = (&Point2d, &Point2d)> {
        periodic_windows_2(&self.points)
    }
}

impl Cell<TwoD> {
    fn get_faces(c: &Constructor<TwoD>, p: PointIndex) -> Vec<VoronoiFace<TwoD>> {
        let tetras = &c.point_to_tetras_map[&p];
        arrange_cyclic_by(tetras, |t1, t2| Self::tetras_are_neighbours(c, t1, t2))
            .map(|(t1, t2)| {
                let line = &c.triangulation.faces[get_common_face(c, t1, t2).unwrap()];
                let p2_index = line.other_point(p);
                let vp1 = c.tetra_to_voronoi_point_map[t1];
                let vp2 = c.tetra_to_voronoi_point_map[t2];
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

    fn new(c: &Constructor<TwoD>, p: PointIndex) -> Self {
        let tetras = &c.point_to_tetras_map[&p];
        let points = arrange_cyclic_by(tetras, |t1, t2| Self::tetras_are_neighbours(c, t1, t2))
            .map(|(t1, _)| c.tetra_to_voronoi_point_map[t1])
            .collect();
        Self {
            delaunay_point: p,
            points,
            is_infinite: c.is_infinite_cell(p),
            index: *c.point_to_cell_map.get_by_right(&p).unwrap(),
            faces: Self::get_faces(c, p),
            center: c.triangulation.points[p],
        }
    }
}

fn pyramid_volume(normal: Point3d, p: Point3d, polygon: &Polygon3d) -> Float {
    let base = polygon.area();
    let height = normal.dot(p - polygon.points[0]).abs();
    1.0 / 3.0 * base * height
}

impl Cell<ThreeD> {
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

impl DimensionCell for Cell<ThreeD> {
    type Dimension = ThreeD;

    fn contains(&self, point: Point3d) -> bool {
        self.faces
            .iter()
            .all(|face| face.normal.dot(point - face.data.points[0]) < 0.0)
    }

    fn size(&self) -> Float {
        (3.0 * self.volume() / (4.0 * PI)).cbrt()
    }

    fn volume(&self) -> Float {
        // The volume is the sum over the volumes of the "pyramids" spanned by
        // the generating delaunay point and a face.
        self.faces
            .iter()
            .map(|face| pyramid_volume(face.normal, self.center, &face.data))
            .sum()
    }

    fn new(c: &Constructor<ThreeD>, p: PointIndex) -> Self {
        let points = c.point_to_tetras_map[&p]
            .iter()
            .map(|tetra| c.tetra_to_voronoi_point_map[tetra])
            .collect();
        Self {
            delaunay_point: p,
            points,
            is_infinite: c.is_infinite_cell(p),
            index: *c.point_to_cell_map.get_by_right(&p).unwrap(),
            faces: Self::get_faces(c, p),
            center: c.triangulation.points[p],
        }
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
    let points = arrange_cyclic_by(&tetras_with_both_points, tetras_are_neighbours)
        .map(|(p1, _)| c.tetra_to_voronoi_point_map[p1])
        .collect();
    let normal = get_normal(c, p1, p2);
    let poly = Polygon3d { points };
    VoronoiFace {
        connection: c.get_connection(p2),
        normal,
        area: poly.area(),
        data: poly,
    }
}
