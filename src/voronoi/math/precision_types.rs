use derive_more::Add;
use derive_more::Sub;
use num::FromPrimitive;
use num::Signed;

use super::super::Point2d;
use super::super::Point3d;
use super::traits::Cross3d;
use super::traits::Dot;
use super::traits::Vector;
use super::traits::Vector2d;
use super::traits::Vector3d;
use crate::impl_vector2d;
use crate::impl_vector3d;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PrecisionError;

impl PrecisionError {
    pub fn check<F: Signed + FloatError>(a: &F, epsilon: f64) -> Result<(), PrecisionError> {
        if FloatError::is_too_close_to_zero(a, epsilon) {
            Err(PrecisionError)
        } else {
            Ok(())
        }
    }
}

pub const TRIANGLE_CONTAINS_EPSILON: f64 = 1e-15;
pub const TRIANGLE_INTERSECTION_TYPE_EPSILON: f64 = 1e-15;
pub const TETRAHEDRON_POINTS_ON_SAME_SIDE_EPSILON: f64 = 1e-15;
pub const DETERMINANT_3X3_EPSILON: f64 = 1e-15;
pub const DETERMINANT_4X4_EPSILON: f64 = 1e-15;
pub const DETERMINANT_5X5_EPSILON: f64 = 1e-15;

pub trait FloatError {
    fn is_too_close_to_zero(&self, epsilon: f64) -> bool;
}

impl FloatError for f64 {
    fn is_too_close_to_zero(&self, epsilon: f64) -> bool {
        self.abs() < epsilon
    }
}

pub type PrecisionFloat = num::BigRational;

impl FloatError for PrecisionFloat {
    fn is_too_close_to_zero(&self, _epsilon: f64) -> bool {
        false
    }
}

#[derive(Add, Sub, Clone, Debug)]
pub struct PrecisionPoint3d {
    pub x: PrecisionFloat,
    pub y: PrecisionFloat,
    pub z: PrecisionFloat,
}

impl PrecisionPoint3d {
    pub fn new(p: Point3d) -> Self {
        Self {
            x: PrecisionFloat::from_f64(p.x).unwrap(),
            y: PrecisionFloat::from_f64(p.y).unwrap(),
            z: PrecisionFloat::from_f64(p.z).unwrap(),
        }
    }
}

#[derive(Add, Sub, Clone, Debug)]
pub struct PrecisionPoint2d {
    pub x: PrecisionFloat,
    pub y: PrecisionFloat,
}

impl PrecisionPoint2d {
    pub fn new(p: Point2d) -> Self {
        Self {
            x: PrecisionFloat::from_f64(p.x).unwrap(),
            y: PrecisionFloat::from_f64(p.y).unwrap(),
        }
    }
}

impl Vector for PrecisionPoint3d {
    type Float = PrecisionFloat;
}

impl Cross3d for PrecisionPoint3d {
    fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y.clone() * other.z.clone() - other.y.clone() * self.z.clone(),
            y: self.z.clone() * other.x.clone() - other.z.clone() * self.x.clone(),
            z: self.x.clone() * other.y.clone() - other.x.clone() * self.y.clone(),
        }
    }
}

impl_vector2d!(PrecisionPoint2d);
impl_vector3d!(PrecisionPoint3d);

impl Vector for PrecisionPoint2d {
    type Float = PrecisionFloat;
}

impl Dot for PrecisionPoint3d {
    fn dot(self, other: Self) -> Self::Float {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}
