use bevy::prelude::Color;
use mpi::traits::Equivalence;

use crate::communication::Rank;
use crate::prelude::Float;

#[derive(Equivalence, Clone, Copy, Debug)]
pub struct RColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl RColor {
    pub const BLUE: RColor = RColor::rgb(0.0, 0.0, 1.0);
    pub const RED: RColor = RColor::rgb(1.0, 0.0, 0.0);
    pub const GREEN: RColor = RColor::rgb(0.0, 1.0, 0.0);
    pub const YELLOW: RColor = RColor::rgb(1.0, 1.0, 0.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn reds(v: Float) -> Self {
        Self::rgb(v.clamp(0.0, 1.0) as f32, 0.0, 0.0)
    }
}

impl From<RColor> for Color {
    fn from(color: RColor) -> Color {
        Color::rgba(color.r, color.g, color.b, color.a)
    }
}

impl From<Color> for RColor {
    fn from(color: Color) -> RColor {
        Self {
            r: color.r(),
            g: color.g(),
            b: color.b(),
            a: color.a(),
        }
    }
}

const COLORS: &[RColor] = &[RColor::RED, RColor::BLUE, RColor::GREEN, RColor::YELLOW];

pub fn color_map(rank: Rank) -> RColor {
    COLORS[(rank as usize).rem_euclid(COLORS.len())]
}
