use std::collections::HashMap;

use super::delaunay::dimension::Dimension;
use super::delaunay::Delaunay;
use super::delaunay::TetraIndex;
use super::primitives::triangle::TriangleData;
use super::primitives::Point2d;
use super::primitives::Point3d;
use super::Cell;
use super::DelaunayTriangulation;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Statement {
    statement: String,
    is_new_item: bool,
}

impl From<String> for Statement {
    fn from(statement: String) -> Self {
        Self {
            statement,
            is_new_item: true,
        }
    }
}

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
            .map(|statement| {
                if statement.is_new_item {
                    format!(
                        "\"{} = {}\"",
                        &self.statement_names[statement], &statement.statement
                    )
                } else {
                    format!("\"{}\"", &statement.statement)
                }
            })
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
        use super::utils::periodic_windows_2;
        let points = [self.p1, self.p2, self.p3];
        let point_names: Vec<_> = points
            .into_iter()
            .map(|p| visualizer.add(&p)[0].clone())
            .collect();
        periodic_windows_2(&point_names)
            .map(|(p1, p2)| format!("Segment({}, {})", p1, p2).into())
            .collect()
    }
}

impl Visualizable for super::primitives::tetrahedron::TetrahedronData {
    fn get_statements(&self, visualizer: &mut Visualizer) -> Vec<Statement> {
        use super::utils::periodic_windows_3;
        let points = [self.p1, self.p2, self.p3, self.p4];
        let point_names: Vec<_> = points
            .into_iter()
            .map(|p| visualizer.add(&p)[0].clone())
            .collect();
        periodic_windows_3(&point_names)
            .map(|(p1, p2, p3)| format!("Polygon({}, {}, {})", p1, p2, p3).into())
            .collect()
    }
}

impl<D> Visualizable for DelaunayTriangulation<D>
where
    D: Dimension,
    DelaunayTriangulation<D>: Delaunay<D>,
    <D as Dimension>::TetraData: Visualizable,
{
    fn get_statements(&self, visualizer: &mut Visualizer) -> Vec<Statement> {
        self.tetras
            .iter()
            .flat_map(|(_, tetra)| self.get_tetra_data(tetra).get_statements(visualizer))
            .collect()
    }
}

impl<D> Visualizable for (&DelaunayTriangulation<D>, TetraIndex)
where
    D: Dimension,
    DelaunayTriangulation<D>: Delaunay<D>,
    <D as Dimension>::TetraData: Visualizable,
{
    fn get_statements(&self, visualizer: &mut Visualizer) -> Vec<Statement> {
        self.0
            .get_tetra_data(&self.0.tetras[self.1])
            .get_statements(visualizer)
    }
}

impl<D> Visualizable for Cell<D>
where
    D: Dimension,
    <D as Dimension>::Point: Visualizable,
{
    fn get_statements(&self, visualizer: &mut Visualizer) -> Vec<Statement> {
        let points: Vec<_> = self
            .points
            .iter()
            .map(|p| p.get_statements(visualizer)[0].statement.clone())
            .collect();
        vec![format!("Polygon({})", points.join(",")).into()]
    }
}

impl Visualizable for Point3d {
    fn get_statements(&self, _visualizer: &mut Visualizer) -> Vec<Statement> {
        vec![format!("({}, {}, {})", self.x, self.y, self.z).into()]
    }
}

impl Visualizable for Point2d {
    fn get_statements(&self, _visualizer: &mut Visualizer) -> Vec<Statement> {
        vec![format!("({}, {})", self.x, self.y).into()]
    }
}

pub struct Color {
    pub x: Box<dyn Visualizable>,
    pub color: (f64, f64, f64),
}

impl Visualizable for Color {
    fn get_statements(&self, visualizer: &mut Visualizer) -> Vec<Statement> {
        let statements = self.x.get_statements(visualizer);
        statements
            .into_iter()
            .map(|statement| {
                let name = visualizer.add_statement(statement);
                Statement {
                    statement: format!(
                        "SetColor({}, {}, {}, {})",
                        name, self.color.0, self.color.1, self.color.2
                    ),
                    is_new_item: false,
                }
            })
            .collect()
    }
}

#[macro_export]
macro_rules! vis {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vis = crate::voronoi::visualizer::Visualizer::default();
            $(
                temp_vis.add($x);
            )*
        }
    };
}

#[macro_export]
macro_rules! highlight_red {
    ( $x:expr) => {{
        &crate::voronoi::visualizer::Color {
            x: Box::new($x.clone()),
            color: (1.0, 0.0, 0.0),
        }
    }};
}

#[macro_export]
macro_rules! highlight_blue {
    ( $x:expr) => {{
        &crate::voronoi::visualizer::Color {
            x: Box::new($x.clone()),
            color: (0.0, 0.0, 1.0),
        }
    }};
}

#[macro_export]
macro_rules! highlight_green {
    ( $x:expr) => {{
        &crate::voronoi::visualizer::Color {
            x: Box::new($x.clone()),
            color: (0.0, 1.0, 0.0),
        }
    }};
}
