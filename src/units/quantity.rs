use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use super::dimension::Dimension;
use crate::units::NONE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity<const D: Dimension>(pub(super) f64);

impl<const D: Dimension> Quantity<D> {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }
}

impl Quantity<{ NONE }> {
    /// Get the value of a dimensionless quantity
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl<const D: Dimension> Quantity<D> {
    /// Unwrap the value of a quantity, regardless of whether
    /// it is dimensionless or not. Use this carefully, since the
    /// result depends on the underlying base units
    pub fn unwrap_value(&self) -> f64 {
        self.0
    }
}

impl<const D: Dimension> Add for Quantity<D> {
    type Output = Quantity<D>;

    fn add(self, rhs: Self) -> Self::Output {
        Quantity::<D>(self.0 + rhs.0)
    }
}

impl<const D: Dimension> Sub for Quantity<D> {
    type Output = Quantity<D>;

    fn sub(self, rhs: Self) -> Self::Output {
        Quantity::<D>(self.0 - rhs.0)
    }
}

impl<const D: Dimension> Mul<f64> for Quantity<D> {
    type Output = Quantity<D>;

    fn mul(self, rhs: f64) -> Self::Output {
        Quantity(self.0 * rhs)
    }
}

impl<const D: Dimension> Mul<Quantity<D>> for f64 {
    type Output = Quantity<D>;

    fn mul(self, rhs: Quantity<D>) -> Self::Output {
        Quantity(self * rhs.0)
    }
}

impl<const DL: Dimension, const DR: Dimension> Mul<Quantity<DR>> for Quantity<DL>
where
    Quantity<{ DL.dimension_mul(DR) }>:,
{
    type Output = Quantity<{ DL.dimension_mul(DR) }>;

    fn mul(self, rhs: Quantity<DR>) -> Self::Output {
        Quantity(self.0 * rhs.0)
    }
}

impl<const DL: Dimension, const DR: Dimension> Div<Quantity<DR>> for Quantity<DL>
where
    Quantity<{ DL.dimension_div(DR) }>:,
{
    type Output = Quantity<{ DL.dimension_div(DR) }>;

    fn div(self, rhs: Quantity<DR>) -> Self::Output {
        Quantity(self.0 / rhs.0)
    }
}
