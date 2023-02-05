use generational_arena::Arena;
use generational_arena::Index;

use self::face::Face;
use self::face::OtherTetraInfo;
use self::tetra::Tetra;
use self::tetra::TetraData;

mod face;
mod tetra;

#[cfg(feature = "2d")]
pub type Point = glam::DVec2;
#[cfg(feature = "3d")]
pub type Point = glam::DVec3;

type TetraList = Arena<Tetra>;
type FaceList = Arena<Face>;
type PointList = Arena<Point>;

pub struct DelaunayTriangulation {
    pub tetras: TetraList,
    pub faces: FaceList,
    pub points: PointList,
    pub to_check: Vec<Index>,
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
        let mut faces = FaceList::new();
        let f3 = faces.insert(Face {
            p1: p1,
            p2: p2,
            opposing: None,
        });
        let f1 = faces.insert(Face {
            p1: p2,
            p2: p3,
            opposing: None,
        });
        let f2 = faces.insert(Face {
            p1: p3,
            p2: p1,
            opposing: None,
        });
        let mut tetras = TetraList::new();
        tetras.insert(Tetra {
            p1,
            p2,
            p3,
            f1,
            f2,
            f3,
            #[cfg(not(feature = "2d"))]
            p4,
        });
        DelaunayTriangulation {
            tetras,
            faces: FaceList::default(),
            to_check: vec![],
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

    fn get_tetra_data(&self, tetra: &Tetra) -> TetraData {
        TetraData {
            p1: self.points[tetra.p1],
            p2: self.points[tetra.p2],
            p3: self.points[tetra.p3],
        }
    }

    pub fn find_containing_tetra(&self, point: Point) -> Index {
        self.tetras
            .iter()
            .find(|(_, tetra)| {
                let tetra_data = self.get_tetra_data(tetra);
                tetra_data.contains(point)
            })
            .map(|(index, _)| index)
            .expect("No tetra containing the point {point:?} found")
    }

    fn insert(&mut self, point: Point) {
        let t = self.find_containing_tetra(point);
        let new_point_index = self.points.insert(point);
        self.split(t, new_point_index);
        while let Some(check) = self.to_check.pop() {
            self.check_empty_circumcircle(check);
        }
    }

    fn set_opposing_tetras(&mut self, tetra: Index, tetra_a: Index, tetra_b: Index, point: Index) {
        let tetra = &self.tetras[tetra];
        self.faces[tetra.f1].opposing = Some(OtherTetraInfo {
            tetra: tetra_a,
            point,
        });
        self.faces[tetra.f2].opposing = Some(OtherTetraInfo {
            tetra: tetra_b,
            point,
        });
    }

    fn make_tetra(&mut self, p: Index, p_a: Index, p_b: Index, old_face: Index) -> Index {
        // Leave f1.opposing and f2.opposing uninitialized for now, since we do not know the index
        // before we have inserted the other two tetras
        let f1 = self.faces.insert(Face {
            p1: p,
            p2: p_a,
            opposing: None,
        });
        let f2 = self.faces.insert(Face {
            p1: p,
            p2: p_b,
            opposing: None,
        });
        self.tetras.insert(Tetra {
            p1: p_a,
            p2: p_b,
            p3: p,
            f1,
            f2,
            f3: old_face,
        })
    }

    fn split(&mut self, tetra: Index, point: Index) {
        let old_tetra = self.tetras.remove(tetra).unwrap();
        let t1 = self.make_tetra(point, old_tetra.p2, old_tetra.p3, old_tetra.f1);
        let t2 = self.make_tetra(point, old_tetra.p3, old_tetra.p1, old_tetra.f2);
        let t3 = self.make_tetra(point, old_tetra.p1, old_tetra.p2, old_tetra.f3);
        self.set_opposing_tetras(t1, t3, t2, old_tetra.p1);
        self.set_opposing_tetras(t2, t1, t3, old_tetra.p2);
        self.set_opposing_tetras(t3, t2, t1, old_tetra.p3);
        for t in [t1, t2, t3] {
            self.to_check.push(t);
        }
    }

    fn check_empty_circumcircle(&mut self, to_check: Index) {
        todo!()
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
