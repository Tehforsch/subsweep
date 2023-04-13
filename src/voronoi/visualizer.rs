use std::fs;
use std::path::PathBuf;

use super::constructor::SearchData;
use super::delaunay::dimension::DDimension;
use super::delaunay::Delaunay;
use super::delaunay::PointKind;
use super::primitives::tetrahedron::TetrahedronData;
use super::primitives::triangle::TriangleData;
use super::primitives::Point2d;
use super::Triangulation;
use crate::dimension::Dimension;
use crate::dimension::TwoD;

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
            fs::write(&f, &contents).unwrap();
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

impl Visualizable for f64 {
    fn get_statements(&self) -> Vec<String> {
        unimplemented!()
    }
}

impl Visualizable for TetrahedronData {
    fn get_statements(&self) -> Vec<String> {
        unimplemented!()
    }
}

impl Visualizable for glam::DVec3 {
    fn get_statements(&self) -> Vec<String> {
        unimplemented!()
    }
}

pub trait Visualizable {
    fn get_statements(&self) -> Vec<String>;
}

impl Visualizable for TriangleData<Point2d> {
    fn get_statements(&self) -> Vec<String> {
        vec![format!(
            "Triangle {} {} {} {} {} {}",
            self.p1.x, self.p1.y, self.p2.x, self.p2.y, self.p3.x, self.p3.y,
        )
        .into()]
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
        for (index, point) in self.points.iter() {
            let color = match self.point_kinds[&index] {
                PointKind::Inner => (1.0, 0.0, 0.0),
                PointKind::Outer => (0.0, 1.0, 0.0),
                PointKind::Halo(_) => (0.0, 0.0, 1.0),
            };
            s.extend(Color { x: *point, color }.get_statements());
        }
        for (_, tetra) in self.tetras.iter() {
            s.extend(self.get_tetra_data(tetra).get_statements());
        }
        s
    }
}

impl Visualizable for Point2d {
    fn get_statements(&self) -> Vec<String> {
        vec![format!("Point {} {}", self.x, self.y).into()]
    }
}

impl Visualizable for SearchData<TwoD> {
    fn get_statements(&self) -> Vec<String> {
        vec![format!("Circle {} {} {}", self.point.x, self.point.y, self.radius).into()]
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
                    "{} {} {} {} {}",
                    statement, self.color.0, self.color.1, self.color.2, 1.0
                )
            })
            .collect()
    }
}

#[macro_export]
macro_rules! vis {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vis = $crate::voronoi::visualizer::Visualizer::default();
            temp_vis.f = Some(std::path::Path::new(&format!("vis/out{}", crate::mpi_log::RANK.load(core::sync::atomic::Ordering::SeqCst))).into());
            $(
                temp_vis.add($x);
            )*
        }
    };
}
