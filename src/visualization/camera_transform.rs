use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use glam::Vec2;

use crate::units::Length;
use crate::units::VecLength;

#[derive(Debug, Deref, DerefMut, Default, Component)]
pub struct CameraTransform {
    scale: Length,
}

impl CameraTransform {
    pub fn from_scale(scale: Length) -> Self {
        Self { scale }
    }

    pub fn position_to_pixels(&self, pos: VecLength) -> Vec2 {
        pos.in_units(self.scale).as_vec2()
    }

    pub fn length_to_pixels(&self, length: Length) -> f32 {
        length.in_units(self.scale) as f32
    }

    pub fn pixels_to_position(&self, pixel_pos: Vec2) -> VecLength {
        VecLength::from_vector_and_scale(pixel_pos.as_dvec2(), self.scale)
    }
}
