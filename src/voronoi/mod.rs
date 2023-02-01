use generational_arena::Arena;

use self::face::Face;
use self::tetra::{Tetra, TetraData};

mod face;
mod tetra;

#[cfg(feature="2d")]
pub type Point = glam::DVec2;
#[cfg(feature="3d")]
pub type Point = glam::DVec3;

struct TetraIndex(pub usize);
struct FaceIndex(pub usize);
struct PointIndex(pub usize);

type TetraList = Arena<Tetra>;
type FaceList = Arena<Face>;
type PointList = Arena<Point>;

pub struct DelaunayTriangulation {
    pub tetras: TetraList,
    faces: FaceList,
    pub points: PointList,
}

impl DelaunayTriangulation {
    pub fn all_encompassing(points: &[Point]) -> DelaunayTriangulation {
        let initial_tetra_data = get_all_encompassing_tetra(points);
        let mut points = PointList::new();
        let p1 = points.insert(initial_tetra_data.p1);
        let p2 = points.insert(initial_tetra_data.p2);
        let p3 = points.insert(initial_tetra_data.p3);
        #[cfg(not(feature = "2d"))]
        let p4 = points.insert(initial_tetra_data.p4);
        let mut tetras = TetraList::new();
        tetras.insert(Tetra {
            p1,
            p2,
            p3,
            #[cfg(not(feature = "2d"))]
            p4
        });
        DelaunayTriangulation {
            tetras,
            faces: FaceList::default(),
            points,
        }
    }

    pub fn construct(points: &[Point]) -> Self {
        let mut constructor = DelaunayTriangulation::all_encompassing(points);
        for p in points {
            constructor.insert(*p);
        }
        constructor
    }

    pub fn insert(&mut self, point: Point) {

    }
}

#[cfg(feature = "2d")]
fn get_all_encompassing_tetra(points: &[Point]) -> TetraData {
    let (min, max) = get_min_and_max(points).unwrap();
    // A small overshooting factor for numerical safety
    let alpha = 0.01;
    let p1 = min - (max - min) * alpha;
    let p2 = Point::new(min.x, max.y + (max.y - min.y) * (1.0 + alpha));
    let p3 = Point::new(max.x + (max.x - min.x) * (1.0 + alpha), min.y);
    TetraData { p1, p2, p3 }

}

fn get_min_and_max(points: &[Point]) -> Option<(Point, Point)> {
    let mut min = None;
    let mut max = None;
    let update_min = |min: &mut Option<Point>, pos: Point| {
        if let Some(ref mut min) = min {
            *min = min.min(pos);
        } else {
            *min = Some(pos);
        }
    };
    let update_max = |max: &mut Option<Point>, pos: Point| {
        if let Some(ref mut max) = max {
            *max = max.max(pos);
        } else {
            *max = Some(pos);
        }
    };
    for p in points {
        update_min(&mut min, *p);
        update_max(&mut max, *p);
    }
    Some((min?, max?))
}
