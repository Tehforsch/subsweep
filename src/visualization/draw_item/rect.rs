use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use mpi::traits::Equivalence;

use super::super::color::RColor;
use super::DrawItem;
use crate::units::VecLength;

#[derive(Equivalence, Component, Clone, Debug)]
pub struct DrawRect {
    center: VecLength,
    size: VecLength,
    color: RColor,
}

impl DrawRect {
    pub fn from_min_max(min: VecLength, max: VecLength, color: RColor) -> Self {
        Self {
            center: (min + max) / 2.0,
            size: (max - min),
            color,
        }
    }
}

impl DrawItem for DrawRect {
    fn translation(&self) -> &VecLength {
        &self.center
    }

    fn set_translation(&mut self, pos: &VecLength) {
        self.center = *pos;
    }

    fn get_color(&self) -> RColor {
        self.color
    }

    fn get_scale(&self, camera_transform: &super::CameraTransform) -> Vec2 {
        Vec2::new(
            camera_transform.length_to_pixels(self.size.x()),
            camera_transform.length_to_pixels(self.size.y()),
        )
    }

    fn get_mesh() -> Mesh {
        // Adapted from bevys Quad but with LineStrip instead of TriangleList
        let size = Vec2::new(1.0, 1.0);
        let extent_x = size.x / 2.0;
        let extent_y = size.y / 2.0;

        let (u_left, u_right) = (0.0, 1.0);
        let vertices = [
            ([-extent_x, -extent_y, 0.0], [0.0, 0.0, 1.0], [u_left, 1.0]),
            ([-extent_x, extent_y, 0.0], [0.0, 0.0, 1.0], [u_left, 0.0]),
            ([extent_x, extent_y, 0.0], [0.0, 0.0, 1.0], [u_right, 0.0]),
            ([extent_x, -extent_y, 0.0], [0.0, 0.0, 1.0], [u_right, 1.0]),
        ];

        let indices = Indices::U32(vec![0, 1, 2, 3, 0]);

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
