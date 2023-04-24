use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use glam::DVec2;
use glam::DVec3;

use crate::prelude::Num;
use crate::units::Vec2Length;
use crate::units::Vec3Length;

pub type Point2d = glam::DVec2;
pub type Point3d = glam::DVec3;

pub trait DVector:
    Sized
    + Sub<Self, Output = Self>
    + Add<Self, Output = Self>
    + Mul<Self::Float, Output = Self>
    + Div<Self::Float, Output = Self>
    + Copy
    + Clone
    + MinMax
{
    type Float: Num;
    fn distance(&self, p: Self) -> Self::Float;
    fn distance_squared(&self, p: Self) -> Self::Float;
    fn normalize(&self) -> Self;
    fn dot(self, other: Self) -> Self::Float;
    fn max_element(self) -> Self::Float;
}

pub trait DVector2d: DVector {
    fn x(&self) -> <Self as DVector>::Float;
    fn y(&self) -> <Self as DVector>::Float;
}

pub trait DVector3d: DVector {
    fn x(&self) -> <Self as DVector>::Float;
    fn y(&self) -> <Self as DVector>::Float;
    fn z(&self) -> <Self as DVector>::Float;
}

pub trait MinMax {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

macro_rules! impl_dvector_for_vector {
    ($vec: ident, $f: ty) => {
        impl DVector for $vec {
            type Float = $f;
            fn distance(&self, p: Self) -> $f {
                (*self - p).length()
            }

            fn distance_squared(&self, p: Self) -> $f {
                (*self - p).length_squared()
            }

            fn normalize(&self) -> Self {
                $vec::normalize(*self)
            }

            fn dot(self, other: Self) -> $f {
                $vec::dot(self, other)
            }

            fn max_element(self) -> $f {
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

impl_dvector_for_vector!(DVec2, f64);
impl_dvector_for_vector!(DVec3, f64);

impl_min_max_for_vector!(DVec2);
impl_min_max_for_vector!(DVec3);
impl_min_max_for_vector!(Vec2Length);
impl_min_max_for_vector!(Vec3Length);

impl DVector2d for Point2d {
    fn x(&self) -> <Self as DVector>::Float {
        self.x
    }

    fn y(&self) -> <Self as DVector>::Float {
        self.y
    }
}

impl DVector for f64 {
    type Float = f64;

    fn distance(&self, p: Self) -> f64 {
        (*self - p).abs()
    }

    fn distance_squared(&self, p: Self) -> f64 {
        (*self - p).powi(2)
    }

    fn normalize(&self) -> Self {
        self.signum()
    }

    fn dot(self, other: Self) -> f64 {
        self * other
    }

    fn max_element(self) -> f64 {
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
