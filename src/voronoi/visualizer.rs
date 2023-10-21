use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;

use super::constructor::SearchData;
use super::delaunay::dimension::DDimension;
use super::delaunay::Delaunay;
use super::delaunay::PointKind;
use super::primitives::tetrahedron::TetrahedronData;
use super::primitives::triangle::TriangleData;
use super::primitives::Point2d;
use super::Point3d;
use super::Triangulation;
use crate::dimension::Dimension;
use crate::dimension::ThreeD;
use crate::dimension::TwoD;
use crate::extent::Extent;

pub static NUM_VIS: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
pub struct Visualizer {
    statements: Vec<String>,
    pub f: Option<PathBuf>,
}

impl Visualizer {
    pub fn add(&mut self, p: &impl Visualizable) {
        self.statements.extend(p.get_statements())
    }

    fn dump(&self) {
        let contents = self.statements.join("\n");
        if let Some(f) = &self.f {
            fs::write(f, &contents).unwrap();
        } else {
            println!("{}", &contents);
        }
    }
}

impl Drop for Visualizer {
    fn drop(&mut self) {
        self.dump();
    }
}

pub trait Visualizable {
    fn get_statements(&self) -> Vec<String>;
}

impl Visualizable for f64 {
    fn get_statements(&self) -> Vec<String> {
        unimplemented!()
    }
}

fn project_2d(p: Point3d) -> Point2d {
    Point2d::new(p.y, p.z)
}

impl Visualizable for TetrahedronData {
    fn get_statements(&self) -> Vec<String> {
        vec![
            format!(
                "Polygon {} {} {} {} {} {}",
                project_2d(self.p1).x,
                project_2d(self.p1).y,
                project_2d(self.p2).x,
                project_2d(self.p2).y,
                project_2d(self.p3).x,
                project_2d(self.p3).y,
            ),
            format!(
                "Polygon {} {} {} {} {} {}",
                project_2d(self.p1).x,
                project_2d(self.p1).y,
                project_2d(self.p2).x,
                project_2d(self.p2).y,
                project_2d(self.p4).x,
                project_2d(self.p4).y,
            ),
            format!(
                "Polygon {} {} {} {} {} {}",
                project_2d(self.p1).x,
                project_2d(self.p1).y,
                project_2d(self.p3).x,
                project_2d(self.p3).y,
                project_2d(self.p4).x,
                project_2d(self.p4).y,
            ),
            format!(
                "Polygon {} {} {} {} {} {}",
                project_2d(self.p2).x,
                project_2d(self.p2).y,
                project_2d(self.p3).x,
                project_2d(self.p3).y,
                project_2d(self.p4).x,
                project_2d(self.p4).y,
            ),
        ]
    }
}

impl Visualizable for TriangleData<Point2d> {
    fn get_statements(&self) -> Vec<String> {
        vec![format!(
            "Polygon {} {} {} {} {} {}",
            self.p1.x, self.p1.y, self.p2.x, self.p2.y, self.p3.x, self.p3.y,
        )]
    }
}

impl Visualizable for Extent<Point2d> {
    fn get_statements(&self) -> Vec<String> {
        let p1 = Point2d::new(self.min.x, self.min.y);
        let p2 = Point2d::new(self.max.x, self.min.y);
        let p3 = Point2d::new(self.max.x, self.max.y);
        let p4 = Point2d::new(self.min.x, self.max.y);
        vec![format!(
            "Polygon {} {} {} {} {} {} {} {} color 0.0 1.0 0.0 1.0",
            p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, p4.x, p4.y,
        )]
    }
}

impl Visualizable for Extent<Point3d> {
    fn get_statements(&self) -> Vec<String> {
        let p1 = Point2d::new(self.min.x, self.min.y);
        let p2 = Point2d::new(self.max.x, self.min.y);
        let p3 = Point2d::new(self.max.x, self.max.y);
        let p4 = Point2d::new(self.min.x, self.max.y);
        vec![format!(
            "Polygon {} {} {} {} {} {} {} {} color 0.0 1.0 0.0 1.0",
            p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, p4.x, p4.y,
        )]
    }
}

impl<D> Visualizable for Triangulation<D>
where
    D: DDimension,
    Triangulation<D>: Delaunay<D>,
    <D as DDimension>::TetraData: Visualizable,
    <D as Dimension>::Point: Visualizable,
{
    fn get_statements(&self) -> Vec<String> {
        let mut s = vec![];
        for (index, point) in self.iter_original_points() {
            let color = match self.point_kinds[&index] {
                PointKind::Inner => (1.0, 0.0, 0.0),
                PointKind::Outer => (0.0, 1.0, 0.0),
                PointKind::Halo(_) => (0.0, 0.0, 1.0),
            };
            s.extend(Color { x: point, color }.get_statements());
        }
        for (_, tetra) in self.tetras.iter() {
            s.extend(self.get_tetra_data(tetra).get_statements());
        }
        s
    }
}

impl Visualizable for Point2d {
    fn get_statements(&self) -> Vec<String> {
        vec![format!("Point {} {}", self.x, self.y)]
    }
}

impl Visualizable for Point3d {
    fn get_statements(&self) -> Vec<String> {
        vec![format!(
            "Point {} {}",
            project_2d(*self).x,
            project_2d(*self).y
        )]
    }
}

impl Visualizable for SearchData<TwoD> {
    fn get_statements(&self) -> Vec<String> {
        vec![format!(
            "Circle {} {} {}",
            self.point.x, self.point.y, self.radius
        )]
    }
}

impl Visualizable for SearchData<ThreeD> {
    fn get_statements(&self) -> Vec<String> {
        vec![format!(
            "Circle {} {} {}",
            self.point.x, self.point.y, self.radius
        )]
    }
}

pub struct Color<T> {
    pub x: T,
    pub color: (f64, f64, f64),
}

impl<T: Visualizable> Visualizable for Color<T> {
    fn get_statements(&self) -> Vec<String> {
        self.x
            .get_statements()
            .into_iter()
            .map(|statement| {
                format!(
                    "{} color {} {} {} {}",
                    statement, self.color.0, self.color.1, self.color.2, 1.0
                )
            })
            .collect()
    }
}

impl<T: Visualizable> Visualizable for Vec<T> {
    fn get_statements(&self) -> Vec<String> {
        self.iter()
            .flat_map(|x| x.get_statements().into_iter())
            .collect()
    }
}

#[macro_export]
macro_rules! vis {
    ( $( $x:expr ),* ) => {
        {
            let rank = $crate::mpi_log::RANK.load(core::sync::atomic::Ordering::SeqCst);
            let num_vis = $crate::voronoi::visualizer::NUM_VIS.load(core::sync::atomic::Ordering::SeqCst);
            let folder = format!("vis/{:01}", rank);
            let folder = std::path::Path::new(&folder);
            if num_vis == 0 {
                std::fs::remove_dir_all(folder).ok();
            }
            std::fs::create_dir_all(&folder).unwrap();
            let mut temp_vis = $crate::voronoi::visualizer::Visualizer::default();
            $crate::voronoi::visualizer::NUM_VIS.swap(num_vis+1, core::sync::atomic::Ordering::SeqCst);
            temp_vis.f = Some(std::path::Path::new(&folder.join(&format!("{:03}", num_vis))).into());
            $(
                temp_vis.add($x);
            )*
        }
    };
}
