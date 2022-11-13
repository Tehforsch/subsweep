use bevy::prelude::*;
use mpi::traits::Equivalence;

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
    fn translation(&self) -> &VecLength {
        &self.position
    }

    fn set_translation(&mut self, pos: &VecLength) {
        self.position = *pos;
    }

    fn get_color(&self) -> RColor {
        self.color
    }

    fn get_mesh() -> Mesh {
        shape::Circle::new(CIRCLE_RADIUS as f32).into()
    }

    fn get_scale(&self, _: &super::CameraTransform) -> Vec2 {
        Vec2::splat((self.radius.0 / CIRCLE_RADIUS) as f32)
    }
}
