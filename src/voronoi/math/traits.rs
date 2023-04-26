use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use glam::DVec2;
use glam::DVec3;
use num::traits::NumOps;
use num::Signed;

use super::super::Point2d;
use super::super::Point3d;
use super::precision_types::FloatError;
use crate::units::Vec2Length;
use crate::units::Vec3Length;

pub trait Num:
    num::Num + Clone + Signed + PartialOrd + FloatError + std::fmt::Debug + NumOps
{
}

impl<T> Num for T where T: num::Num + Clone + Signed + PartialOrd + FloatError + std::fmt::Debug {}

pub trait Vector {
    type Float: Num;
}

pub trait DVector:
    Vector
    + Dot
    + Sized
    + Sub<Self, Output = Self>
    + Add<Self, Output = Self>
    + Mul<Self::Float, Output = Self>
    + Div<Self::Float, Output = Self>
    + Clone
    + MinMax
{
    fn distance(&self, p: Self) -> Self::Float;
    fn distance_squared(&self, p: Self) -> Self::Float;
    fn normalize(&self) -> Self;
    fn max_element(self) -> Self::Float;
}

pub trait Dot: Vector {
    fn dot(self, other: Self) -> Self::Float;
}

pub trait DVector2d: Vector {
    fn x(&self) -> <Self as Vector>::Float;
    fn y(&self) -> <Self as Vector>::Float;
}

pub trait DVector3d: Vector {
    fn cross(&self, other: &Self) -> Self;
    fn x(&self) -> <Self as Vector>::Float;
    fn y(&self) -> <Self as Vector>::Float;
    fn z(&self) -> <Self as Vector>::Float;
}

pub trait MinMax {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

macro_rules! impl_dvector_for_vector {
    ($vec: ident, $f: ty) => {
        impl Vector for $vec {
            type Float = $f;
        }

        impl Dot for $vec {
            fn dot(self, other: Self) -> $f {
                $vec::dot(self, other)
            }
        }

        impl DVector for $vec {
            fn distance(&self, p: Self) -> $f {
                (*self - p).length()
            }

            fn distance_squared(&self, p: Self) -> $f {
                (*self - p).length_squared()
            }

            fn normalize(&self) -> Self {
                $vec::normalize(*self)
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
    fn x(&self) -> <Self as Vector>::Float {
        self.x
    }

    fn y(&self) -> <Self as Vector>::Float {
        self.y
    }
}

impl DVector3d for Point3d {
    fn cross(&self, other: &Self) -> Self {
        Point3d::cross(*self, *other)
    }

    fn x(&self) -> <Self as Vector>::Float {
        self.x
    }

    fn y(&self) -> <Self as Vector>::Float {
        self.y
    }

    fn z(&self) -> <Self as Vector>::Float {
        self.z
    }
}

impl Vector for f64 {
    type Float = f64;
}

impl Dot for f64 {
    fn dot(self, other: Self) -> f64 {
        self * other
    }
}

impl DVector for f64 {
    fn distance(&self, p: Self) -> f64 {
        (*self - p).abs()
    }

    fn distance_squared(&self, p: Self) -> f64 {
        (*self - p).powi(2)
    }

    fn normalize(&self) -> Self {
        self.signum()
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
