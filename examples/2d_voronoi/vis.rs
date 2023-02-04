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
        let vertices = [
            ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
            ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
            ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
        ];

        let indices = Indices::U32(vec![0, 1, 2, 0]);

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
