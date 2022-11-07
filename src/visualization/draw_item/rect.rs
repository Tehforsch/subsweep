use bevy::prelude::*;
use mpi::traits::Equivalence;

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
    fn translation(&self) -> &VecLength {
        &self.lower_left
    }

    fn set_translation(&mut self, pos: &VecLength) {
        self.lower_left = *pos;
    }

    fn get_color(&self) -> RColor {
        self.color
    }

    fn get_mesh() -> Mesh {
        todo!()
    }
}
