use std::collections::HashMap;

use super::primitives::triangle::TriangleData;
use super::primitives::Point2d;
use super::primitives::Point3d;
use super::DelaunayTriangulation;

pub type Statement = String;
pub type Name = String;

#[derive(Default)]
pub struct Visualizer {
    statement_names: HashMap<Statement, Name>,
    statements: Vec<Statement>,
}

impl Visualizer {
    fn get_new_statement_name(&mut self) -> Name {
        format!("A_{}", self.statement_names.len())
    }

    fn add_statement(&mut self, statement: Statement) -> Name {
        let new_name = self.get_new_statement_name();
        if !self.statement_names.contains_key(&statement) {
            self.statement_names.insert(statement.clone(), new_name);
            self.statements.push(statement.clone());
        }
        self.statement_names[&statement].clone()
    }

    pub fn add(&mut self, p: &impl Visualizable) -> Vec<Name> {
        let res = p
            .get_statements(self)
            .into_iter()
            .map(|statement| self.add_statement(statement))
            .collect();
        res
    }

    fn dump(&self) {
        // The second list is to make sure we iterate in the correct order. Hacky but who cares
        let statements: Vec<_> = self
            .statements
            .iter()
            .map(|statement| format!("\"{} = {}\"", &self.statement_names[statement], &statement))
            .collect();
        println!("Execute({{ {} }})", statements.join(", "));
    }
}

impl Drop for Visualizer {
    fn drop(&mut self) {
        self.dump();
    }
}

pub trait Visualizable {
    fn get_statements(&self, vis: &mut Visualizer) -> Vec<Statement>;
}

impl Visualizable for TriangleData<Point2d> {
    fn get_statements(&self, visualizer: &mut Visualizer) -> Vec<Statement> {
        use super::utils::periodic_windows;
        let points = [self.p1, self.p2, self.p3];
        let point_names: Vec<_> = points
            .into_iter()
            .map(|p| visualizer.add(&p)[0].clone())
            .collect();
        periodic_windows(&point_names)
            .map(|(p1, p2)| format!("Segment({}, {})", p1, p2))
            .collect()
    }
}

#[cfg(feature = "3d")]
impl Visualizable for super::tetra_3d::Tetra3dData {
    fn get_statements(&self, visualizer: &mut Visualizer) -> Vec<Statement> {
        use super::utils::periodic_windows_3;
        let points = [self.p1, self.p2, self.p3, self.p4];
        let point_names: Vec<_> = points
            .into_iter()
            .map(|p| visualizer.add(&p)[0].clone())
            .collect();
        periodic_windows_3(&point_names)
            .map(|(p1, p2, p3)| format!("Polygon({}, {}, {})", p1, p2, p3))
            .collect()
    }
}

impl Visualizable for DelaunayTriangulation {
    fn get_statements(&self, visualizer: &mut Visualizer) -> Vec<String> {
        self.tetras
            .iter()
            .flat_map(|(_, tetra)| self.get_tetra_data(tetra).get_statements(visualizer))
            .collect()
    }
}

impl Visualizable for Point3d {
    fn get_statements(&self, _visualizer: &mut Visualizer) -> Vec<String> {
        vec![format!("({}, {}, {})", self.x, self.y, self.z)]
    }
}

impl Visualizable for Point2d {
    fn get_statements(&self, _visualizer: &mut Visualizer) -> Vec<String> {
        vec![format!("({}, {})", self.x, self.y)]
    }
}

/// Debug print the expression only on MPI rank 0
#[macro_export]
macro_rules! vis {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vis = crate::voronoi::visualizer::Visualizer::default();
            $(
                temp_vis.add($x);
            )*
        }
    };}
