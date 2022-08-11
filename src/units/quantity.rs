use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use super::dimension::Dimension;
use crate::units::NONE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity<S, const D: Dimension>(pub(super) S);

impl<S, const D: Dimension> Quantity<S, D> {
    pub fn new(value: S) -> Self {
        Self(value)
    }
}

impl<S> Quantity<S, { NONE }> {
    /// Get the value of a dimensionless quantity
    pub fn value(&self) -> &S {
        &self.0
    }
}

impl<S, const D: Dimension> Quantity<S, D> {
    /// Unwrap the value of a quantity, regardless of whether
    /// it is dimensionless or not. Use this carefully, since the
    /// result depends on the underlying base units
    pub fn unwrap_value(&self) -> &S {
        &self.0
    }
}

impl<S, const D: Dimension> Add for Quantity<S, D>
where
    S: Add<Output = S>,
{
    type Output = Quantity<S, D>;

    fn add(self, rhs: Self) -> Self::Output {
        Quantity::<S, D>(self.0 + rhs.0)
    }
}

impl<S, const D: Dimension> AddAssign for Quantity<S, D>
where
    S: AddAssign<S>,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<S, const D: Dimension> Sub for Quantity<S, D>
where
    S: Sub<Output = S>,
{
    type Output = Quantity<S, D>;

    fn sub(self, rhs: Self) -> Self::Output {
        Quantity::<S, D>(self.0 - rhs.0)
    }
}

impl<S, const D: Dimension> Mul<f64> for Quantity<S, D>
where
    S: Mul<f64, Output = S>,
{
    type Output = Quantity<S, D>;

    fn mul(self, rhs: f64) -> Self::Output {
        Quantity(self.0 * rhs)
    }
}

impl<S, const D: Dimension> Mul<Quantity<S, D>> for f64
where
    f64: Mul<S, Output = S>,
{
    type Output = Quantity<S, D>;

    fn mul(self, rhs: Quantity<S, D>) -> Self::Output {
        Quantity(self * rhs.0)
    }
}

impl<S, const DL: Dimension, const DR: Dimension> Mul<Quantity<S, DR>> for Quantity<S, DL>
where
    Quantity<S, { DL.dimension_mul(DR) }>:,
    S: Mul<S, Output = S>,
{
    type Output = Quantity<S, { DL.dimension_mul(DR) }>;

    fn mul(self, rhs: Quantity<S, DR>) -> Self::Output {
        Quantity(self.0 * rhs.0)
    }
}

impl<S, const DL: Dimension, const DR: Dimension> Div<Quantity<S, DR>> for Quantity<S, DL>
where
    Quantity<S, { DL.dimension_div(DR) }>:,
    S: Div<S, Output = S>,
{
    type Output = Quantity<S, { DL.dimension_div(DR) }>;

    fn div(self, rhs: Quantity<S, DR>) -> Self::Output {
        Quantity(self.0 / rhs.0)
    }
}
