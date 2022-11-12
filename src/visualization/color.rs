use bevy::prelude::Color;
use mpi::traits::Equivalence;

use crate::communication::Rank;
use crate::prelude::Float;

#[derive(Equivalence, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl RColor {
    pub const BLUE: RColor = RColor::rgb(0.0, 0.0, 1.0);
    pub const RED: RColor = RColor::rgb(1.0, 0.0, 0.0);
    pub const GREEN: RColor = RColor::rgb(0.0, 1.0, 0.0);
    pub const YELLOW: RColor = RColor::rgb(1.0, 1.0, 0.0);
    pub const BLACK: RColor = RColor::rgb(0.0, 0.0, 0.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: 255,
        }
    }

    pub fn reds(v: Float) -> Self {
        Self::rgb(v.clamp(0.0, 1.0) as f32, 0.0, 0.0)
    }
}

impl From<RColor> for Color {
    fn from(color: RColor) -> Color {
        Color::rgba(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        )
    }
}

impl From<Color> for RColor {
    fn from(color: Color) -> RColor {
        Self {
            r: (color.r() * 255.0) as u8,
            g: (color.g() * 255.0) as u8,
            b: (color.b() * 255.0) as u8,
            a: (color.a() * 255.0) as u8,
        }
    }
}

const COLORS: &[RColor] = &[RColor::RED, RColor::BLUE, RColor::GREEN, RColor::YELLOW];

pub fn color_map(rank: Rank) -> RColor {
    COLORS[(rank as usize).rem_euclid(COLORS.len())]
}
