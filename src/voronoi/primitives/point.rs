use std::ops::Sub;

use super::Float;

pub type Point2d = glam::DVec2;
pub type Point3d = glam::DVec3;

pub trait Vector: Sized + Sub<Self, Output = Self> {
    fn distance(&self, p: Self) -> Float;
    fn distance_squared(&self, p: Self) -> Float;
    fn normalize(&self) -> Self;
}

impl Vector for glam::DVec2 {
    fn distance(&self, p: Self) -> Float {
        (*self - p).length()
    }

    fn distance_squared(&self, p: Self) -> Float {
        (*self - p).length_squared()
    }

    fn normalize(&self) -> Self {
        self.normalize()
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
        self.normalize()
    }
}
