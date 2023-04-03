use bevy::prelude::Component;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::prelude::Resource;
use glam::Vec2;

use crate::units::Length;
use crate::units::Vec2Length;
use crate::units::VecLength;

#[derive(Debug, Deref, DerefMut, Default, Component, Resource)]
pub struct CameraTransform {
    scale: Length,
}

impl CameraTransform {
    pub fn from_scale(scale: Length) -> Self {
        Self { scale }
    }

    pub fn position_to_pixels(&self, pos: VecLength) -> Vec2 {
        #[cfg(feature = "2d")]
        {
            pos.in_units(self.scale).as_vec2()
        }
        #[cfg(not(feature = "2d"))]
        {
            let pos = pos.in_units(self.scale);
            Vec2::new(pos.x as f32, pos.y as f32)
        }
    }

    pub fn length_to_pixels(&self, length: Length) -> f32 {
        length.in_units(self.scale) as f32
    }

    pub fn pixels_to_position(&self, pixel_pos: Vec2) -> Vec2Length {
        Vec2Length::from_vector_and_scale(pixel_pos.as_dvec2(), self.scale)
    }
}
