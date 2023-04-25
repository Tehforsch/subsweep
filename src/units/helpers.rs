use num::traits::NumOps;
use num::BigRational;
use num::Signed;

use super::Dimension;
use super::Quantity;

/// The default floating point type.
pub type Float = f64;

#[cfg(feature = "2d")]
/// The default vector type.
pub type MVec = glam::DVec2;
#[cfg(not(feature = "2d"))]
/// The default vector type.
pub type MVec = glam::DVec3;

impl<const D: Dimension> Quantity<MVec, D> {
    /// Construct the vector from just x and y, filling z with zero,
    /// if available. This helps with constructing vectors in examples
    /// without having to differentiate between 2D and 3D
    pub fn from_xy(x: Quantity<f64, D>, y: Quantity<f64, D>) -> Self {
        #[cfg(feature = "2d")]
        return Self::new(x, y);
        #[cfg(not(feature = "2d"))]
        return Self::new(x, y, Quantity::<f64, D>::zero());
    }
}

pub trait Num:
    num::Num + Clone + Signed + PartialOrd + FloatError + std::fmt::Debug + NumOps
{
}

impl<T> Num for T where T: num::Num + Clone + Signed + PartialOrd + FloatError + std::fmt::Debug {}

pub const ERROR_TRESHOLD: f64 = 1e-9;

pub trait FloatError {
    fn is_too_close_to_zero(&self) -> bool;
}

impl FloatError for f64 {
    fn is_too_close_to_zero(&self) -> bool {
        self.abs() < ERROR_TRESHOLD
    }
}

impl FloatError for BigRational {
    fn is_too_close_to_zero(&self) -> bool {
        false
    }
}
