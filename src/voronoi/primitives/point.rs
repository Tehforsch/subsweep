use std::ops::Mul;
use std::ops::Sub;

use super::Float;
use crate::peano_hilbert::PeanoKey2d;

pub type Point2d = glam::DVec2;
pub type Point3d = glam::DVec3;

pub trait DVector: Sized + Sub<Self, Output = Self> + Mul<Float, Output = Self> {
    fn distance(&self, p: Self) -> Float;
    fn distance_squared(&self, p: Self) -> Float;
    fn normalize(&self) -> Self;
    fn dot(self, other: Self) -> Float;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn get_peano_hilbert_key(self, min: Self, max: Self) -> PeanoKey2d;
}

impl DVector for glam::DVec2 {
    fn distance(&self, p: Self) -> Float {
        (*self - p).length()
    }

    fn distance_squared(&self, p: Self) -> Float {
        (*self - p).length_squared()
    }

    fn normalize(&self) -> Self {
        glam::DVec2::normalize(*self)
    }

    fn dot(self, other: Self) -> Float {
        glam::DVec2::dot(self, other)
    }

    fn min(self, other: Self) -> Self {
        glam::DVec2::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        glam::DVec2::max(self, other)
    }

    fn get_peano_hilbert_key(self, min: Self, max: Self) -> PeanoKey2d {
        PeanoKey2d::from_point_and_min_max_2d(self, min, max)
    }
}

impl DVector for glam::DVec3 {
    fn distance(&self, p: Self) -> Float {
        (*self - p).length()
    }

    fn distance_squared(&self, p: Self) -> Float {
        (*self - p).length_squared()
    }

    fn normalize(&self) -> Self {
        glam::DVec3::normalize(*self)
    }

    fn dot(self, other: Self) -> Float {
        glam::DVec3::dot(self, other)
    }

    fn min(self, other: Self) -> Self {
        glam::DVec3::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        glam::DVec3::max(self, other)
    }

    fn get_peano_hilbert_key(self, _min: Self, _max: Self) -> PeanoKey2d {
        // TODO implement this
        PeanoKey2d(0)
    }
}
