use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use mpi::traits::Equivalence;

use super::super::camera_transform::CameraTransform;
use super::super::color::RColor;
use super::DrawItem;
use crate::units::VecLength;

#[derive(Equivalence, Component, Clone, Debug)]
pub struct DrawRect {
    pub lower_left: VecLength,
    pub upper_right: VecLength,
    pub color: RColor,
}

impl DrawItem for DrawRect {
    type Output = ShapeBundle;
    fn get_bundle(&self, camera_transform: &CameraTransform) -> Self::Output {
        let shape = shapes::Rectangle {
            extents: camera_transform.position_to_pixels(self.upper_right - self.lower_left),
            origin: RectangleOrigin::BottomLeft,
        };

        GeometryBuilder::build_as(
            &shape,
            DrawMode::Stroke(StrokeMode::new(self.color.into(), 2.0)),
            Transform::default(),
        )
    }

    fn translation(&self) -> &VecLength {
        &self.lower_left
    }

    fn set_translation(&mut self, pos: &VecLength) {
        self.lower_left = *pos;
    }

    fn get_color(&self) -> RColor {
        self.color
    }
}
