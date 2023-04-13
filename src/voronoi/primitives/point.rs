use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use glam::DVec2;
use glam::DVec3;

use super::Float;
use crate::units::Vec2Length;
use crate::units::Vec3Length;

pub type Point2d = glam::DVec2;
pub type Point3d = glam::DVec3;

pub trait DVector:
    Sized
    + Sub<Self, Output = Self>
    + Add<Self, Output = Self>
    + Mul<Float, Output = Self>
    + Div<Float, Output = Self>
    + Copy
    + Clone
    + MinMax
{
    fn distance(&self, p: Self) -> Float;
    fn distance_squared(&self, p: Self) -> Float;
    fn normalize(&self) -> Self;
    fn dot(self, other: Self) -> Float;
    fn max_element(self) -> Float;
}

pub trait MinMax {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

macro_rules! impl_for_vector {
    ($vec: ident) => {
        impl DVector for $vec {
            fn distance(&self, p: Self) -> Float {
                (*self - p).length()
            }

            fn distance_squared(&self, p: Self) -> Float {
                (*self - p).length_squared()
            }

            fn normalize(&self) -> Self {
                $vec::normalize(*self)
            }

            fn dot(self, other: Self) -> Float {
                $vec::dot(self, other)
            }

            fn max_element(self) -> Float {
                $vec::max_element(self)
            }
        }
    };
}

macro_rules! impl_min_max_for_vector {
    ($vec:ident) => {
        impl MinMax for $vec {
            fn min(self, other: Self) -> Self {
                $vec::min(self, other)
            }

            fn max(self, other: Self) -> Self {
                $vec::max(self, other)
            }
        }
    };
}

impl_for_vector!(DVec2);
impl_for_vector!(DVec3);

impl_min_max_for_vector!(DVec2);
impl_min_max_for_vector!(DVec3);
impl_min_max_for_vector!(Vec2Length);
impl_min_max_for_vector!(Vec3Length);

impl DVector for f64 {
    fn distance(&self, p: Self) -> Float {
        (*self - p).abs()
    }

    fn distance_squared(&self, p: Self) -> Float {
        (*self - p).powi(2)
    }

    fn normalize(&self) -> Self {
        self.signum()
    }

    fn dot(self, other: Self) -> Float {
        self * other
    }

    fn max_element(self) -> Float {
        self
    }
}

impl MinMax for f64 {
    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    fn max(self, other: Self) -> Self {
        self.max(other)
    }
}
