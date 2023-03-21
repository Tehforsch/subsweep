use std::ops::Sub;

use super::Float;

pub type Point2d = glam::DVec2;
pub type Point3d = glam::DVec3;

pub trait Vector: Sized + Sub<Self, Output = Self> {
    fn distance(&self, p: Self) -> Float;
    fn distance_squared(&self, p: Self) -> Float;
    fn normalize(&self) -> Self;
    fn dot(&self, other: Self) -> Float;
}

impl Vector for glam::DVec2 {
    fn distance(&self, p: Self) -> Float {
        (*self - p).length()
    }

    fn distance_squared(&self, p: Self) -> Float {
        (*self - p).length_squared()
    }

    fn normalize(&self) -> Self {
        glam::DVec2::normalize(*self)
    }

    fn dot(&self, other: Self) -> Float {
        glam::DVec2::dot(*self, other)
    }
}

impl Vector for glam::DVec3 {
    fn distance(&self, p: Self) -> Float {
        (*self - p).length()
    }

    fn distance_squared(&self, p: Self) -> Float {
        (*self - p).length_squared()
    }

    fn normalize(&self) -> Self {
        glam::DVec3::normalize(*self)
    }

    fn dot(&self, other: Self) -> Float {
        glam::DVec3::dot(*self, other)
    }
}
