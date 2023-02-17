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
            (
                [self.p1.x as f32, self.p1.y as f32, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0],
            ),
            (
                [self.p2.x as f32, self.p2.y as f32, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0],
            ),
            (
                [self.p3.x as f32, self.p3.y as f32, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0],
            ),
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

#[derive(Clone, Debug)]
pub struct DrawPolygon {
    pub points: Vec<Point>,
}

impl DrawPolygon {
    pub fn get_mesh(&self) -> Mesh {
        let vertices: Vec<_> = self
            .points
            .iter()
            .map(|p| ([p.x as f32, p.y as f32, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]))
            .collect();

        let mut indices: Vec<u32> = (0..self.points.len()).map(|i| i as u32).collect();
        indices.push(0);
        let indices = Indices::U32(indices);

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
