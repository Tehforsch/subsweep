use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use mpi::traits::Equivalence;

use super::super::camera_transform::CameraTransform;
use super::DrawItem;
use super::Pixels;
use crate::units::VecLength;
use crate::visualization::color::RColor;

pub static CIRCLE_RADIUS: f64 = 3.0;

#[derive(Equivalence, Component, Clone, Debug)]
pub struct DrawCircle {
    pub position: VecLength,
    pub radius: Pixels,
    pub color: RColor,
}

impl DrawCircle {
    pub fn from_position_and_color(position: VecLength, color: RColor) -> Self {
        Self {
            position,
            color,
            radius: Pixels(CIRCLE_RADIUS),
        }
    }
}

impl DrawItem for DrawCircle {
    type Output = ShapeBundle;
    fn get_bundle(&self, _: &CameraTransform) -> Self::Output {
        let shape = shapes::Circle {
            radius: self.radius.0 as f32,
            center: Vec2::new(0.0, 0.0),
        };

        GeometryBuilder::build_as(
            &shape,
            DrawMode::Fill(FillMode::color(self.color.into())),
            Transform::default(),
        )
    }

    fn translation(&self) -> &VecLength {
        &self.position
    }

    fn set_translation(&mut self, pos: &VecLength) {
        self.position = *pos;
    }
}
