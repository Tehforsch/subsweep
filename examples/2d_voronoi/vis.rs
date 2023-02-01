use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use raxiom::voronoi::Point;

#[derive(Clone, Debug)]
pub struct DrawTriangle {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

impl DrawTriangle {
    pub fn get_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 1.0]; 3]);
        mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));
        mesh
    }
}
