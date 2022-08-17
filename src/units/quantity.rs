use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
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

impl<S, const D: Dimension> Neg for Quantity<S, D>
where
    S: Neg<Output = S>,
{
    type Output = Quantity<S, D>;

    fn neg(self) -> Self::Output {
        Quantity::<S, D>(-self.0)
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

impl<S, const D: Dimension> Mul<f32> for Quantity<S, D>
where
    S: Mul<f32, Output = S>,
{
    type Output = Quantity<S, D>;

    fn mul(self, rhs: f32) -> Self::Output {
        Quantity(self.0 * rhs)
    }
}

impl<S, const D: Dimension> Mul<Quantity<S, D>> for f32
where
    f32: Mul<S, Output = S>,
{
    type Output = Quantity<S, D>;

    fn mul(self, rhs: Quantity<S, D>) -> Self::Output {
        Quantity(self * rhs.0)
    }
}

impl<SL, SR, const DL: Dimension, const DR: Dimension> Mul<Quantity<SR, DR>> for Quantity<SL, DL>
where
    Quantity<SL, { DL.dimension_mul(DR) }>:,
    SL: Mul<SR, Output = SL>,
{
    type Output = Quantity<SL, { DL.dimension_mul(DR) }>;

    fn mul(self, rhs: Quantity<SR, DR>) -> Self::Output {
        Quantity(self.0 * rhs.0)
    }
}

impl<SL, SR, const DL: Dimension, const DR: Dimension> Div<Quantity<SR, DR>> for Quantity<SL, DL>
where
    Quantity<SL, { DL.dimension_div(DR) }>:,
    SL: Div<SR, Output = SL>,
{
    type Output = Quantity<SL, { DL.dimension_div(DR) }>;

    fn div(self, rhs: Quantity<SR, DR>) -> Self::Output {
        Quantity(self.0 / rhs.0)
    }
}

impl<S, const D: Dimension> Quantity<S, D>
where
    Quantity<S, { D.dimension_div(D) }>:,
    S: Div<S, Output = S> + Copy,
{
    pub fn to_value(&self, unit: impl Fn(f32) -> Quantity<S, D>) -> S {
        *(*self / unit(1.0)).unwrap_value()
    }
}

impl<S> std::fmt::Display for Quantity<S, { NONE }>
where
    S: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}
